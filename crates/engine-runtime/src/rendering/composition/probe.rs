use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::super::meshlet_scene::SurfaceProbe;
use super::super::renderer::Renderer;
use super::super::terrain::TerrainProbe;
use super::COMPOSITION_REVISION;
use super::authority::{self, TriangleClass};
use super::contact::{self, ContactProbe};

mod objects;
mod terrain_query;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionProbe {
    revision: &'static str,
    pair: Value,
    canonical_objects: objects::CanonicalObjectEvidence,
    grounding: GroundingProbe,
    terrain_query: terrain_query::Probe,
    simulation_schedule: Value,
    contact: ContactProbe,
    terrain: TerrainProbe,
    surface: SurfaceProbe,
    clear_count: u32,
    fixed_terrain_dispatches: u32,
    fixed_skeletal_dispatches: u32,
    timing: CompositionTiming,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroundingProbe {
    authority: &'static str,
    grounding_mode: u32,
    ground_denominator: u32,
    position_lattice_denominator: u32,
    position_sha256: String,
    identity_keyed_position_sha256: String,
    identity_keyed_ground_sha256: String,
    candidate_count: u32,
    gpu_sha256: String,
    cpu_sha256: String,
    minimum_numerator: i32,
    maximum_numerator: i32,
    mismatch_count: u32,
    first_mismatch: Option<GroundMismatch>,
    readback_bytes: u64,
    allocation_bytes: u64,
    instance_readback_bytes: u64,
    instance_readback_allocation_bytes: u64,
    instance_readback_copy_count: u32,
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
        background_color: [f32; 4],
        presentation_tick: u32,
        presentation_status: &serde_json::Value,
        simulation_status: &serde_json::Value,
        actor: Option<crate::rendering::ActorRenderProjection>,
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
        self.composition
            .published
            .as_ref()
            .context("composition probe has no published pair")?;
        let projection = self.terrain_renderer.projection()?;
        let terrain_query = terrain_query::probe(self, assignments, tiles, projection)?;
        ensure!(
            snapshot.object_stable_seed_namespace == authority::object_source_namespace(),
            "composition object stable-seed namespace does not match its authority"
        );

        let payload_readback = unsafe { self.async_resident_renderer.read_active_payload() }?;
        let records = &payload_readback.records;
        ensure!(
            records.len() == assignments.len(),
            "composition payload readback does not match the active mapping"
        );

        let candidate_count = snapshot.config.candidate_instance_count() as usize;
        let gpu = unsafe {
            self.skeletal_scene_renderer
                .read_ground_numerators(candidate_count)
        }?;
        let mut cpu = Vec::with_capacity(candidate_count);
        let mut position_digest = Sha256::new();
        let mut identity_evidence = Vec::with_capacity(candidate_count);
        let mut triangles = TriangleCoverage {
            first: 0,
            diagonal: 0,
            second: 0,
        };
        for (active_index, (assignment, tile)) in assignments.iter().zip(tiles).enumerate() {
            ensure!(
                assignment.region_id == tile.region_id,
                "composition terrain tile does not match its logical mapping"
            );
            let region_records = &records[active_index];
            let region_local_ids = &payload_readback.local_ids[active_index];
            ensure!(
                region_records.len() == crate::load::INSTANCES_PER_REGION as usize
                    && region_local_ids.len() == region_records.len(),
                "composition payload page has the wrong record or identity count"
            );
            let semantic_region_id = projection.region_id(active_index)?;
            for (local_index, record) in region_records.iter().enumerate() {
                let position = projection.position(active_index, record.position)?;
                position_digest.update(position[0].to_bits().to_le_bytes());
                position_digest.update(position[2].to_bits().to_le_bytes());
                let (ground, triangle) =
                    authority::sample_ground(tile, position, semantic_region_id);
                cpu.push(ground);
                identity_evidence.push((
                    active_index as u32,
                    region_local_ids[local_index],
                    position[0].to_bits(),
                    position[2].to_bits(),
                    ground,
                ));
                match triangle {
                    TriangleClass::First => triangles.first += 1,
                    TriangleClass::Diagonal => triangles.diagonal += 1,
                    TriangleClass::Second => triangles.second += 1,
                }
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
        let boundaries = boundary_probe(
            assignments,
            records,
            &payload_readback.local_ids,
            &cpu,
            projection,
        )?;
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
        identity_evidence
            .sort_by_key(|(active_index, local_id, _, _, _)| (*active_index, *local_id));
        let mut identity_position = Sha256::new();
        let mut identity_ground = Sha256::new();
        for (active_index, local_id, x, z, ground) in identity_evidence {
            for digest in [&mut identity_position, &mut identity_ground] {
                digest.update(active_index.to_le_bytes());
                digest.update(local_id.to_le_bytes());
            }
            identity_position.update(x.to_le_bytes());
            identity_position.update(z.to_le_bytes());
            identity_ground.update(ground.to_le_bytes());
        }
        let canonical_objects = objects::canonical_object_evidence(
            snapshot.object_source_namespace,
            snapshot.object_stable_seed_namespace,
            assignments,
            records,
            projection,
            &payload_readback,
        )?;
        let terrain = unsafe { self.terrain_renderer.read_probe(scene) }?;
        let skeletal = unsafe {
            self.skeletal_scene_renderer.read_composition_probe(
                super::super::meshlet_scene::CompositionProbeInput {
                    snapshot,
                    scene,
                    presentation_tick,
                    ground_numerators: &cpu,
                    ground_denominator: authority::GROUND_DENOMINATOR,
                    instance_records: records,
                    local_ids: &payload_readback.local_ids,
                    presentations: &payload_readback.presentations,
                    actor,
                },
            )
        }?;
        let contact = contact::evaluate(contact::ContactInput {
            assignments,
            tiles,
            records,
            local_ids: &payload_readback.local_ids,
            exact_ground: &cpu,
            ground_denominator: authority::GROUND_DENOMINATOR,
            scene,
            projection,
        })?;
        let mesh_read_count = skeletal.visible_count();
        let skeletal_timing = skeletal.gpu_timing();
        let terrain_total_ms = terrain.total_gpu_ms();
        let surface = unsafe {
            self.skeletal_scene_renderer.read_composition_surface_probe(
                skeletal,
                super::super::meshlet_scene::CompositionSurfaceInput {
                    scene,
                    presentation_tick,
                    background_color,
                    instance_records: records,
                    local_ids: &payload_readback.local_ids,
                    presentations: &payload_readback.presentations,
                    projection,
                    ground_numerators: &cpu,
                    ground_denominator: authority::GROUND_DENOMINATOR,
                    actor,
                },
            )
        }?;
        let mut pair = self.composition_status();
        pair["presentationClock"] = presentation_status.clone();
        Ok(CompositionProbe {
            revision: COMPOSITION_REVISION,
            pair,
            canonical_objects,
            grounding: GroundingProbe {
                authority: authority::NAME,
                grounding_mode: authority::GROUNDING_MODE,
                ground_denominator: authority::GROUND_DENOMINATOR,
                position_lattice_denominator: authority::POSITION_DENOMINATOR,
                position_sha256: format!("{:x}", position_digest.finalize()),
                identity_keyed_position_sha256: format!("{:x}", identity_position.finalize()),
                identity_keyed_ground_sha256: format!("{:x}", identity_ground.finalize()),
                candidate_count: candidate_count as u32,
                gpu_sha256: hash(&gpu),
                cpu_sha256: hash(&cpu),
                minimum_numerator,
                maximum_numerator,
                mismatch_count,
                first_mismatch,
                readback_bytes: candidate_count as u64 * 4,
                allocation_bytes: super::super::meshlet_scene::GROUND_BYTES,
                instance_readback_bytes: payload_readback.readback_bytes,
                instance_readback_allocation_bytes: payload_readback.allocation_bytes,
                instance_readback_copy_count: payload_readback.copy_count,
                cull_write_count: candidate_count as u32,
                mesh_read_count,
                gpu_fused_cull_ms: skeletal_timing[0],
                triangles,
                boundaries,
            },
            terrain_query,
            simulation_schedule: simulation_status.clone(),
            contact,
            terrain,
            surface,
            clear_count: 1,
            fixed_terrain_dispatches: 4,
            fixed_skeletal_dispatches: 6,
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
    local_ids: &[Vec<u32>],
    ground: &[i32],
    projection: super::super::terrain::TerrainProjection,
) -> Result<BoundaryProbe> {
    let active = assignments
        .iter()
        .enumerate()
        .map(|(index, _)| {
            projection
                .region_id(index)
                .map(|region_id| (region_id, index))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut result = BoundaryProbe {
        logical_neighbor_edges: 0,
        pair_comparisons: 0,
        position_mismatch_count: 0,
        ground_mismatch_count: 0,
        first_mismatch: None,
    };
    for first_index in 0..assignments.len() {
        let region_id = projection.region_id(first_index)?;
        let region_x = region_id % crate::load::MAX_REGION_SIDE;
        for (second_region_id, x_edge) in [
            (region_id + 1, true),
            (region_id + crate::load::MAX_REGION_SIDE, false),
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
                let first_physical = local_ids[first_index]
                    .iter()
                    .position(|local_id| *local_id == first_local as u32)
                    .context("first boundary local ID is absent")?;
                let second_physical = local_ids[second_index]
                    .iter()
                    .position(|local_id| *local_id == second_local as u32)
                    .context("second boundary local ID is absent")?;
                let first_position = projection
                    .position(first_index, records[first_index][first_physical].position)?;
                let second_position = projection.position(
                    second_index,
                    records[second_index][second_physical].position,
                )?;
                let position_matches = first_position[0].to_bits() == second_position[0].to_bits()
                    && first_position[2].to_bits() == second_position[2].to_bits();
                let first_ground = ground
                    [first_index * crate::load::INSTANCES_PER_REGION as usize + first_physical];
                let second_ground = ground
                    [second_index * crate::load::INSTANCES_PER_REGION as usize + second_physical];
                result.pair_comparisons += 1;
                result.position_mismatch_count += u32::from(!position_matches);
                result.ground_mismatch_count += u32::from(first_ground != second_ground);
                if result.first_mismatch.is_none()
                    && (!position_matches || first_ground != second_ground)
                {
                    result.first_mismatch = Some(BoundaryMismatch {
                        first_region_id: region_id,
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
    Ok(result)
}
