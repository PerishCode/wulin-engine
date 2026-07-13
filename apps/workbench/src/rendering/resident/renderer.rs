use std::mem::size_of;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D12::*;

use crate::load::{LoadConfig, MAX_VISIBLE_INSTANCES};
use crate::resident::{ACTIVE_MAPPING_BYTES, CACHE_REGION_CAPACITY, REGION_INSTANCE_BYTES};
use crate::resident::{RegionCache, StreamReport};
use crate::scene::SceneState;

use super::pipeline::{RESIDENT_CONSTANT_COUNT, ResidentPipeline};
use super::resources::{
    QUERY_COUNT, create_buffer, create_query_heap, read_values, record_stream_copies, set_viewport,
    transition, uav_barrier, write_staging,
};
use crate::rendering::load::{LoadProbe, PROBE_ITERATIONS};

pub struct ResidentRenderer {
    pipeline: ResidentPipeline,
    instances: ID3D12Resource,
    active_regions: ID3D12Resource,
    instance_upload: ID3D12Resource,
    active_upload: ID3D12Resource,
    visible_instances: ID3D12Resource,
    draw_arguments: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    argument_readback: ID3D12Resource,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    config: Option<LoadConfig>,
    cache: RegionCache,
    pending_stream: Option<PendingStream>,
}

struct PendingStream {
    next_cache: RegionCache,
    report: StreamReport,
    copy_slots: Vec<u32>,
    started_at: Instant,
}

impl ResidentRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let pipeline = unsafe { ResidentPipeline::new(device) }?;
        let instance_buffer_bytes = (CACHE_REGION_CAPACITY * REGION_INSTANCE_BYTES) as u64;
        let instances = unsafe {
            create_buffer(
                device,
                instance_buffer_bytes,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let active_regions = unsafe {
            create_buffer(
                device,
                ACTIVE_MAPPING_BYTES as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let instance_upload = unsafe {
            create_buffer(
                device,
                instance_buffer_bytes,
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let active_upload = unsafe {
            create_buffer(
                device,
                ACTIVE_MAPPING_BYTES as u64,
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
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
        Ok(Self {
            pipeline,
            instances,
            active_regions,
            instance_upload,
            active_upload,
            visible_instances,
            draw_arguments,
            query_heap,
            timestamp_readback,
            argument_readback,
            timestamp_frequency,
            width,
            height,
            config: None,
            cache: RegionCache::default(),
            pending_stream: None,
        })
    }

    pub unsafe fn prepare_stream(&mut self, config: LoadConfig) -> Result<()> {
        if self.pending_stream.is_some() {
            bail!("a resident stream transaction is already pending");
        }
        let started_at = Instant::now();
        let plan = self.cache.plan(config)?;
        unsafe { write_staging(&self.instance_upload, &self.active_upload, &plan) }?;
        self.config = Some(config);
        self.pending_stream = Some(PendingStream {
            next_cache: plan.next_cache,
            report: plan.report,
            copy_slots: plan.uploads.iter().map(|upload| upload.slot).collect(),
            started_at,
        });
        Ok(())
    }

    pub fn disable(&mut self) {
        self.config = None;
        self.pending_stream = None;
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.config
    }

    pub fn has_pending_stream(&self) -> bool {
        self.pending_stream.is_some()
    }

    pub fn complete_stream(&mut self) -> Result<StreamReport> {
        let pending = self
            .pending_stream
            .take()
            .context("resident stream completed without a pending transaction")?;
        self.cache = pending.next_cache;
        let mut report = pending.report;
        report.transaction_ms = pending.started_at.elapsed().as_secs_f64() * 1_000.0;
        Ok(report)
    }

    unsafe fn record_stream(&self, command_list: &ID3D12GraphicsCommandList) {
        let pending = self
            .pending_stream
            .as_ref()
            .expect("record_stream requires a pending transaction");
        unsafe {
            record_stream_copies(
                command_list,
                &self.instances,
                &self.active_regions,
                &self.instance_upload,
                &self.active_upload,
                &pending.copy_slots,
            )
        };
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        scene: &SceneState,
        render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
        depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
        probe: bool,
    ) -> Result<()> {
        let config = self.config.context("resident renderer is not configured")?;
        if self.pending_stream.is_some() {
            unsafe { self.record_stream(command_list) };
        }
        let constants = resident_constants(scene, config, self.width, self.height);
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0) };
        }
        unsafe {
            command_list.SetComputeRootSignature(&self.pipeline.compute_root);
            command_list.SetComputeRoot32BitConstants(
                0,
                RESIDENT_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootShaderResourceView(1, self.instances.GetGPUVirtualAddress());
            command_list
                .SetComputeRootShaderResourceView(2, self.active_regions.GetGPUVirtualAddress());
            command_list.SetComputeRootUnorderedAccessView(
                3,
                self.visible_instances.GetGPUVirtualAddress(),
            );
            command_list
                .SetComputeRootUnorderedAccessView(4, self.draw_arguments.GetGPUVirtualAddress());
            let [groups_x, groups_y, groups_z] = config.dispatch();
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
                RESIDENT_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list
                .SetGraphicsRootShaderResourceView(1, self.instances.GetGPUVirtualAddress());
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
                );
            }
        }
        unsafe {
            transition(
                command_list,
                &self.visible_instances,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
        }
        Ok(())
    }

    pub unsafe fn read_probe(&self) -> Result<LoadProbe> {
        let config = self.config.context("resident renderer is not configured")?;
        let timestamps =
            unsafe { read_values::<u64>(&self.timestamp_readback, QUERY_COUNT as usize) }?;
        let arguments = unsafe { read_values::<u32>(&self.argument_readback, 4) }?;
        if arguments[0] != 6 || arguments[2] != 0 || arguments[3] != 0 {
            bail!("indirect draw arguments are invalid: {arguments:?}");
        }
        if arguments[1] > config.candidate_instance_count() {
            bail!("visible instance count exceeds active candidates");
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

fn resident_constants(
    scene: &SceneState,
    config: LoadConfig,
    width: u32,
    height: u32,
) -> [u32; RESIDENT_CONSTANT_COUNT as usize] {
    let mut constants = [0u32; RESIDENT_CONSTANT_COUNT as usize];
    for (destination, value) in constants[..16].iter_mut().zip(
        scene
            .view_projection(width as f32 / height as f32)
            .to_cols_array(),
    ) {
        *destination = value.to_bits();
    }
    constants[16] = config.active_region_count();
    constants[17] = MAX_VISIBLE_INSTANCES;
    constants
}
