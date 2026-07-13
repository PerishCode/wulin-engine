use animation_catalog::Catalog as AnimationCatalog;
use anyhow::{Result, ensure};
use meshlet_catalog::Catalog as MeshletCatalog;
use serde_json::{Value, json};
use surface_catalog::{MATERIAL_COUNT, MIP_COUNT};
use windows::Win32::Graphics::Direct3D12::*;

use crate::load::LoadConfig;
use crate::rendering::resident::{set_viewport, transition, uav_barrier};

use super::super::pipeline::SKELETAL_CONSTANT_COUNT;
use super::super::probe::SkeletalProbe;
use super::super::resources::{ExecutionResources, QUERY_COUNT};
use super::pipeline::{SURFACE_CONSTANT_COUNT, SurfacePipeline};
use super::probe::{self, ProbeInput, SurfaceProbe};
use super::resources::{SAMPLE_BYTES, STATS_BYTES, SurfaceResources};

pub const SURFACE_REVISION: &str = "gpu-surface-resolve-v1";

#[derive(Clone, Copy)]
pub struct SurfaceSettings {
    pub material_count: u32,
    pub mip_level: u32,
}

impl Default for SurfaceSettings {
    fn default() -> Self {
        Self {
            material_count: MATERIAL_COUNT,
            mip_level: 0,
        }
    }
}

pub struct SurfaceRenderer {
    pipeline: SurfacePipeline,
    pub resources: SurfaceResources,
    width: u32,
    height: u32,
    enabled: bool,
    settings: SurfaceSettings,
}

pub struct SurfaceFrame<'a> {
    pub back_buffer: &'a ID3D12Resource,
    pub object_id_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub background_color: [f32; 4],
    pub probe: bool,
}

pub struct SurfaceProbeContext<'a> {
    pub skeletal: SkeletalProbe,
    pub animation_catalog: &'a AnimationCatalog,
    pub skeletal_settings: super::super::renderer::SkeletalSettings,
    pub config: LoadConfig,
    pub background_color: [f32; 4],
    pub timestamp_readback: &'a ID3D12Resource,
    pub timestamp_frequency: u64,
}

impl SurfaceRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        source_heap: &ID3D12DescriptorHeap,
        mesh: &MeshletCatalog,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        Ok(Self {
            pipeline: unsafe { SurfacePipeline::new(device) }?,
            resources: unsafe {
                SurfaceResources::new(device, queue, source_heap, mesh, width, height)
            }?,
            width,
            height,
            enabled: false,
            settings: SurfaceSettings::default(),
        })
    }

    pub fn configure(&mut self, settings: SurfaceSettings) -> Result<()> {
        ensure!(
            matches!(settings.material_count, 1 | 8 | 64),
            "materialCount must be one of 1, 8, or 64"
        );
        ensure!(
            matches!(settings.mip_level, 0 | 3 | 6) && settings.mip_level < MIP_COUNT,
            "mipLevel must be one of 0, 3, or 6"
        );
        self.settings = settings;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn status_json(&self) -> Value {
        json!({
            "revision": SURFACE_REVISION,
            "enabled": self.enabled,
            "settings": self.settings_json(),
            "catalog": {
                "sha256": self.resources.catalog_sha256,
                "vertexCount": self.resources.catalog.vertices.len(),
                "primitiveCount": self.resources.catalog.primitives.len(),
                "materialCount": self.resources.catalog.materials.len(),
                "textureMipCount": self.resources.catalog.texture_mips.len(),
                "gpuBytes": self.resources.uploaded.total_bytes,
            },
            "resources": {
                "width": self.width,
                "height": self.height,
                "visibilityFormat": "R32G32_UINT",
                "colorFormat": "R8G8B8A8_UNORM",
                "executionBytes": self.resources.execution_bytes,
            },
            "submission": {
                "indirectVisibilityDispatchCount": 1,
                "resolveDispatchCount": 1,
                "resolveGroups": [self.width.div_ceil(8), self.height.div_ceil(8), 1],
            }
        })
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        execution: &ExecutionResources,
        skeletal_constants: [u32; SKELETAL_CONSTANT_COUNT as usize],
        frame: SurfaceFrame<'_>,
    ) {
        let surface_constants = self.constants(skeletal_constants, frame.background_color);
        let gpu_start = unsafe { self.resources.gpu_start() };
        let visibility_target = unsafe { self.resources.visibility_handle() };
        unsafe {
            transition(
                command_list,
                &execution.visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &execution.palette,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            command_list.SetDescriptorHeaps(&[Some(self.resources.heap.clone())]);
            let (winner_gpu, winner_cpu) = self.resources.winner_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                winner_gpu,
                winner_cpu,
                &self.resources.visibility_winner,
                &[0; 4],
                &[],
            );
            uav_barrier(command_list, &self.resources.visibility_winner);
            let render_targets = [visibility_target, frame.object_id_target];
            command_list.OMSetRenderTargets(
                2,
                Some(render_targets.as_ptr()),
                false,
                Some(&frame.depth_target),
            );
            command_list.ClearRenderTargetView(visibility_target, &[0.0; 4], None);
            command_list.ClearRenderTargetView(frame.object_id_target, &[0.0; 4], None);
            command_list.ClearDepthStencilView(
                frame.depth_target,
                D3D12_CLEAR_FLAG_DEPTH,
                0.0,
                0,
                None,
            );
            set_viewport(command_list, self.width, self.height);
            self.bind_graphics(command_list, surface_constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.visibility);
            command_list.ExecuteIndirect(
                &self.pipeline.mesh_signature,
                1,
                &execution.counters,
                0,
                None,
                0,
            );
            uav_barrier(command_list, &self.resources.candidate_to_visible);
            uav_barrier(command_list, &self.resources.visibility_winner);
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 4);
            }

            transition(
                command_list,
                &self.resources.visibility,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.resources.candidate_to_visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            let (stats_gpu, stats_cpu) = self.resources.stats_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                stats_gpu,
                stats_cpu,
                &self.resources.stats,
                &[0; 4],
                &[],
            );
            self.bind_compute(command_list, surface_constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.shade);
            command_list.Dispatch(self.width.div_ceil(8), self.height.div_ceil(8), 1);
            for resource in [
                &self.resources.color,
                &self.resources.stats,
                &self.resources.samples,
            ] {
                uav_barrier(command_list, resource);
            }
            self.copy_resolved_color(command_list, frame.back_buffer);
            if frame.probe {
                self.record_probe_copies(command_list);
            } else {
                transition(
                    command_list,
                    &self.resources.visibility,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                );
            }
            transition(
                command_list,
                &self.resources.candidate_to_visible,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &execution.visible,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &execution.palette,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 5);
                command_list.ResolveQueryData(
                    &execution.query_heap,
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    0,
                    QUERY_COUNT,
                    &execution.timestamp_readback,
                    0,
                );
            }
        }
    }

    unsafe fn bind_graphics(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        surface: [u32; SURFACE_CONSTANT_COUNT as usize],
        gpu_start: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            command_list.SetGraphicsRootSignature(&self.pipeline.root);
            command_list.SetGraphicsRoot32BitConstants(
                0,
                SURFACE_CONSTANT_COUNT,
                surface.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootDescriptorTable(1, gpu_start);
        }
    }

    unsafe fn bind_compute(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        surface: [u32; SURFACE_CONSTANT_COUNT as usize],
        gpu_start: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            command_list.SetComputeRootSignature(&self.pipeline.root);
            command_list.SetComputeRoot32BitConstants(
                0,
                SURFACE_CONSTANT_COUNT,
                surface.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootDescriptorTable(1, gpu_start);
        }
    }

    unsafe fn copy_resolved_color(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        back_buffer: &ID3D12Resource,
    ) {
        unsafe {
            transition(
                command_list,
                &self.resources.color,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            transition(
                command_list,
                back_buffer,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COPY_DEST,
            );
            command_list.CopyResource(back_buffer, &self.resources.color);
            transition(
                command_list,
                back_buffer,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
            transition(
                command_list,
                &self.resources.color,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
        }
    }

    unsafe fn record_probe_copies(&self, command_list: &ID3D12GraphicsCommandList) {
        unsafe {
            transition(
                command_list,
                &self.resources.visibility,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            self.resources
                .visibility_readback
                .record(command_list, &self.resources.visibility);
            transition(
                command_list,
                &self.resources.visibility,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
            for (source, destination, bytes) in [
                (
                    &self.resources.stats,
                    &self.resources.stats_readback,
                    STATS_BYTES,
                ),
                (
                    &self.resources.samples,
                    &self.resources.sample_readback,
                    SAMPLE_BYTES,
                ),
            ] {
                transition(
                    command_list,
                    source,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(destination, 0, source, 0, bytes);
                transition(
                    command_list,
                    source,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        }
    }

    pub fn settings_json(&self) -> Value {
        json!({
            "materialCount": self.settings.material_count,
            "mipLevel": self.settings.mip_level,
        })
    }

    pub unsafe fn read_probe(&self, context: SurfaceProbeContext<'_>) -> Result<SurfaceProbe> {
        unsafe {
            probe::read(ProbeInput {
                resources: &self.resources,
                settings: self.settings,
                settings_json: self.settings_json(),
                skeletal: context.skeletal,
                animation_catalog: context.animation_catalog,
                skeletal_settings: context.skeletal_settings,
                config: context.config,
                background_color: context.background_color,
                timestamp_readback: context.timestamp_readback,
                timestamp_frequency: context.timestamp_frequency,
                width: self.width,
                height: self.height,
            })
        }
    }

    fn constants(
        &self,
        skeletal: [u32; SKELETAL_CONSTANT_COUNT as usize],
        background_color: [f32; 4],
    ) -> [u32; SURFACE_CONSTANT_COUNT as usize] {
        let mut constants = [0; SURFACE_CONSTANT_COUNT as usize];
        constants[..16].copy_from_slice(&skeletal[..16]);
        constants[16..20].copy_from_slice(&[
            self.settings.material_count,
            self.settings.mip_level,
            self.width,
            self.height,
        ]);
        for (destination, channel) in constants[20..24].iter_mut().zip(background_color) {
            *destination = channel.to_bits();
        }
        constants[24] = skeletal[49];
        constants
    }
}
