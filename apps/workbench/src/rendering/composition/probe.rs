use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::meshlet_scene::SkeletalProbe;
use super::super::renderer::Renderer;
use super::super::terrain::TerrainProbe;
use super::{COMPOSITION_REVISION, CompositionOrder};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionProbe {
    revision: &'static str,
    order: CompositionOrder,
    pair: Value,
    grounding: GroundingProbe,
    terrain: TerrainProbe,
    skeletal: SkeletalProbe,
    clear_count: u32,
    fixed_terrain_dispatches: u32,
    fixed_skeletal_dispatches: u32,
    timing: CompositionTiming,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroundingProbe {
    candidate_count: u32,
    gpu_sha256: String,
    cpu_sha256: String,
    minimum_numerator: i32,
    maximum_numerator: i32,
    mismatch_count: u32,
    first_mismatch: Option<GroundMismatch>,
    readback_bytes: u64,
    allocation_bytes: u64,
    cull_write_count: u32,
    mesh_read_count: u32,
    gpu_fused_cull_ms: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroundMismatch {
    logical_index: u32,
    region_id: u32,
    local_index: u32,
    gpu_numerator: i32,
    cpu_numerator: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompositionTiming {
    terrain_total_ms: f64,
    ground_and_cull_classify_ms: f64,
    pose_compact_ms: f64,
    pose_evaluate_ms: f64,
    mesh_skin_ms: f64,
    skeletal_total_ms: f64,
    combined_gpu_ms: f64,
}

impl Renderer {
    pub(in crate::rendering) unsafe fn read_composition_probe(
        &self,
        scene: &crate::scene::SceneState,
    ) -> Result<CompositionProbe> {
        let snapshot = self
            .async_resident_renderer
            .snapshot()
            .context("composition probe has no resident snapshot")?;
        let assignments = self
            .terrain_renderer
            .active_assignments()
            .context("composition probe has no terrain mapping")?;
        let tiles = self
            .terrain_renderer
            .published_tiles()
            .context("composition probe has no terrain tiles")?;
        ensure!(
            assignments.len() == tiles.len(),
            "composition terrain mapping shape differs from tile shape"
        );

        let candidate_count = snapshot.config.candidate_instance_count() as usize;
        let gpu = unsafe {
            self.skeletal_scene_renderer
                .read_ground_numerators(candidate_count)
        }?;
        let mut cpu = Vec::with_capacity(candidate_count);
        for (assignment, tile) in assignments.iter().zip(tiles) {
            ensure!(
                assignment.region_id == tile.region_id,
                "composition terrain tile does not match its logical mapping"
            );
            for local_index in 0..crate::load::INSTANCES_PER_REGION as usize {
                let x = local_index % terrain_format::CELL_SIDE;
                let z = local_index / terrain_format::CELL_SIDE;
                let right = i32::from(tile.heights[z * terrain_format::SAMPLE_SIDE + x + 1]);
                let bottom = i32::from(tile.heights[(z + 1) * terrain_format::SAMPLE_SIDE + x]);
                cpu.push(right + bottom);
            }
        }
        ensure!(
            gpu.len() == candidate_count && cpu.len() == candidate_count,
            "composition ground evidence has the wrong candidate count"
        );
        let first_mismatch = gpu
            .iter()
            .zip(&cpu)
            .enumerate()
            .find(|(_, (actual, expected))| actual != expected)
            .map(|(logical_index, (actual, expected))| {
                let active_index = logical_index / crate::load::INSTANCES_PER_REGION as usize;
                GroundMismatch {
                    logical_index: logical_index as u32,
                    region_id: assignments[active_index].region_id,
                    local_index: (logical_index % crate::load::INSTANCES_PER_REGION as usize)
                        as u32,
                    gpu_numerator: *actual,
                    cpu_numerator: *expected,
                }
            });
        let mismatch_count = gpu
            .iter()
            .zip(&cpu)
            .filter(|(actual, expected)| actual != expected)
            .count() as u32;
        ensure!(
            mismatch_count == 0,
            "composition GPU ground numerators differ from the CPU oracle"
        );
        let hash = |values: &[i32]| {
            let mut digest = Sha256::new();
            for value in values {
                digest.update(value.to_le_bytes());
            }
            format!("{:x}", digest.finalize())
        };
        let minimum_numerator = gpu.iter().copied().min().unwrap_or(0);
        let maximum_numerator = gpu.iter().copied().max().unwrap_or(0);
        let terrain = unsafe { self.terrain_renderer.read_probe(scene) }?;
        let skeletal = unsafe {
            self.skeletal_scene_renderer
                .read_grounded_probe(snapshot, scene, &cpu)
        }?;
        let mesh_read_count = skeletal.visible_count();
        let skeletal_timing = skeletal.gpu_timing();
        let terrain_total_ms = terrain.total_gpu_ms();
        Ok(CompositionProbe {
            revision: COMPOSITION_REVISION,
            order: self.composition.order,
            pair: self.composition.status_json(),
            grounding: GroundingProbe {
                candidate_count: candidate_count as u32,
                gpu_sha256: hash(&gpu),
                cpu_sha256: hash(&cpu),
                minimum_numerator,
                maximum_numerator,
                mismatch_count,
                first_mismatch,
                readback_bytes: candidate_count as u64 * 4,
                allocation_bytes: super::super::meshlet_scene::GROUND_BYTES,
                cull_write_count: candidate_count as u32,
                mesh_read_count,
                gpu_fused_cull_ms: skeletal_timing[0],
            },
            terrain,
            skeletal,
            clear_count: 1,
            fixed_terrain_dispatches: 3,
            fixed_skeletal_dispatches: 5,
            timing: CompositionTiming {
                terrain_total_ms,
                ground_and_cull_classify_ms: skeletal_timing[0],
                pose_compact_ms: skeletal_timing[1],
                pose_evaluate_ms: skeletal_timing[2],
                mesh_skin_ms: skeletal_timing[3],
                skeletal_total_ms: skeletal_timing[4],
                combined_gpu_ms: terrain_total_ms + skeletal_timing[4],
            },
        })
    }
}
