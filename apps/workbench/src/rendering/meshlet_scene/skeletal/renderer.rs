use animation_catalog::{BONE_COUNT, CLIP_COUNT, Catalog as AnimationCatalog};
use anyhow::{Result, ensure};
use meshlet_catalog::{Catalog as MeshletCatalog, LOD_COUNT};
use serde_json::{Value, json};
use windows::Win32::Graphics::Direct3D12::*;

use crate::scene::SceneState;

use super::pipeline::{SKELETAL_CONSTANT_COUNT, SkeletalPipeline};
use super::probe::{self, ProbeInput, SkeletalProbe};
use super::resources::{
    AnimationBuffers, COUNTER_BYTES, ExecutionResources, MAX_SHARED_POSES, MAX_SKELETAL_VISIBLE,
    PALETTE_BYTES, QUERY_COUNT, SAMPLE_BYTES,
};
use super::surface::{SurfaceFrame, SurfaceRenderer};
use crate::rendering::async_resident::PublishedSnapshot;
use crate::rendering::meshlet_scene::CatalogBuffers;
use crate::rendering::resident::{set_viewport, transition, uav_barrier};

pub const SKELETAL_REVISION: &str = "gpu-skeletal-crowds-v1";

#[derive(Clone, Copy)]
pub struct SkeletalSettings {
    pub animated_percent: u32,
    pub bone_count: u32,
    pub phase_count: u32,
    pub time_tick: u32,
    pub unique_poses: bool,
    pub forced_lod: Option<u32>,
}

impl Default for SkeletalSettings {
    fn default() -> Self {
        Self {
            animated_percent: 100,
            bone_count: 64,
            phase_count: 64,
            time_tick: 0,
            unique_poses: false,
            forced_lod: None,
        }
    }
}

pub struct SkeletalSceneRenderer {
    pipeline: SkeletalPipeline,
    mesh_catalog: MeshletCatalog,
    pub(super) animation_catalog: AnimationCatalog,
    mesh_catalog_sha256: String,
    animation_catalog_sha256: String,
    mesh_buffers: CatalogBuffers,
    animation_buffers: AnimationBuffers,
    pub(super) resources: ExecutionResources,
    pub(super) surface: SurfaceRenderer,
    pub(super) timestamp_frequency: u64,
    width: u32,
    height: u32,
    pub(super) enabled: bool,
    pub(super) settings: SkeletalSettings,
}

#[derive(Clone, Copy)]
pub struct SkeletalFrame<'a> {
    pub snapshot: &'a PublishedSnapshot,
    pub scene: &'a SceneState,
    pub back_buffer: &'a ID3D12Resource,
    pub render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
    pub depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub background_color: [f32; 4],
    pub probe: bool,
}

impl SkeletalSceneRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        region_heap: &ID3D12DescriptorHeap,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let mesh_catalog = MeshletCatalog::build();
        let animation_catalog = AnimationCatalog::build();
        let mesh_catalog_sha256 = mesh_catalog.sha256();
        let animation_catalog_sha256 = animation_catalog.sha256();
        let pipeline = unsafe { SkeletalPipeline::new(device) }?;
        let mesh_buffers = unsafe { CatalogBuffers::new(device, queue, &mesh_catalog) }?;
        let animation_buffers =
            unsafe { AnimationBuffers::new(device, queue, &animation_catalog) }?;
        let resources = unsafe {
            ExecutionResources::new(
                device,
                region_heap,
                &mesh_catalog,
                &animation_catalog,
                &mesh_buffers,
                &animation_buffers,
            )
        }?;
        let surface = unsafe {
            SurfaceRenderer::new(device, queue, &resources.heap, &mesh_catalog, width, height)
        }?;
        Ok(Self {
            pipeline,
            mesh_catalog,
            animation_catalog,
            mesh_catalog_sha256,
            animation_catalog_sha256,
            mesh_buffers,
            animation_buffers,
            resources,
            surface,
            timestamp_frequency,
            width,
            height,
            enabled: false,
            settings: SkeletalSettings::default(),
        })
    }

    pub fn configure(&mut self, settings: SkeletalSettings) -> Result<()> {
        ensure!(
            matches!(settings.animated_percent, 0 | 25 | 50 | 100),
            "animatedPercent must be one of 0, 25, 50, or 100"
        );
        ensure!(
            matches!(settings.bone_count, 16 | 32 | 64 | 128),
            "boneCount must be one of 16, 32, 64, or 128"
        );
        ensure!(
            matches!(settings.phase_count, 1 | 8 | 64),
            "phaseCount must be one of 1, 8, or 64"
        );
        ensure!(
            settings.forced_lod.is_none_or(|lod| lod < LOD_COUNT),
            "forcedLod must be null or in the range 0..=2"
        );
        self.settings = settings;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.surface.disable();
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.surface.disable();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn status_json(&self) -> Value {
        json!({
            "revision": SKELETAL_REVISION,
            "enabled": self.enabled,
            "settings": self.settings_json(),
            "catalog": {
                "meshletSha256": self.mesh_catalog_sha256,
                "animationSha256": self.animation_catalog_sha256,
                "boneCount": BONE_COUNT,
                "clipCount": CLIP_COUNT,
                "sampleCountPerClip": animation_catalog::SAMPLE_COUNT,
                "skinBindingCount": self.animation_catalog.skin_bindings.len(),
                "meshletGpuBytes": self.mesh_buffers.total_bytes,
                "animationGpuBytes": self.animation_buffers.total_bytes,
            },
            "resources": {
                "visibleCapacity": MAX_SKELETAL_VISIBLE,
                "sharedPoseCapacity": MAX_SHARED_POSES,
                "uniquePoseCapacity": MAX_SKELETAL_VISIBLE,
                "paletteBytes": PALETTE_BYTES,
                "executionBytes": self.resources.execution_bytes,
            },
            "submission": {
                "resetDispatchCount": 1,
                "cullDispatchCount": 1,
                "poseCompactDispatchCount": 1,
                "indirectPoseDispatchCount": 1,
                "indirectMeshDispatchCount": 1,
            }
        })
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        frame: SkeletalFrame<'_>,
    ) -> Result<()> {
        let constants = self.constants(frame.scene, frame.snapshot);
        let gpu_start = unsafe { self.resources.heap.GetGPUDescriptorHandleForHeapStart() };
        unsafe {
            command_list.SetDescriptorHeaps(&[Some(self.resources.heap.clone())]);
            command_list.SetComputeRootSignature(&self.pipeline.root);
            command_list.SetComputeRoot32BitConstants(
                0,
                SKELETAL_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootDescriptorTable(1, gpu_start);
        }
        if frame.probe {
            unsafe {
                command_list.EndQuery(&self.resources.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0)
            };
        }
        unsafe {
            command_list.SetPipelineState(&self.pipeline.reset);
            command_list.Dispatch(1, 1, 1);
            for resource in [
                &self.resources.counters,
                &self.resources.pose_bitset,
                &self.resources.sample,
            ] {
                uav_barrier(command_list, resource);
            }
            command_list.SetPipelineState(&self.pipeline.cull);
            let [x, y, z] = frame.snapshot.config.dispatch();
            command_list.Dispatch(x, y, z);
            for resource in [
                &self.resources.visible,
                &self.resources.counters,
                &self.resources.animated_indices,
                &self.resources.pose_bitset,
            ] {
                uav_barrier(command_list, resource);
            }
        }
        if frame.probe {
            unsafe {
                command_list.EndQuery(&self.resources.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1)
            };
        }
        unsafe {
            command_list.SetPipelineState(&self.pipeline.compact);
            command_list.Dispatch(1, 1, 1);
            uav_barrier(command_list, &self.resources.counters);
            uav_barrier(command_list, &self.resources.active_pose_keys);
        }
        if frame.probe {
            unsafe {
                command_list.EndQuery(&self.resources.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 2)
            };
        }
        unsafe {
            transition(
                command_list,
                &self.resources.counters,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            );
            command_list.SetPipelineState(&self.pipeline.pose);
            command_list.ExecuteIndirect(
                &self.pipeline.dispatch_signature,
                1,
                &self.resources.counters,
                56,
                None,
                0,
            );
            uav_barrier(command_list, &self.resources.palette);
            uav_barrier(command_list, &self.resources.sample);
        }
        if frame.probe {
            unsafe {
                command_list.EndQuery(&self.resources.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 3)
            };
        }
        if self.surface.is_enabled() {
            unsafe {
                self.surface.record(
                    command_list,
                    &self.resources,
                    constants,
                    SurfaceFrame {
                        back_buffer: frame.back_buffer,
                        object_id_target: frame.render_targets[1],
                        depth_target: frame.depth_target,
                        background_color: frame.background_color,
                        probe: frame.probe,
                    },
                )
            };
        } else {
            unsafe { self.record_mesh(command_list, frame, constants, gpu_start) };
            if frame.probe {
                unsafe {
                    command_list.EndQuery(
                        &self.resources.query_heap,
                        D3D12_QUERY_TYPE_TIMESTAMP,
                        4,
                    );
                    command_list.ResolveQueryData(
                        &self.resources.query_heap,
                        D3D12_QUERY_TYPE_TIMESTAMP,
                        0,
                        QUERY_COUNT - 1,
                        &self.resources.timestamp_readback,
                        0,
                    );
                }
            }
        }
        if frame.probe {
            unsafe {
                transition(
                    command_list,
                    &self.resources.counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                transition(
                    command_list,
                    &self.resources.sample,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.resources.counter_readback,
                    0,
                    &self.resources.counters,
                    0,
                    COUNTER_BYTES,
                );
                command_list.CopyBufferRegion(
                    &self.resources.sample_readback,
                    0,
                    &self.resources.sample,
                    0,
                    SAMPLE_BYTES,
                );
                transition(
                    command_list,
                    &self.resources.counters,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
                transition(
                    command_list,
                    &self.resources.sample,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        } else {
            unsafe {
                transition(
                    command_list,
                    &self.resources.counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        }
        Ok(())
    }

    unsafe fn record_mesh(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        frame: SkeletalFrame<'_>,
        constants: [u32; SKELETAL_CONSTANT_COUNT as usize],
        gpu_start: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            transition(
                command_list,
                &self.resources.visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.resources.palette,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
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
            command_list.SetPipelineState(&self.pipeline.graphics);
            command_list.SetGraphicsRoot32BitConstants(
                0,
                SKELETAL_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootDescriptorTable(1, gpu_start);
            command_list.ExecuteIndirect(
                &self.pipeline.mesh_signature,
                1,
                &self.resources.counters,
                0,
                None,
                0,
            );
            transition(
                command_list,
                &self.resources.visible,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &self.resources.palette,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
        }
    }

    pub unsafe fn read_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
    ) -> Result<SkeletalProbe> {
        unsafe {
            probe::read(ProbeInput {
                resources: &self.resources,
                mesh_catalog: &self.mesh_catalog,
                animation_catalog: &self.animation_catalog,
                mesh_catalog_sha256: &self.mesh_catalog_sha256,
                animation_catalog_sha256: &self.animation_catalog_sha256,
                settings: self.settings,
                settings_json: self.settings_json(),
                timestamp_frequency: self.timestamp_frequency,
                width: self.width,
                height: self.height,
                snapshot,
                scene,
            })
        }
    }

    fn settings_json(&self) -> Value {
        json!({
            "animatedPercent": self.settings.animated_percent,
            "boneCount": self.settings.bone_count,
            "phaseCount": self.settings.phase_count,
            "timeTick": self.settings.time_tick,
            "uniquePoses": self.settings.unique_poses,
            "forcedLod": self.settings.forced_lod,
        })
    }

    fn constants(
        &self,
        scene: &SceneState,
        snapshot: &PublishedSnapshot,
    ) -> [u32; SKELETAL_CONSTANT_COUNT as usize] {
        let mut constants = [0u32; SKELETAL_CONSTANT_COUNT as usize];
        for (destination, value) in constants[..16].iter_mut().zip(
            scene
                .view_projection(self.width as f32 / self.height as f32)
                .to_cols_array(),
        ) {
            *destination = value.to_bits();
        }
        constants[16] = snapshot.config.active_region_count();
        constants[17] = MAX_SKELETAL_VISIBLE;
        constants[18] = (1 << CLIP_COUNT) - 1;
        constants[19] = self.settings.forced_lod.unwrap_or(u32::MAX);
        constants[20..20 + snapshot.active_slots.len()].copy_from_slice(&snapshot.active_slots);
        constants[48] = self.settings.animated_percent;
        constants[49] = self.settings.bone_count;
        constants[50] = self.settings.phase_count;
        constants[51] = self.settings.time_tick;
        constants[52] = u32::from(self.settings.unique_poses);
        constants[53] = MAX_SKELETAL_VISIBLE;
        constants[54] = MAX_SHARED_POSES;
        constants[55] = 7;
        constants
    }
}
