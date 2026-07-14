use animation_catalog::{CLIP_COUNT, Catalog as AnimationCatalog, RIG_COUNT};
use anyhow::{Result, ensure};
use meshlet_catalog::Catalog as MeshletCatalog;
use windows::Win32::Graphics::Direct3D12::*;

use crate::scene::SceneState;

use super::pipeline::{SKELETAL_CONSTANT_COUNT, SkeletalPipeline};
use super::resources::{
    AnimationBuffers, COUNTER_BYTES, ExecutionResources, GROUND_BYTES, MAX_SHARED_POSES,
    MAX_SKELETAL_VISIBLE, SAMPLE_BYTES,
};
use super::surface::{SurfaceFrame, SurfaceRenderer};
use crate::rendering::async_resident::PublishedSnapshot;
use crate::rendering::meshlet_scene::CatalogBuffers;
use crate::rendering::resident::{transition, uav_barrier};
use crate::rendering::terrain::TerrainProjection;

pub const SKELETAL_REVISION: &str = "gpu-skeletal-crowds-v3-source-duration";

#[derive(Clone, Copy)]
pub struct SkeletalSettings {
    pub bone_count: u32,
    pub phase_count: u32,
    pub time_tick: u32,
    pub unique_poses: bool,
}

impl Default for SkeletalSettings {
    fn default() -> Self {
        Self {
            bone_count: 64,
            phase_count: 64,
            time_tick: 0,
            unique_poses: false,
        }
    }
}

pub struct SkeletalSceneRenderer {
    pipeline: SkeletalPipeline,
    pub(super) mesh_catalog: MeshletCatalog,
    pub(super) animation_catalog: AnimationCatalog,
    pub(super) mesh_catalog_sha256: String,
    pub(super) animation_catalog_sha256: String,
    pub(super) _mesh_buffers: CatalogBuffers,
    pub(super) _animation_buffers: AnimationBuffers,
    pub(super) resources: ExecutionResources,
    pub(super) surface: SurfaceRenderer,
    pub(super) timestamp_frequency: u64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) settings: SkeletalSettings,
    pub(super) time_running: bool,
    pub(super) automatic_time_advance_count: u64,
    pub(super) manual_time_step_count: u64,
    pub(super) time_wrap_count: u64,
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
    pub terrain_slots: Option<&'a [u32]>,
    pub grounding_mode: u32,
    pub projection: TerrainProjection,
}

impl SkeletalSceneRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        region_heap: &ID3D12DescriptorHeap,
        terrain_heap: &ID3D12DescriptorHeap,
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
                terrain_heap,
                &mesh_catalog,
                &animation_catalog,
                &mesh_buffers,
                &animation_buffers,
            )
        }?;
        let surface = unsafe {
            super::surface_bridge::create_surface(
                device,
                queue,
                &resources,
                &mesh_catalog,
                &animation_catalog,
                [width, height],
            )
        }?;
        Ok(Self {
            pipeline,
            mesh_catalog,
            animation_catalog,
            mesh_catalog_sha256,
            animation_catalog_sha256,
            _mesh_buffers: mesh_buffers,
            _animation_buffers: animation_buffers,
            resources,
            surface,
            timestamp_frequency,
            width,
            height,
            settings: SkeletalSettings::default(),
            time_running: true,
            automatic_time_advance_count: 0,
            manual_time_step_count: 0,
            time_wrap_count: 0,
        })
    }

    pub(in crate::rendering) fn pause_presentation_time(&mut self) {
        self.time_running = false;
    }

    pub(in crate::rendering) fn resume_presentation_time(&mut self) {
        self.time_running = true;
    }

    pub(in crate::rendering) fn set_presentation_time(&mut self, tick: u32) -> Result<()> {
        ensure!(
            !self.time_running,
            "presentation clock must be paused before setting time"
        );
        ensure!(
            tick < animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD,
            "presentation tick must be below {}",
            animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD
        );
        self.settings.time_tick = tick;
        Ok(())
    }

    pub(in crate::rendering) fn step_presentation_time(&mut self, ticks: u32) -> Result<()> {
        ensure!(
            !self.time_running,
            "presentation clock must be paused before stepping"
        );
        ensure!(
            (1..=4_096).contains(&ticks),
            "presentation step must contain 1..=4096 ticks"
        );
        self.advance_presentation_ticks(ticks);
        self.manual_time_step_count = self.manual_time_step_count.wrapping_add(u64::from(ticks));
        Ok(())
    }

    pub(in crate::rendering) fn advance_presentation_frame(&mut self) {
        if self.time_running {
            self.advance_presentation_ticks(1);
            self.automatic_time_advance_count = self.automatic_time_advance_count.wrapping_add(1);
        }
    }

    fn advance_presentation_ticks(&mut self, ticks: u32) {
        let total = u64::from(self.settings.time_tick) + u64::from(ticks);
        let period = u64::from(animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD);
        self.time_wrap_count = self.time_wrap_count.wrapping_add(total / period);
        self.settings.time_tick = (total % period) as u32;
    }

    pub unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        frame: SkeletalFrame<'_>,
    ) -> Result<()> {
        let constants = self.constants(
            frame.scene,
            frame.snapshot,
            frame.terrain_slots,
            frame.grounding_mode,
            frame.projection,
        )?;
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
                &self.resources.ground,
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
                if frame.terrain_slots.is_some() {
                    transition(
                        command_list,
                        &self.resources.ground,
                        D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                    );
                    command_list.CopyBufferRegion(
                        &self.resources.ground_readback,
                        0,
                        &self.resources.ground,
                        0,
                        GROUND_BYTES,
                    );
                    transition(
                        command_list,
                        &self.resources.ground,
                        D3D12_RESOURCE_STATE_COPY_SOURCE,
                        D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    );
                }
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

    fn constants(
        &self,
        scene: &SceneState,
        snapshot: &PublishedSnapshot,
        terrain_slots: Option<&[u32]>,
        grounding_mode: u32,
        projection: TerrainProjection,
    ) -> Result<[u32; SKELETAL_CONSTANT_COUNT as usize]> {
        let mut constants = [0u32; SKELETAL_CONSTANT_COUNT as usize];
        let camera = projection.camera(scene.camera());
        for (destination, value) in constants[..16].iter_mut().zip(
            crate::scene::view_projection(camera, self.width as f32 / self.height as f32)
                .to_cols_array(),
        ) {
            *destination = value.to_bits();
        }
        constants[16] = snapshot.config.active_region_count();
        constants[17] = MAX_SKELETAL_VISIBLE;
        constants[18] = (1 << CLIP_COUNT) - 1;
        constants[19] = u32::MAX;
        for (index, instance_slot) in snapshot.active_slots.iter().copied().enumerate() {
            let terrain_slot = terrain_slots.map_or(0, |slots| slots[index]);
            let semantic_region = projection.region_id(index)?;
            ensure!(
                semantic_region < 1 << 14,
                "object semantic region exceeds the packed mapping"
            );
            constants[20 + index] = instance_slot | (terrain_slot << 6) | (semantic_region << 12);
        }
        constants[48] = RIG_COUNT;
        constants[49] = self.settings.bone_count;
        constants[50] = self.settings.phase_count;
        constants[51] = self.settings.time_tick;
        constants[52] = u32::from(self.settings.unique_poses);
        constants[53] = MAX_SKELETAL_VISIBLE;
        constants[54] = MAX_SHARED_POSES;
        constants[55] = grounding_mode;
        constants[56..59].copy_from_slice(&animation_catalog::IMPORTED_SOURCE_CLIP_DURATION_UNITS);
        Ok(constants)
    }
}
