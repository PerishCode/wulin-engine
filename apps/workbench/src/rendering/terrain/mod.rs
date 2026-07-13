mod cache;
mod control;
mod copy_timing;
mod descriptors;
mod pipeline;
mod probe;
mod transfer;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

use crate::load::LoadConfig;
use crate::scene::SceneState;
use crate::terrain::{
    TerrainAssignment, TerrainIoMetrics, TerrainReservationReport, TerrainTransactionReport,
    TerrainUpload,
};

use self::pipeline::{TERRAIN_CONSTANT_COUNT, TerrainPipeline};
use self::transfer::{TerrainPublication, TerrainTransfer};
use super::resident::{create_buffer, create_query_heap, set_viewport, transition, uav_barrier};

pub use self::probe::TerrainProbe;

const TERRAIN_REVISION: &str = "gpu-streamed-terrain-v1";
const PATCH_GROUP_COUNT: u32 = 400;
const PATCHES_PER_REGION: u32 = 16;
const VERTICES_PER_PATCH: u32 = 81;
const TRIANGLES_PER_PATCH: u32 = 128;
const STATS_BYTES: u64 = 32;
const QUERY_COUNT: u32 = 3;

pub struct TerrainRenderer {
    pipeline: TerrainPipeline,
    transfer: TerrainTransfer,
    stats: ID3D12Resource,
    seams: ID3D12Resource,
    stats_readback: ID3D12Resource,
    seams_readback: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    enabled: bool,
    generation: u64,
    published: Option<PublishedTerrain>,
}

struct PublishedTerrain {
    config: LoadConfig,
    active: Vec<TerrainAssignment>,
    tiles: Vec<terrain_format::TerrainTile>,
    generation: u64,
    report: TerrainTransactionReport,
}

pub struct TerrainFrame<'a> {
    pub scene: &'a SceneState,
    pub render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
    pub depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub probe: bool,
}

impl TerrainRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let stats = unsafe {
            create_buffer(
                device,
                STATS_BYTES,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let seams = unsafe {
            create_buffer(
                device,
                STATS_BYTES,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let transfer = unsafe { TerrainTransfer::new(device, &stats, &seams) }?;
        let pipeline = unsafe { TerrainPipeline::new(device) }?;
        let stats_readback = unsafe {
            create_buffer(
                device,
                STATS_BYTES,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let seams_readback = unsafe {
            create_buffer(
                device,
                STATS_BYTES,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let query_heap = unsafe { create_query_heap(device) }?;
        let timestamp_readback = unsafe {
            create_buffer(
                device,
                QUERY_COUNT as u64 * size_of::<u64>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        Ok(Self {
            pipeline,
            transfer,
            stats,
            seams,
            stats_readback,
            seams_readback,
            query_heap,
            timestamp_readback,
            timestamp_frequency,
            width,
            height,
            enabled: false,
            generation: 0,
            published: None,
        })
    }

    pub fn reserve(&mut self, config: LoadConfig) -> Result<TerrainReservationReport> {
        let protected = self
            .published
            .as_ref()
            .map(|value| value.active.iter().map(|entry| entry.slot).collect())
            .unwrap_or_default();
        self.transfer.reserve(config, &protected)
    }

    pub fn cancel(&mut self, transaction_id: u64) -> Result<()> {
        self.transfer.cancel(transaction_id)
    }

    pub unsafe fn submit(
        &mut self,
        transaction_id: u64,
        uploads: Vec<TerrainUpload>,
        io: TerrainIoMetrics,
        queue: &ID3D12CommandQueue,
        fence: &ID3D12Fence,
        release_fence: u64,
    ) -> Result<TerrainTransactionReport> {
        unsafe {
            self.transfer
                .submit(transaction_id, uploads, io, queue, fence, release_fence)
        }
    }

    pub unsafe fn prepare_frame(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Result<Option<TerrainTransactionReport>> {
        let Some(TerrainPublication {
            active,
            tiles,
            report,
        }) = unsafe { self.transfer.poll(command_list) }?
        else {
            return Ok(None);
        };
        self.generation += 1;
        self.published = Some(PublishedTerrain {
            config: report.config,
            active,
            tiles,
            generation: self.generation,
            report: report.clone(),
        });
        Ok(Some(report))
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        frame: TerrainFrame<'_>,
    ) -> Result<()> {
        let snapshot = self
            .published
            .as_ref()
            .context("terrain renderer has no published snapshot")?;
        let constants = constants(frame.scene, snapshot, self.width, self.height);
        let heap = self.transfer.descriptor_heap();
        let gpu = unsafe { heap.GetGPUDescriptorHandleForHeapStart() };
        unsafe {
            command_list.SetDescriptorHeaps(&[Some(heap.clone())]);
            if frame.probe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0);
            }
            command_list.SetComputeRootSignature(&self.pipeline.root);
            command_list.SetComputeRoot32BitConstants(
                0,
                TERRAIN_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootDescriptorTable(1, gpu);
            command_list.SetPipelineState(&self.pipeline.reset);
            command_list.Dispatch(1, 1, 1);
            uav_barrier(command_list, &self.stats);
            uav_barrier(command_list, &self.seams);
            command_list.SetPipelineState(&self.pipeline.seam);
            command_list.Dispatch(25, 2, 1);
            uav_barrier(command_list, &self.seams);
            if frame.probe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1);
            }
            command_list.OMSetRenderTargets(
                2,
                Some(frame.render_targets.as_ptr()),
                false,
                Some(&frame.depth_target),
            );
            command_list.ClearRenderTargetView(frame.render_targets[1], &[0.0; 4], None);
            command_list.ClearDepthStencilView(
                frame.depth_target,
                D3D12_CLEAR_FLAG_DEPTH,
                0.0,
                0,
                None,
            );
            set_viewport(command_list, self.width, self.height);
            command_list.SetGraphicsRootSignature(&self.pipeline.root);
            command_list.SetGraphicsRoot32BitConstants(
                0,
                TERRAIN_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootDescriptorTable(1, gpu);
            command_list.SetPipelineState(&self.pipeline.graphics);
            let mesh_list: ID3D12GraphicsCommandList6 = command_list.cast()?;
            mesh_list.DispatchMesh(PATCH_GROUP_COUNT, 1, 1);
            uav_barrier(command_list, &self.stats);
            if frame.probe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 2);
                for (source, destination) in [
                    (&self.stats, &self.stats_readback),
                    (&self.seams, &self.seams_readback),
                ] {
                    transition(
                        command_list,
                        source,
                        D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                    );
                    command_list.CopyBufferRegion(destination, 0, source, 0, STATS_BYTES);
                    transition(
                        command_list,
                        source,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                        D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    );
                }
                command_list.ResolveQueryData(
                    &self.query_heap,
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    0,
                    QUERY_COUNT,
                    &self.timestamp_readback,
                    0,
                );
            }
        }
        Ok(())
    }

    pub fn enable(&mut self) -> Result<()> {
        ensure!(
            self.published.is_some(),
            "terrain requires a published snapshot"
        );
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.published.as_ref().map(|value| value.config)
    }

    pub fn status_json(&self) -> Value {
        json!({
            "revision": TERRAIN_REVISION,
            "enabled": self.enabled,
            "published": self.published.as_ref().map(|value| json!({
                "config": value.config,
                "generation": value.generation,
                "active": value.active,
                "transaction": value.report,
            })),
            "transfer": self.transfer.status_json(),
            "submission": {
                "meshDispatchCount": 1,
                "meshDispatchGroups": [PATCH_GROUP_COUNT, 1, 1],
                "seamDispatchCount": 1,
                "seamDispatchGroups": [25, 2, 1],
            },
        })
    }

    pub fn arm_copy_gate(&mut self) -> Result<u64> {
        self.transfer.arm_gate()
    }

    pub unsafe fn release_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.transfer.release_gate() }
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.transfer.wait_idle() }
    }
}

fn constants(
    scene: &SceneState,
    snapshot: &PublishedTerrain,
    width: u32,
    height: u32,
) -> [u32; TERRAIN_CONSTANT_COUNT as usize] {
    let mut constants = [0u32; TERRAIN_CONSTANT_COUNT as usize];
    for (destination, value) in constants[..16].iter_mut().zip(
        scene
            .view_projection(width as f32 / height as f32)
            .to_cols_array(),
    ) {
        *destination = value.to_bits();
    }
    constants[16..20].copy_from_slice(&[
        snapshot.active.len() as u32,
        snapshot.config.active_radius * 2 + 1,
        width,
        height,
    ]);
    for (destination, entry) in constants[20..].iter_mut().zip(&snapshot.active) {
        *destination = entry.slot | (entry.region_id << 6);
    }
    constants
}
