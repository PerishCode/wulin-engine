use std::mem::size_of;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D12::*;

use crate::address::GlobalRegionConfig;
use crate::async_resident::{
    AsyncReservationReport, AsyncTransactionReport, ObjectSourceNamespace, PayloadPreparation,
};
use crate::load::{LoadConfig, MAX_VISIBLE_INSTANCES};
use crate::resident::RegionUpload;
use crate::scene::SceneState;

use super::pipeline::{ASYNC_CONSTANT_COUNT, AsyncResidentPipeline};
use super::transfer::{AsyncTransfer, Publication};
use crate::rendering::load::{LoadProbe, PROBE_ITERATIONS};
use crate::rendering::resident::{
    QUERY_COUNT, create_buffer, create_query_heap, read_values, set_viewport, transition,
    uav_barrier,
};
use crate::rendering::terrain::TerrainProjection;

mod global;
mod payload;
mod status;

pub(in crate::rendering) use payload::ActivePayloadReadback;

pub struct AsyncResidentRenderer {
    pipeline: AsyncResidentPipeline,
    transfer: AsyncTransfer,
    visible_instances: ID3D12Resource,
    draw_arguments: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    argument_readback: ID3D12Resource,
    active_payload_readback: ID3D12Resource,
    active_payload_allocation_bytes: u64,
    active_payload_probe_count: u64,
    active_payload_copy_count: u64,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    published: Option<PublishedSnapshot>,
    staged: Option<Publication>,
}

pub(in crate::rendering) struct PublishedSnapshot {
    pub config: LoadConfig,
    pub global_config: Option<GlobalRegionConfig>,
    pub object_source_namespace: Option<ObjectSourceNamespace>,
    pub object_stable_seed_namespace: Option<ObjectSourceNamespace>,
    pub object_page_checksums: Option<Vec<[u8; 32]>>,
    pub active_slots: Vec<u32>,
}

impl PublishedSnapshot {
    pub(in crate::rendering) fn projection(&self) -> Result<TerrainProjection> {
        TerrainProjection::for_objects(self.config, self.object_source_namespace)
    }
}

impl AsyncResidentRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let pipeline = unsafe { AsyncResidentPipeline::new(device) }?;
        let transfer = unsafe { AsyncTransfer::new(device) }?;
        let visible_instances = unsafe {
            create_buffer(
                device,
                u64::from(MAX_VISIBLE_INSTANCES) * size_of::<u32>() as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let draw_arguments = unsafe {
            create_buffer(
                device,
                size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let query_heap = unsafe { create_query_heap(device) }?;
        let timestamp_readback = unsafe {
            create_buffer(
                device,
                u64::from(QUERY_COUNT) * size_of::<u64>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let argument_readback = unsafe {
            create_buffer(
                device,
                size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let (active_payload_readback, active_payload_allocation_bytes) =
            unsafe { payload::create_readback(device) }?;
        Ok(Self {
            pipeline,
            transfer,
            visible_instances,
            draw_arguments,
            query_heap,
            timestamp_readback,
            argument_readback,
            active_payload_readback,
            active_payload_allocation_bytes,
            active_payload_probe_count: 0,
            active_payload_copy_count: 0,
            timestamp_frequency,
            width,
            height,
            published: None,
            staged: None,
        })
    }

    pub unsafe fn schedule(
        &mut self,
        config: LoadConfig,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<AsyncTransactionReport> {
        let protected = self.protected_slots();
        unsafe {
            self.transfer.schedule(
                config,
                &protected,
                direct_queue,
                direct_fence,
                direct_release_fence,
            )
        }
    }

    pub fn reserve(&mut self, config: LoadConfig) -> Result<AsyncReservationReport> {
        self.transfer.reserve(config, &self.protected_slots())
    }

    pub(in crate::rendering) fn reserve_composition(
        &mut self,
        config: LoadConfig,
    ) -> Result<AsyncReservationReport> {
        self.transfer
            .reserve_composition(config, &self.protected_slots())
    }

    pub fn cancel_reservation(&mut self, transaction_id: u64) -> Result<()> {
        self.transfer.cancel_reservation(transaction_id)
    }

    pub unsafe fn submit(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        preparation_ms: f64,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<AsyncTransactionReport> {
        unsafe {
            self.transfer.submit(
                transaction_id,
                uploads,
                PayloadPreparation::cooked(preparation_ms),
                direct_queue,
                direct_fence,
                direct_release_fence,
            )
        }
    }

    pub(in crate::rendering) unsafe fn submit_generated(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        generation_ms: f64,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<AsyncTransactionReport> {
        unsafe {
            self.transfer.submit(
                transaction_id,
                uploads,
                PayloadPreparation::generated(generation_ms),
                direct_queue,
                direct_fence,
                direct_release_fence,
            )
        }
    }

    pub unsafe fn prepare_frame(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Option<AsyncTransactionReport> {
        unsafe { self.stage_frame(command_list) };
        self.commit_staged()
    }

    pub(in crate::rendering) unsafe fn stage_frame(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> bool {
        if self.staged.is_some() {
            return false;
        }
        let Some(publication) = (unsafe { self.transfer.poll_publication(command_list) }) else {
            return false;
        };
        self.staged = Some(publication);
        true
    }

    pub(in crate::rendering) fn commit_staged(&mut self) -> Option<AsyncTransactionReport> {
        let Publication {
            config,
            active_slots,
            report,
        } = self.staged.take()?;
        let global_config = report.global_config;
        let object_source_namespace = report.object_source_namespace;
        let object_stable_seed_namespace = report.object_stable_seed_namespace;
        let object_page_checksums = report.object_page_checksums.clone();
        self.published = Some(PublishedSnapshot {
            config,
            global_config,
            object_source_namespace,
            object_stable_seed_namespace,
            object_page_checksums,
            active_slots,
        });
        Some(report)
    }

    pub(in crate::rendering) fn discard_staged(&mut self) -> Option<AsyncTransactionReport> {
        self.staged.take().map(|publication| publication.report)
    }

    pub(in crate::rendering) fn staged_report(&self) -> Option<&AsyncTransactionReport> {
        self.staged.as_ref().map(|publication| &publication.report)
    }

    pub(in crate::rendering) fn staged_active_slots(&self) -> Option<&[u32]> {
        self.staged
            .as_ref()
            .map(|publication| publication.active_slots.as_slice())
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        scene: &SceneState,
        render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
        depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
        probe: bool,
    ) -> Result<()> {
        let snapshot = self
            .published
            .as_ref()
            .context("async resident renderer has no published snapshot")?;
        let constants = async_constants(scene, snapshot, self.width, self.height)?;
        let heap = self.transfer.descriptor_heap();
        let gpu_start = unsafe { heap.GetGPUDescriptorHandleForHeapStart() };
        unsafe { command_list.SetDescriptorHeaps(&[Some(heap.clone())]) };
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0) };
        }
        unsafe {
            command_list.SetComputeRootSignature(&self.pipeline.compute_root);
            command_list.SetComputeRoot32BitConstants(
                0,
                ASYNC_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootDescriptorTable(1, gpu_start);
            command_list.SetComputeRootUnorderedAccessView(
                2,
                self.visible_instances.GetGPUVirtualAddress(),
            );
            command_list
                .SetComputeRootUnorderedAccessView(3, self.draw_arguments.GetGPUVirtualAddress());
            let [groups_x, groups_y, groups_z] = snapshot.config.dispatch();
            let iterations = if probe { PROBE_ITERATIONS } else { 1 };
            for _ in 0..iterations {
                command_list.SetPipelineState(&self.pipeline.reset);
                command_list.Dispatch(1, 1, 1);
                uav_barrier(command_list, &self.draw_arguments);
                command_list.SetPipelineState(&self.pipeline.cull);
                command_list.Dispatch(groups_x, groups_y, groups_z);
                uav_barrier(command_list, &self.visible_instances);
                uav_barrier(command_list, &self.draw_arguments);
            }
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1) };
        }

        unsafe {
            transition(
                command_list,
                &self.visible_instances,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.draw_arguments,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            );
            command_list.OMSetRenderTargets(
                2,
                Some(render_targets.as_ptr()),
                false,
                Some(&depth_target),
            );
            command_list.ClearRenderTargetView(render_targets[1], &[0.0; 4], None);
            command_list.ClearDepthStencilView(depth_target, D3D12_CLEAR_FLAG_DEPTH, 0.0, 0, None);
            set_viewport(command_list, self.width, self.height);
            command_list.SetGraphicsRootSignature(&self.pipeline.graphics_root);
            command_list.SetPipelineState(&self.pipeline.graphics);
            command_list.SetGraphicsRoot32BitConstants(
                0,
                ASYNC_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootDescriptorTable(1, gpu_start);
            command_list.SetGraphicsRootShaderResourceView(
                2,
                self.visible_instances.GetGPUVirtualAddress(),
            );
            command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 2) };
        }
        let iterations = if probe { PROBE_ITERATIONS } else { 1 };
        unsafe {
            for _ in 0..iterations {
                command_list.ClearDepthStencilView(
                    depth_target,
                    D3D12_CLEAR_FLAG_DEPTH,
                    0.0,
                    0,
                    None,
                );
                command_list.ExecuteIndirect(
                    &self.pipeline.command_signature,
                    1,
                    &self.draw_arguments,
                    0,
                    None,
                    0,
                );
            }
        }
        if probe {
            unsafe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 3);
                command_list.ResolveQueryData(
                    &self.query_heap,
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    0,
                    QUERY_COUNT,
                    &self.timestamp_readback,
                    0,
                );
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.argument_readback,
                    0,
                    &self.draw_arguments,
                    0,
                    size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                );
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        } else {
            unsafe {
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                )
            };
        }
        unsafe {
            transition(
                command_list,
                &self.visible_instances,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            )
        };
        Ok(())
    }

    pub unsafe fn read_probe(&self) -> Result<LoadProbe> {
        let config = self
            .config()
            .context("async resident mode is not published")?;
        let timestamps = unsafe { read_values::<u64>(&self.timestamp_readback, 4) }?;
        let arguments = unsafe { read_values::<u32>(&self.argument_readback, 4) }?;
        if arguments[0] != 6 || arguments[2] != 0 || arguments[3] != 0 {
            bail!("async indirect draw arguments are invalid: {arguments:?}");
        }
        if arguments[1] > config.candidate_instance_count() {
            bail!("async visible instance count exceeds active candidates");
        }
        let milliseconds = |start: usize, end: usize| {
            timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
                / self.timestamp_frequency as f64
                / PROBE_ITERATIONS as f64
        };
        Ok(LoadProbe {
            config,
            logical_instance_count: config.logical_instance_count(),
            active_region_count: config.active_region_count(),
            candidate_instance_count: config.candidate_instance_count(),
            dispatch: config.dispatch(),
            indirect_draw_count: 1,
            probe_iterations: PROBE_ITERATIONS,
            visible_instance_count: arguments[1],
            gpu_compaction_ms: milliseconds(0, 1),
            gpu_draw_ms: milliseconds(2, 3),
            gpu_total_ms: milliseconds(0, 3),
        })
    }
}

fn async_constants(
    scene: &SceneState,
    snapshot: &PublishedSnapshot,
    width: u32,
    height: u32,
) -> Result<[u32; ASYNC_CONSTANT_COUNT as usize]> {
    let mut constants = [0u32; ASYNC_CONSTANT_COUNT as usize];
    let projection = snapshot.projection()?;
    let camera = projection.camera(scene.camera());
    for (destination, value) in constants[..16]
        .iter_mut()
        .zip(crate::scene::view_projection(camera, width as f32 / height as f32).to_cols_array())
    {
        *destination = value.to_bits();
    }
    constants[16] = snapshot.config.active_region_count();
    constants[17] = MAX_VISIBLE_INSTANCES;
    for (index, slot) in snapshot.active_slots.iter().copied().enumerate() {
        let semantic_region = if projection.is_canonical() {
            projection.region_id(index, 0)?
        } else {
            0
        };
        constants[20 + index] = slot | (semantic_region << 6);
    }
    Ok(constants)
}
