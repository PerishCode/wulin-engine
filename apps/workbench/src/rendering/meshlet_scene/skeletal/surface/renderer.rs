mod recording;

use animation_catalog::Catalog as AnimationCatalog;
use anyhow::Result;
use meshlet_catalog::Catalog as MeshletCatalog;
use serde_json::{Value, json};
use surface_catalog::MATERIAL_COUNT;
use windows::Win32::Graphics::Direct3D12::*;

use crate::rendering::resident::{transition, uav_barrier};

use super::super::pipeline::SKELETAL_CONSTANT_COUNT;
use super::super::probe::SkeletalProbe;
use super::occlusion::{self, BoundProof};
use super::pipeline::{SURFACE_CONSTANT_COUNT, SurfacePipeline};
use super::probe::{self, ProbeInput, SurfaceProbe};
use super::resources::{SAMPLE_BYTES, STATS_BYTES, SurfaceResourceInput, SurfaceResources};

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
    settings: SurfaceSettings,
    history_signature: Option<[u32; SKELETAL_CONSTANT_COUNT as usize]>,
    last_history_queried: bool,
    history_reset_count: u64,
    pending_invalidation_reason: &'static str,
    last_bypass_reason: &'static str,
    bound_proof: BoundProof,
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
    pub mesh_catalog: &'a MeshletCatalog,
    pub scene: &'a crate::scene::SceneState,
    pub skeletal_settings: super::super::renderer::SkeletalSettings,
    pub instance_records: &'a [Vec<crate::resident::InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<crate::resident::PresentationRecord>],
    pub projection: crate::rendering::terrain::TerrainProjection,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub background_color: [f32; 4],
    pub timestamp_readback: &'a ID3D12Resource,
    pub timestamp_frequency: u64,
}

pub struct SurfaceRendererInput<'a> {
    pub queue: &'a ID3D12CommandQueue,
    pub source_heap: &'a ID3D12DescriptorHeap,
    pub source_visible: &'a ID3D12Resource,
    pub source_counters: &'a ID3D12Resource,
    pub mesh: &'a MeshletCatalog,
    pub animation: &'a AnimationCatalog,
    pub extent: [u32; 2],
}

impl SurfaceRenderer {
    pub unsafe fn new(device: &ID3D12Device, input: SurfaceRendererInput<'_>) -> Result<Self> {
        let [width, height] = input.extent;
        let bound_proof = occlusion::validate_fixture_bound(input.mesh, input.animation)?;
        Ok(Self {
            pipeline: unsafe { SurfacePipeline::new(device) }?,
            resources: unsafe {
                SurfaceResources::new(
                    device,
                    SurfaceResourceInput {
                        queue: input.queue,
                        source_heap: input.source_heap,
                        source_visible: input.source_visible,
                        source_counters: input.source_counters,
                        mesh: input.mesh,
                        extent: input.extent,
                    },
                )
            }?,
            width,
            height,
            settings: SurfaceSettings::default(),
            history_signature: None,
            last_history_queried: false,
            history_reset_count: 0,
            pending_invalidation_reason: "startup",
            last_bypass_reason: "startup",
            bound_proof,
        })
    }

    pub fn activate_canonical(&mut self) {
        self.invalidate_occlusion_history("canonical-activation");
    }

    fn invalidate_occlusion_history(&mut self, reason: &'static str) {
        self.history_signature = None;
        self.last_history_queried = false;
        self.history_reset_count += 1;
        self.pending_invalidation_reason = reason;
    }

    unsafe fn build_hierarchy(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        mut constants: [u32; SURFACE_CONSTANT_COUNT as usize],
        gpu_start: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            transition(
                command_list,
                &self.resources.visibility_winner,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            self.bind_compute(command_list, constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.hiz_mip0);
            command_list.Dispatch(self.width.div_ceil(8), self.height.div_ceil(8), 1);
            uav_barrier(command_list, &self.resources.occlusion.hierarchy);
            command_list.SetPipelineState(&self.pipeline.hiz_reduce);
            for destination_mip in 1..self.resources.occlusion.mip_count {
                let destination_width = (self.width >> destination_mip).max(1);
                let destination_height = (self.height >> destination_mip).max(1);
                constants[32..36].copy_from_slice(&[
                    destination_mip - 1,
                    destination_mip,
                    destination_width,
                    destination_height,
                ]);
                command_list.SetComputeRoot32BitConstants(
                    0,
                    4,
                    constants[32..36].as_ptr().cast(),
                    32,
                );
                command_list.Dispatch(
                    destination_width.div_ceil(8),
                    destination_height.div_ceil(8),
                    1,
                );
                uav_barrier(command_list, &self.resources.occlusion.hierarchy);
            }
            transition(
                command_list,
                &self.resources.visibility_winner,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
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

    unsafe fn preserve_composed_color(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        back_buffer: &ID3D12Resource,
    ) {
        unsafe {
            transition(
                command_list,
                &self.resources.color,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COPY_DEST,
            );
            transition(
                command_list,
                back_buffer,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            command_list.CopyResource(&self.resources.color, back_buffer);
            transition(
                command_list,
                back_buffer,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
            transition(
                command_list,
                &self.resources.color,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
        }
    }

    unsafe fn publish_resolved_color(
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
                &self.resources.visibility_winner,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            self.resources
                .winner_readback
                .record(command_list, &self.resources.visibility_winner);
            transition(
                command_list,
                &self.resources.visibility_winner,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &self.resources.occlusion.hierarchy,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
            );
            self.resources
                .occlusion
                .hierarchy_readback
                .record(command_list, &self.resources.occlusion.hierarchy);
            transition(
                command_list,
                &self.resources.occlusion.hierarchy,
                D3D12_RESOURCE_STATE_COPY_SOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
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
                mesh_catalog: context.mesh_catalog,
                scene: context.scene,
                skeletal_settings: context.skeletal_settings,
                instance_records: context.instance_records,
                local_ids: context.local_ids,
                presentations: context.presentations,
                projection: context.projection,
                ground_numerators: context.ground_numerators,
                ground_denominator: context.ground_denominator,
                background_color: context.background_color,
                timestamp_readback: context.timestamp_readback,
                timestamp_frequency: context.timestamp_frequency,
                width: self.width,
                height: self.height,
                occlusion_enabled: true,
                history_queried: self.last_history_queried,
                history_reset_count: self.history_reset_count,
                bypass_reason: self.last_bypass_reason,
                bound_proof: self.bound_proof,
            })
        }
    }

    fn constants(
        &self,
        skeletal: [u32; SKELETAL_CONSTANT_COUNT as usize],
        background_color: [f32; 4],
        history_queried: bool,
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
        constants[25] = skeletal[16];
        constants[26] = 65_536;
        constants[28..32].copy_from_slice(&[
            u32::from(history_queried),
            self.resources.occlusion.mip_count,
            self.width,
            self.height,
        ]);
        constants[36..40].copy_from_slice(&[
            super::occlusion::BOUND_RADIAL_SCALE.to_bits(),
            super::occlusion::BOUND_RADIAL_BIAS.to_bits(),
            super::occlusion::BOUND_VERTICAL_PAD.to_bits(),
            super::occlusion::PIXEL_EXPANSION.to_bits(),
        ]);
        constants[40] = super::occlusion::DEPTH_BIAS.to_bits();
        constants[41] = super::occlusion::IMPORTED_BOUND_RADIAL.to_bits();
        constants
    }
}
