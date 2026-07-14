use std::mem::size_of;

use animation_catalog::{CLIP_COUNT, Catalog as AnimationCatalog};
use anyhow::{Result, bail, ensure};
use meshlet_catalog::{Catalog as MeshletCatalog, IMPORTED_ARCHETYPE, LOD_COUNT};
use serde::Serialize;
use serde_json::Value;

use crate::load::LoadConfig;
use crate::rendering::async_resident::PublishedSnapshot;
use crate::rendering::resident::read_values;
use crate::scene::SceneState;

use super::oracle::{self, WorkloadCounts};
use super::renderer::{SKELETAL_REVISION, SkeletalSettings};
use super::resources::ExecutionResources;

const PALETTE_TOLERANCE: f32 = 0.00002;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSample {
    pub clip: u32,
    pub phase: u32,
    pub bone_count: u32,
    pub variant: u32,
    pub pose_slot: u32,
    pub sampled_bones: u32,
    pub maximum_absolute_delta: f32,
    pub tolerance: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedGeometryProbe {
    pub revision: &'static str,
    pub archetype: u32,
    pub source_json_sha256: String,
    pub source_bin_sha256: String,
    pub source_texture_sha256: String,
    pub cooked_sha256: String,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub lod_index_counts: [u32; 3],
    pub lod_triangle_counts: [u32; 3],
    pub lod_meshlet_counts: [u32; 3],
    pub lod_emitted_vertex_counts: [u32; 3],
    pub lod_errors: [f32; 3],
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkeletalProbe {
    pub revision: &'static str,
    pub config: LoadConfig,
    pub logical_instance_count: u64,
    pub candidate_instance_count: u32,
    pub settings: Value,
    pub gpu: WorkloadCounts,
    pub cpu_oracle: WorkloadCounts,
    pub pose_dispatch_groups: u32,
    pub reset_dispatch_count: u32,
    pub cull_dispatch_count: u32,
    pub pose_compact_dispatch_count: u32,
    pub indirect_pose_dispatch_count: u32,
    pub indirect_mesh_dispatch_count: u32,
    pub palette_write_bytes: u64,
    pub palette_sample: Option<PaletteSample>,
    pub imported_geometry: ImportedGeometryProbe,
    pub meshlet_catalog_sha256: String,
    pub animation_catalog_sha256: String,
    pub gpu_cull_classify_ms: f64,
    pub gpu_pose_compact_ms: f64,
    pub gpu_pose_evaluate_ms: f64,
    pub gpu_mesh_skin_ms: f64,
    pub gpu_total_ms: f64,
}

impl SkeletalProbe {
    pub(in crate::rendering) fn visible_count(&self) -> u32 {
        self.gpu.visible
    }

    pub(in crate::rendering) fn gpu_timing(&self) -> [f64; 5] {
        [
            self.gpu_cull_classify_ms,
            self.gpu_pose_compact_ms,
            self.gpu_pose_evaluate_ms,
            self.gpu_mesh_skin_ms,
            self.gpu_total_ms,
        ]
    }
}

pub struct ProbeInput<'a> {
    pub resources: &'a ExecutionResources,
    pub mesh_catalog: &'a MeshletCatalog,
    pub animation_catalog: &'a AnimationCatalog,
    pub mesh_catalog_sha256: &'a str,
    pub animation_catalog_sha256: &'a str,
    pub settings: SkeletalSettings,
    pub settings_json: Value,
    pub timestamp_frequency: u64,
    pub width: u32,
    pub height: u32,
    pub snapshot: &'a PublishedSnapshot,
    pub scene: &'a SceneState,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub instance_records: &'a [Vec<crate::resident::InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<crate::resident::PresentationRecord>],
}

pub unsafe fn read(input: ProbeInput<'_>) -> Result<SkeletalProbe> {
    let timestamps = unsafe { read_values::<u64>(&input.resources.timestamp_readback, 5) }?;
    let counters = unsafe { read_values::<u32>(&input.resources.counter_readback, 20) }?;
    ensure!(
        counters[1] == 1 && counters[2] == 1,
        "skeletal mesh arguments are invalid"
    );
    ensure!(
        counters[15] == 1 && counters[16] == 1,
        "skeletal pose arguments are invalid"
    );
    ensure!(
        counters[0] == counters[3],
        "skeletal visible counters diverged"
    );
    ensure!(counters[18] == 0, "skeletal visible buffer overflowed");
    ensure!(
        counters[14] == counters[17],
        "skeletal pose dispatch and active counts diverged"
    );
    let gpu = decode_counts(&counters, input.settings.bone_count);
    let projection = input.snapshot.projection()?;
    let cpu_oracle = oracle::evaluate(
        input.mesh_catalog,
        input.settings,
        oracle::EvaluationInput {
            scene: input.scene,
            viewport: [input.width, input.height],
            projection,
            grounding: oracle::GroundingInput {
                numerators: input.ground_numerators,
                denominator: input.ground_denominator,
                instance_records: input.instance_records,
                local_ids: input.local_ids,
                presentations: input.presentations,
            },
        },
    )?;
    if gpu != cpu_oracle {
        bail!("skeletal GPU counters {gpu:?} differ from CPU oracle {cpu_oracle:?}");
    }
    let palette_sample = if gpu.active_poses == 0 {
        None
    } else {
        Some(unsafe { read_palette_sample(input.resources, input.animation_catalog) }?)
    };
    let milliseconds = |start: usize, end: usize| {
        timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
            / input.timestamp_frequency as f64
    };
    Ok(SkeletalProbe {
        revision: SKELETAL_REVISION,
        config: input.snapshot.config,
        logical_instance_count: input.snapshot.config.logical_instance_count(),
        candidate_instance_count: input.snapshot.config.candidate_instance_count(),
        settings: input.settings_json,
        gpu,
        cpu_oracle,
        pose_dispatch_groups: counters[14],
        reset_dispatch_count: 1,
        cull_dispatch_count: 1,
        pose_compact_dispatch_count: 1,
        indirect_pose_dispatch_count: 1,
        indirect_mesh_dispatch_count: 1,
        palette_write_bytes: u64::from(counters[17])
            * u64::from(input.settings.bone_count)
            * size_of::<animation_catalog::Affine>() as u64,
        palette_sample,
        imported_geometry: imported_geometry_probe(input.mesh_catalog),
        meshlet_catalog_sha256: input.mesh_catalog_sha256.to_owned(),
        animation_catalog_sha256: input.animation_catalog_sha256.to_owned(),
        gpu_cull_classify_ms: milliseconds(0, 1),
        gpu_pose_compact_ms: milliseconds(1, 2),
        gpu_pose_evaluate_ms: milliseconds(2, 3),
        gpu_mesh_skin_ms: milliseconds(3, 4),
        gpu_total_ms: milliseconds(0, 4),
    })
}

fn imported_geometry_probe(catalog: &MeshletCatalog) -> ImportedGeometryProbe {
    let mut lod_meshlet_counts = [0; 3];
    let mut lod_emitted_vertex_counts = [0; 3];
    for lod in 0..LOD_COUNT {
        let descriptor = catalog.lod(IMPORTED_ARCHETYPE, lod);
        lod_meshlet_counts[lod as usize] = descriptor.meshlet_count;
        lod_emitted_vertex_counts[lod as usize] = descriptor.vertex_count;
    }
    ImportedGeometryProbe {
        revision: catalog.imported.revision,
        archetype: IMPORTED_ARCHETYPE,
        source_json_sha256: catalog.imported.source_json_sha256.clone(),
        source_bin_sha256: catalog.imported.source_bin_sha256.clone(),
        source_texture_sha256: catalog.imported.source_texture_sha256.clone(),
        cooked_sha256: catalog.imported.cooked_sha256.clone(),
        vertex_start: catalog.imported.vertex_start,
        vertex_count: catalog.imported.vertex_count,
        lod_index_counts: catalog.imported.lod_index_counts,
        lod_triangle_counts: catalog.imported.lod_index_counts.map(|count| count / 3),
        lod_meshlet_counts,
        lod_emitted_vertex_counts,
        lod_errors: catalog.imported.lod_errors,
        bounds_min: catalog.imported.bounds_min,
        bounds_max: catalog.imported.bounds_max,
    }
}

fn decode_counts(counters: &[u32], bone_count: u32) -> WorkloadCounts {
    WorkloadCounts {
        visible: counters[3],
        rejected: counters[4],
        animated: counters[5],
        static_count: counters[6],
        active_poses: counters[17],
        reused_poses: counters[5].saturating_sub(counters[17]),
        evaluated_bones: counters[17] * bone_count,
        lod_counts: [counters[7], counters[8], counters[9]],
        meshlets: counters[10],
        emitted_vertices: counters[11],
        emitted_triangles: counters[12],
        skin_influences: counters[19],
        observed_archetype_mask: counters[13],
    }
}

unsafe fn read_palette_sample(
    resources: &ExecutionResources,
    catalog: &AnimationCatalog,
) -> Result<PaletteSample> {
    let words = unsafe { read_values::<u32>(&resources.sample_readback, 56) }?;
    let (clip, phase, bone_count, variant, pose_slot) =
        (words[0], words[1], words[2], words[3], words[4]);
    ensure!(
        clip < CLIP_COUNT && phase < 64,
        "skeletal sample metadata is invalid"
    );
    let expected = catalog.evaluate_pose(clip, phase, bone_count, variant);
    let sampled_bones = bone_count.min(4);
    let mut maximum_absolute_delta = 0.0f32;
    for bone in 0..sampled_bones as usize {
        for element in 0..12 {
            let actual = f32::from_bits(words[8 + bone * 12 + element]);
            let expected = expected[bone].rows[element / 4][element % 4];
            maximum_absolute_delta = maximum_absolute_delta.max((actual - expected).abs());
        }
    }
    ensure!(
        maximum_absolute_delta <= PALETTE_TOLERANCE,
        "skeletal palette sample delta {maximum_absolute_delta} exceeds {PALETTE_TOLERANCE}"
    );
    Ok(PaletteSample {
        clip,
        phase,
        bone_count,
        variant,
        pose_slot,
        sampled_bones,
        maximum_absolute_delta,
        tolerance: PALETTE_TOLERANCE,
    })
}
