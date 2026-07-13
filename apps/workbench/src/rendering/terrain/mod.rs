mod cache;
pub(super) mod control;
mod copy_timing;
mod descriptors;
mod lod;
mod pipeline;
mod probe;
mod state;
mod transfer;

use anyhow::{Context, Result};
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

pub use self::lod::TerrainLodSettings;
pub use self::probe::TerrainProbe;

const TERRAIN_REVISION: &str = "gpu-streamed-terrain-v1";
const PATCH_GROUP_COUNT: u32 = 400;
const STATS_BYTES: u64 = 32;
const LOD_STATS_BYTES: u64 = 64;
const QUERY_COUNT: u32 = 3;

pub struct TerrainRenderer {
    pipeline: TerrainPipeline,
    transfer: TerrainTransfer,
    stats: ID3D12Resource,
    seams: ID3D12Resource,
    lod_stats: ID3D12Resource,
    stats_readback: ID3D12Resource,
    seams_readback: ID3D12Resource,
    lod_stats_readback: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    enabled: bool,
    lod_settings: TerrainLodSettings,
    generation: u64,
    published: Option<PublishedTerrain>,
    staged: Option<TerrainPublication>,
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
    pub clear_depth_semantic: bool,
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
        let lod_stats = unsafe {
            create_buffer(
                device,
                LOD_STATS_BYTES,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let transfer = unsafe { TerrainTransfer::new(device, &stats, &seams, &lod_stats) }?;
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
        let lod_stats_readback = unsafe {
            create_buffer(
                device,
                LOD_STATS_BYTES,
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
            lod_stats,
            stats_readback,
            seams_readback,
            lod_stats_readback,
            query_heap,
            timestamp_readback,
            timestamp_frequency,
            width,
            height,
            enabled: false,
            lod_settings: TerrainLodSettings::default(),
            generation: 0,
            published: None,
            staged: None,
        })
    }

    pub fn reserve(&mut self, config: LoadConfig) -> Result<TerrainReservationReport> {
        let protected = self
            .published
            .iter()
            .flat_map(|value| value.active.iter().map(|entry| entry.slot))
            .chain(
                self.staged
                    .iter()
                    .flat_map(|value| value.active.iter().map(|entry| entry.slot)),
            )
            .collect();
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
        unsafe { self.stage_frame(command_list) }?;
        Ok(self.commit_staged())
    }

    pub(in crate::rendering) unsafe fn stage_frame(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Result<bool> {
        if self.staged.is_some() {
            return Ok(false);
        }
        let Some(publication) = unsafe { self.transfer.poll(command_list) }? else {
            return Ok(false);
        };
        self.staged = Some(publication);
        Ok(true)
    }

    pub(in crate::rendering) fn commit_staged(&mut self) -> Option<TerrainTransactionReport> {
        let TerrainPublication {
            active,
            tiles,
            report,
        } = self.staged.take()?;
        self.generation += 1;
        self.published = Some(PublishedTerrain {
            config: report.config,
            active,
            tiles,
            generation: self.generation,
            report: report.clone(),
        });
        Some(report)
    }

    pub(in crate::rendering) fn discard_staged(&mut self) -> Option<TerrainTransactionReport> {
        self.staged.take().map(|publication| publication.report)
    }

    pub(in crate::rendering) fn staged_report(&self) -> Option<&TerrainTransactionReport> {
        self.staged.as_ref().map(|publication| &publication.report)
    }

    pub(in crate::rendering) fn staged_assignments(&self) -> Option<&[TerrainAssignment]> {
        self.staged
            .as_ref()
            .map(|publication| publication.active.as_slice())
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
        let constants = constants(
            frame.scene,
            snapshot,
            self.lod_settings,
            self.width,
            self.height,
        );
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
            uav_barrier(command_list, &self.lod_stats);
            command_list.SetPipelineState(&self.pipeline.seam);
            command_list.Dispatch(25, 2, 1);
            uav_barrier(command_list, &self.seams);
            if self.lod_settings.enabled {
                command_list.SetPipelineState(&self.pipeline.lod_seam);
                command_list.Dispatch(PATCH_GROUP_COUNT, 2, 1);
                uav_barrier(command_list, &self.lod_stats);
            }
            if frame.probe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1);
            }
            command_list.OMSetRenderTargets(
                2,
                Some(frame.render_targets.as_ptr()),
                false,
                Some(&frame.depth_target),
            );
            if frame.clear_depth_semantic {
                command_list.ClearRenderTargetView(frame.render_targets[1], &[0.0; 4], None);
                command_list.ClearDepthStencilView(
                    frame.depth_target,
                    D3D12_CLEAR_FLAG_DEPTH,
                    0.0,
                    0,
                    None,
                );
            }
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
            uav_barrier(command_list, &self.lod_stats);
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
                transition(
                    command_list,
                    &self.lod_stats,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.lod_stats_readback,
                    0,
                    &self.lod_stats,
                    0,
                    LOD_STATS_BYTES,
                );
                transition(
                    command_list,
                    &self.lod_stats,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
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
}

fn constants(
    scene: &SceneState,
    snapshot: &PublishedTerrain,
    lod_settings: TerrainLodSettings,
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
    for (destination, entry) in constants[20..48].iter_mut().zip(&snapshot.active) {
        *destination = entry.slot | (entry.region_id << 6);
    }
    let camera_patch = lod::camera_patch(scene.camera());
    constants[48..56].copy_from_slice(&[
        camera_patch[0] as u32,
        camera_patch[1] as u32,
        lod_settings.near_patch_radius,
        lod_settings.middle_patch_radius,
        u32::from(lod_settings.enabled),
        lod_settings.forced_lod.map_or(0, |value| value + 1),
        0,
        0,
    ]);
    constants
}
