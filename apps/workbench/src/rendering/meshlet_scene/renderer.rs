use std::mem::size_of;

use anyhow::{Result, bail, ensure};
use meshlet_catalog::{ARCHETYPE_COUNT, Catalog, LOD_COUNT};
use serde::Serialize;
use serde_json::{Value, json};
use windows::Win32::Graphics::Direct3D12::*;

use crate::load::{LoadConfig, MAX_VISIBLE_INSTANCES};
use crate::scene::SceneState;

use super::oracle::{self, WorkloadCounts};
use super::pipeline::{MESHLET_CONSTANT_COUNT, MeshletPipeline};
use super::resources::CatalogBuffers;
use crate::rendering::async_resident::PublishedSnapshot;
use crate::rendering::resident::{
    QUERY_COUNT, create_buffer, create_query_heap, read_values, set_viewport, transition,
    uav_barrier,
};

const COUNTER_BYTES: u64 = 64;
const VISIBLE_OBJECT_BYTES: u64 = 16;
pub const PROBE_ITERATIONS: u32 = 16;
pub const MESHLET_REVISION: &str = "gpu-meshlet-scene-v1";

#[derive(Clone, Copy)]
pub struct MeshletSettings {
    pub archetype_mask: u32,
    pub forced_lod: Option<u32>,
}

impl Default for MeshletSettings {
    fn default() -> Self {
        Self {
            archetype_mask: (1 << ARCHETYPE_COUNT) - 1,
            forced_lod: None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshletProbe {
    pub revision: &'static str,
    pub config: LoadConfig,
    pub logical_instance_count: u64,
    pub active_region_count: u32,
    pub candidate_instance_count: u32,
    pub dispatch: [u32; 3],
    pub reset_dispatch_count: u32,
    pub cull_dispatch_count: u32,
    pub indirect_mesh_dispatch_count: u32,
    pub probe_iterations: u32,
    pub archetype_mask: u32,
    pub forced_lod: Option<u32>,
    pub gpu: WorkloadCounts,
    pub cpu_oracle: WorkloadCounts,
    pub catalog_sha256: String,
    pub gpu_cull_lod_ms: f64,
    pub gpu_mesh_ms: f64,
    pub gpu_total_ms: f64,
}

pub struct MeshletSceneRenderer {
    pipeline: MeshletPipeline,
    catalog: Catalog,
    catalog_sha256: String,
    catalog_buffers: CatalogBuffers,
    visible_objects: ID3D12Resource,
    indirect_and_counters: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    counter_readback: ID3D12Resource,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    enabled: bool,
    settings: MeshletSettings,
}

pub struct MeshletFrame<'a> {
    pub snapshot: &'a PublishedSnapshot,
    pub region_heap: &'a ID3D12DescriptorHeap,
    pub scene: &'a SceneState,
    pub render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
    pub depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub probe: bool,
}

impl MeshletSceneRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let catalog = Catalog::build();
        let catalog_sha256 = catalog.sha256();
        let pipeline = unsafe { MeshletPipeline::new(device) }?;
        let catalog_buffers = unsafe { CatalogBuffers::new(device, queue, &catalog) }?;
        let visible_objects = unsafe {
            create_buffer(
                device,
                u64::from(MAX_VISIBLE_INSTANCES) * VISIBLE_OBJECT_BYTES,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let indirect_and_counters = unsafe {
            create_buffer(
                device,
                COUNTER_BYTES,
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
        let counter_readback = unsafe {
            create_buffer(
                device,
                COUNTER_BYTES,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        Ok(Self {
            pipeline,
            catalog,
            catalog_sha256,
            catalog_buffers,
            visible_objects,
            indirect_and_counters,
            query_heap,
            timestamp_readback,
            counter_readback,
            timestamp_frequency,
            width,
            height,
            enabled: false,
            settings: MeshletSettings::default(),
        })
    }

    pub fn configure(&mut self, archetype_mask: u32, forced_lod: Option<u32>) -> Result<()> {
        ensure!(
            archetype_mask != 0 && archetype_mask < (1 << ARCHETYPE_COUNT),
            "archetypeMask must select at least one of eight archetypes"
        );
        ensure!(
            forced_lod.is_none_or(|lod| lod < LOD_COUNT),
            "forcedLod must be null or in the range 0..=2"
        );
        self.settings = MeshletSettings {
            archetype_mask,
            forced_lod,
        };
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
            "revision": MESHLET_REVISION,
            "enabled": self.enabled,
            "settings": {
                "archetypeMask": self.settings.archetype_mask,
                "forcedLod": self.settings.forced_lod,
            },
            "catalog": {
                "sha256": self.catalog_sha256,
                "archetypeCount": ARCHETYPE_COUNT,
                "lodCount": LOD_COUNT,
                "vertexCount": self.catalog.vertices.len(),
                "meshletCount": self.catalog.meshlets.len(),
                "meshletVertexIndexCount": self.catalog.meshlet_vertices.len(),
                "primitiveCount": self.catalog.primitives.len(),
                "gpuBytes": self.catalog_buffers.total_bytes,
            },
            "submission": {
                "resetDispatchCount": 1,
                "cullDispatchCount": 1,
                "indirectMeshDispatchCount": 1,
            }
        })
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        frame: MeshletFrame<'_>,
    ) -> Result<()> {
        let MeshletFrame {
            snapshot,
            region_heap,
            scene,
            render_targets,
            depth_target,
            probe,
        } = frame;
        let constants = self.constants(scene, snapshot);
        let gpu_start = unsafe { region_heap.GetGPUDescriptorHandleForHeapStart() };
        unsafe { command_list.SetDescriptorHeaps(&[Some(region_heap.clone())]) };
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0) };
        }
        let iterations = if probe { PROBE_ITERATIONS } else { 1 };
        unsafe {
            command_list.SetComputeRootSignature(&self.pipeline.compute_root);
            command_list.SetComputeRoot32BitConstants(
                0,
                MESHLET_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootDescriptorTable(1, gpu_start);
            command_list
                .SetComputeRootUnorderedAccessView(2, self.visible_objects.GetGPUVirtualAddress());
            command_list.SetComputeRootUnorderedAccessView(
                3,
                self.indirect_and_counters.GetGPUVirtualAddress(),
            );
            command_list.SetComputeRootShaderResourceView(
                4,
                self.catalog_buffers.lods.GetGPUVirtualAddress(),
            );
            let [groups_x, groups_y, groups_z] = snapshot.config.dispatch();
            for _ in 0..iterations {
                command_list.SetPipelineState(&self.pipeline.reset);
                command_list.Dispatch(1, 1, 1);
                uav_barrier(command_list, &self.indirect_and_counters);
                command_list.SetPipelineState(&self.pipeline.cull);
                command_list.Dispatch(groups_x, groups_y, groups_z);
                uav_barrier(command_list, &self.visible_objects);
                uav_barrier(command_list, &self.indirect_and_counters);
            }
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1) };
        }

        unsafe {
            transition(
                command_list,
                &self.visible_objects,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.indirect_and_counters,
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
                MESHLET_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootDescriptorTable(1, gpu_start);
            for (parameter, resource) in [
                (2, &self.visible_objects),
                (3, &self.catalog_buffers.vertices),
                (4, &self.catalog_buffers.meshlets),
                (5, &self.catalog_buffers.meshlet_vertices),
                (6, &self.catalog_buffers.primitives),
                (7, &self.catalog_buffers.lods),
            ] {
                command_list
                    .SetGraphicsRootShaderResourceView(parameter, resource.GetGPUVirtualAddress());
            }
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 2) };
        }
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
                    &self.indirect_and_counters,
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
                    &self.indirect_and_counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.counter_readback,
                    0,
                    &self.indirect_and_counters,
                    0,
                    COUNTER_BYTES,
                );
                transition(
                    command_list,
                    &self.indirect_and_counters,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        } else {
            unsafe {
                transition(
                    command_list,
                    &self.indirect_and_counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                )
            };
        }
        unsafe {
            transition(
                command_list,
                &self.visible_objects,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            )
        };
        Ok(())
    }

    pub unsafe fn read_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
    ) -> Result<MeshletProbe> {
        let timestamps = unsafe { read_values::<u64>(&self.timestamp_readback, 4) }?;
        let counters = unsafe { read_values::<u32>(&self.counter_readback, 16) }?;
        ensure!(
            counters[1] == 1 && counters[2] == 1,
            "meshlet indirect dimensions are invalid: {:?}",
            &counters[..3]
        );
        ensure!(
            counters[0] == counters[3],
            "meshlet indirect and visible counts diverged"
        );
        let gpu = WorkloadCounts {
            visible: counters[3],
            rejected: counters[4],
            lod_counts: [counters[5], counters[6], counters[7]],
            meshlets: counters[8],
            emitted_vertices: counters[9],
            emitted_triangles: counters[10],
            observed_archetype_mask: counters[11],
        };
        let cpu_oracle = oracle::evaluate(
            &self.catalog,
            self.settings,
            snapshot.config,
            scene,
            self.width,
            self.height,
        )?;
        if gpu != cpu_oracle {
            bail!("meshlet GPU counters {gpu:?} differ from CPU oracle {cpu_oracle:?}");
        }
        let milliseconds = |start: usize, end: usize| {
            timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
                / self.timestamp_frequency as f64
                / PROBE_ITERATIONS as f64
        };
        Ok(MeshletProbe {
            revision: MESHLET_REVISION,
            config: snapshot.config,
            logical_instance_count: snapshot.config.logical_instance_count(),
            active_region_count: snapshot.config.active_region_count(),
            candidate_instance_count: snapshot.config.candidate_instance_count(),
            dispatch: snapshot.config.dispatch(),
            reset_dispatch_count: 1,
            cull_dispatch_count: 1,
            indirect_mesh_dispatch_count: 1,
            probe_iterations: PROBE_ITERATIONS,
            archetype_mask: self.settings.archetype_mask,
            forced_lod: self.settings.forced_lod,
            gpu,
            cpu_oracle,
            catalog_sha256: self.catalog_sha256.clone(),
            gpu_cull_lod_ms: milliseconds(0, 1),
            gpu_mesh_ms: milliseconds(2, 3),
            gpu_total_ms: milliseconds(0, 3),
        })
    }

    fn constants(
        &self,
        scene: &SceneState,
        snapshot: &PublishedSnapshot,
    ) -> [u32; MESHLET_CONSTANT_COUNT as usize] {
        let mut constants = [0u32; MESHLET_CONSTANT_COUNT as usize];
        for (destination, value) in constants[..16].iter_mut().zip(
            scene
                .view_projection(self.width as f32 / self.height as f32)
                .to_cols_array(),
        ) {
            *destination = value.to_bits();
        }
        constants[16] = snapshot.config.active_region_count();
        constants[17] = MAX_VISIBLE_INSTANCES;
        constants[18] = self.settings.archetype_mask;
        constants[19] = self.settings.forced_lod.unwrap_or(u32::MAX);
        constants[20..20 + snapshot.active_slots.len()].copy_from_slice(&snapshot.active_slots);
        constants
    }
}
