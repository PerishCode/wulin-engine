use animation_catalog::{CLIP_COUNT, Catalog as AnimationCatalog, RIG_COUNT};
use anyhow::{Result, ensure};
use meshlet_catalog::Catalog as MeshletCatalog;
use windows::Win32::Graphics::Direct3D12::*;

use crate::scene::SceneState;

use super::pipeline::{SKELETAL_CONSTANT_COUNT, SkeletalPipeline};
use super::resources::{
    AnimationBuffers, COUNTER_BYTES, ExecutionResources, GROUND_BYTES, MAX_SHARED_POSES,
    SAMPLE_BYTES, SKELETAL_CANDIDATE_CAPACITY,
};
use super::surface::{SurfaceFrame, SurfaceRenderer};
use crate::rendering::async_resident::PublishedSnapshot;
use crate::rendering::meshlet_scene::CatalogBuffers;
use crate::rendering::resident::{transition, uav_barrier};
use crate::rendering::terrain::TerrainProjection;

pub const SKELETAL_REVISION: &str = "gpu-skeletal-crowds-v5-dynamic-actor";

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

impl SkeletalSettings {
    pub(super) fn for_tick(time_tick: u32) -> Self {
        Self {
            time_tick,
            ..Self::default()
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
    pub presentation_tick: u32,
    pub actor: Option<crate::rendering::ActorRenderProjection>,
    pub object_target: Option<crate::rendering::ProjectedObjectTarget>,
    pub object_suppression: Option<crate::rendering::ProjectedObjectSuppression>,
    pub frame_slot: u32,
}

impl SkeletalSceneRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        region_heap: &ID3D12DescriptorHeap,
        terrain_heap: &ID3D12DescriptorHeap,
        timestamp_frequency: u64,
        extent: [u32; 2],
        frame_slots: u32,
    ) -> Result<Self> {
        let [width, height] = extent;
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
                [region_heap, terrain_heap],
                &mesh_catalog,
                &animation_catalog,
                &mesh_buffers,
                &animation_buffers,
                frame_slots,
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
        })
    }

    pub unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        frame: SkeletalFrame<'_>,
    ) -> Result<()> {
        let settings = SkeletalSettings::for_tick(frame.presentation_tick);
        let (actor_gpu, _) = unsafe {
            self.resources.actor_upload.write(
                frame.frame_slot,
                frame.actor,
                frame.presentation_tick,
            )
        }?;
        let constants = self.constants(settings, &frame)?;
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
            command_list.SetComputeRootShaderResourceView(2, actor_gpu);
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
            command_list.Dispatch(x + 1, y, z);
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
                    object_target: frame.object_target,
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
        settings: SkeletalSettings,
        frame: &SkeletalFrame<'_>,
    ) -> Result<[u32; SKELETAL_CONSTANT_COUNT as usize]> {
        let mut constants = [0u32; SKELETAL_CONSTANT_COUNT as usize];
        let camera = frame.projection.camera(frame.scene.camera());
        for (destination, value) in constants[..16].iter_mut().zip(
            crate::scene::view_projection(camera, self.width as f32 / self.height as f32)
                .to_cols_array(),
        ) {
            *destination = value.to_bits();
        }
        constants[16] = frame.snapshot.config.active_region_count();
        constants[17] = SKELETAL_CANDIDATE_CAPACITY;
        constants[18] = (1 << CLIP_COUNT) - 1;
        constants[19] = u32::MAX;
        for (index, instance_slot) in frame.snapshot.active_slots.iter().copied().enumerate() {
            let terrain_slot = frame.terrain_slots.map_or(0, |slots| slots[index]);
            let semantic_region = frame.projection.region_id(index)?;
            ensure!(
                semantic_region < 1 << 14,
                "object semantic region exceeds the packed mapping"
            );
            constants[20 + index] = instance_slot | (terrain_slot << 6) | (semantic_region << 12);
        }
        constants[48] = RIG_COUNT;
        constants[49] = settings.bone_count;
        constants[50] = settings.phase_count;
        constants[51] = settings.time_tick;
        constants[52] = u32::from(settings.unique_poses);
        constants[53] = SKELETAL_CANDIDATE_CAPACITY;
        constants[54] = MAX_SHARED_POSES;
        constants[55] = frame.grounding_mode;
        constants[56..59].copy_from_slice(&animation_catalog::IMPORTED_SOURCE_CLIP_DURATION_UNITS);
        constants[59] = crate::rendering::pack_object_suppression(frame.object_suppression)?;
        Ok(constants)
    }
}
