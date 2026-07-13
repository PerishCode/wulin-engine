use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::meshlet_scene::SkeletalProbe;
use super::super::renderer::Renderer;
use super::super::terrain::TerrainProbe;
use super::contact::{self, ContactProbe};
use super::fixture::{self, CompositionFixture, TriangleClass};
use super::{COMPOSITION_REVISION, CompositionOrder};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionProbe {
    revision: &'static str,
    order: CompositionOrder,
    pair: Value,
    grounding: GroundingProbe,
    contact: ContactProbe,
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
    fixture: CompositionFixture,
    grounding_mode: u32,
    ground_denominator: u32,
    position_lattice_denominator: u32,
    position_sha256: String,
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
    triangles: TriangleCoverage,
    boundaries: BoundaryProbe,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TriangleCoverage {
    first: u32,
    diagonal: u32,
    second: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoundaryProbe {
    logical_neighbor_edges: u32,
    pair_comparisons: u32,
    position_mismatch_count: u32,
    ground_mismatch_count: u32,
    first_mismatch: Option<BoundaryMismatch>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BoundaryMismatch {
    first_region_id: u32,
    second_region_id: u32,
    first_local_index: u32,
    second_local_index: u32,
    position_matches: bool,
    first_ground: i32,
    second_ground: i32,
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
        let fixture = self
            .composition
            .published
            .as_ref()
            .context("composition probe has no published pair")?
            .fixture;

        let candidate_count = snapshot.config.candidate_instance_count() as usize;
        let gpu = unsafe {
            self.skeletal_scene_renderer
                .read_ground_numerators(candidate_count)
        }?;
        let mut cpu = Vec::with_capacity(candidate_count);
        let mut records = Vec::with_capacity(assignments.len());
        let mut position_digest = Sha256::new();
        let mut triangles = TriangleCoverage {
            first: 0,
            diagonal: 0,
            second: 0,
        };
        for (assignment, tile) in assignments.iter().zip(tiles) {
            ensure!(
                assignment.region_id == tile.region_id,
                "composition terrain tile does not match its logical mapping"
            );
            let region_records = fixture::generate_fixture_region(assignment.region_id, fixture);
            for (local_index, record) in region_records.iter().enumerate() {
                position_digest.update(record.position[0].to_bits().to_le_bytes());
                position_digest.update(record.position[2].to_bits().to_le_bytes());
                let (ground, triangle) = fixture::sample_ground(tile, local_index, fixture);
                cpu.push(ground);
                match triangle {
                    TriangleClass::First => triangles.first += 1,
                    TriangleClass::Diagonal => triangles.diagonal += 1,
                    TriangleClass::Second => triangles.second += 1,
                }
            }
            records.push(region_records);
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
        let boundaries = boundary_probe(assignments, &records, &cpu, fixture);
        ensure!(
            boundaries.position_mismatch_count == 0 && boundaries.ground_mismatch_count == 0,
            "composition boundary samples diverged"
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
            self.skeletal_scene_renderer.read_grounded_probe(
                snapshot,
                scene,
                &cpu,
                fixture.ground_denominator(),
                &records,
            )
        }?;
        let contact = contact::evaluate(
            assignments,
            tiles,
            &records,
            &cpu,
            fixture.ground_denominator(),
            scene,
            self.terrain_renderer.lod_settings(),
        )?;
        let mesh_read_count = skeletal.visible_count();
        let skeletal_timing = skeletal.gpu_timing();
        let terrain_total_ms = terrain.total_gpu_ms();
        Ok(CompositionProbe {
            revision: COMPOSITION_REVISION,
            order: self.composition.order,
            pair: self.composition.status_json(),
            grounding: GroundingProbe {
                fixture,
                grounding_mode: fixture.grounding_mode(),
                ground_denominator: fixture.ground_denominator(),
                position_lattice_denominator: fixture.position_denominator(),
                position_sha256: format!("{:x}", position_digest.finalize()),
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
                triangles,
                boundaries,
            },
            contact,
            terrain,
            skeletal,
            clear_count: 1,
            fixed_terrain_dispatches: 3 + u32::from(self.terrain_renderer.lod_settings().enabled),
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

fn boundary_probe(
    assignments: &[crate::terrain::TerrainAssignment],
    records: &[Vec<crate::resident::InstanceRecord>],
    ground: &[i32],
    fixture: CompositionFixture,
) -> BoundaryProbe {
    if fixture == CompositionFixture::CellCenter {
        return BoundaryProbe {
            logical_neighbor_edges: 0,
            pair_comparisons: 0,
            position_mismatch_count: 0,
            ground_mismatch_count: 0,
            first_mismatch: None,
        };
    }
    let active = assignments
        .iter()
        .enumerate()
        .map(|(index, assignment)| (assignment.region_id, index))
        .collect::<BTreeMap<_, _>>();
    let mut result = BoundaryProbe {
        logical_neighbor_edges: 0,
        pair_comparisons: 0,
        position_mismatch_count: 0,
        ground_mismatch_count: 0,
        first_mismatch: None,
    };
    for (first_index, assignment) in assignments.iter().enumerate() {
        let region_x = assignment.region_id % crate::load::MAX_REGION_SIDE;
        for (second_region_id, x_edge) in [
            (assignment.region_id + 1, true),
            (assignment.region_id + crate::load::MAX_REGION_SIDE, false),
        ] {
            if x_edge && region_x + 1 >= crate::load::MAX_REGION_SIDE {
                continue;
            }
            let Some(&second_index) = active.get(&second_region_id) else {
                continue;
            };
            result.logical_neighbor_edges += 1;
            for along in 0..terrain_format::CELL_SIDE {
                let (first_local, second_local) = if x_edge {
                    (
                        along * terrain_format::CELL_SIDE + 31,
                        along * terrain_format::CELL_SIDE,
                    )
                } else {
                    (31 * terrain_format::CELL_SIDE + along, along)
                };
                let first_record = records[first_index][first_local];
                let second_record = records[second_index][second_local];
                let position_matches = first_record.position[0].to_bits()
                    == second_record.position[0].to_bits()
                    && first_record.position[2].to_bits() == second_record.position[2].to_bits();
                let first_ground =
                    ground[first_index * crate::load::INSTANCES_PER_REGION as usize + first_local];
                let second_ground = ground
                    [second_index * crate::load::INSTANCES_PER_REGION as usize + second_local];
                result.pair_comparisons += 1;
                result.position_mismatch_count += u32::from(!position_matches);
                result.ground_mismatch_count += u32::from(first_ground != second_ground);
                if result.first_mismatch.is_none()
                    && (!position_matches || first_ground != second_ground)
                {
                    result.first_mismatch = Some(BoundaryMismatch {
                        first_region_id: assignment.region_id,
                        second_region_id,
                        first_local_index: first_local as u32,
                        second_local_index: second_local as u32,
                        position_matches,
                        first_ground,
                        second_ground,
                    });
                }
            }
        }
    }
    result
}
