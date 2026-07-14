use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::async_resident::{ObjectSourceNamespace, canonical_stable_seed};
use crate::world::RegionCoord;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_objects: Option<CanonicalObjectEvidence>,
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
struct CanonicalObjectEvidence {
    revision: &'static str,
    source_namespace: ObjectSourceNamespace,
    entry_count: usize,
    semantic_collision_count: usize,
    stable_seed_collision_count: usize,
    mismatch_count: usize,
    content_sha256: String,
    stable_seed_sha256: String,
    entries: Vec<CanonicalObjectEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalObjectEntry {
    active_index: u32,
    global_region: RegionCoord,
    semantic_region_id: u32,
    object_id: u32,
    stable_seed: u32,
    render_offset_regions: [i32; 2],
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
        let projection = self.terrain_renderer.projection()?;
        let object_projection = snapshot.projection()?;
        ensure!(
            projection.is_canonical() == object_projection.is_canonical(),
            "composition terrain and object projection modes differ"
        );
        ensure!(
            snapshot.object_source_namespace.is_some()
                == snapshot.object_stable_seed_namespace.is_some(),
            "canonical object source and stable-seed namespace presence differs"
        );
        if let Some(namespace) = snapshot.object_stable_seed_namespace {
            ensure!(
                namespace == fixture.object_source_namespace(),
                "composition object stable-seed namespace does not match its fixture"
            );
        }

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
        for (active_index, (assignment, tile)) in assignments.iter().zip(tiles).enumerate() {
            ensure!(
                assignment.region_id == tile.region_id,
                "composition terrain tile does not match its logical mapping"
            );
            let region_records = match (
                snapshot.object_stable_seed_namespace,
                assignment.global_region,
            ) {
                (Some(source), Some(global)) => fixture::generate_canonical_fixture_region(
                    global,
                    canonical_stable_seed(source, global),
                    fixture,
                ),
                (None, _) => fixture::generate_fixture_region(assignment.region_id, fixture),
                (Some(_), None) => {
                    anyhow::bail!("canonical object mapping has no signed region")
                }
            };
            for (local_index, record) in region_records.iter().enumerate() {
                let position = projection.position(active_index, record.position)?;
                position_digest.update(position[0].to_bits().to_le_bytes());
                position_digest.update(position[2].to_bits().to_le_bytes());
                let (ground, triangle) = fixture::sample_ground(
                    tile,
                    local_index,
                    fixture,
                    snapshot
                        .object_stable_seed_namespace
                        .and(assignment.global_region),
                );
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
        let boundaries = boundary_probe(assignments, &records, &cpu, fixture, projection)?;
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
        let canonical_objects = snapshot
            .object_source_namespace
            .zip(snapshot.object_stable_seed_namespace)
            .map(|(source, stable_seed)| {
                canonical_object_evidence(source, stable_seed, assignments, &records, projection)
            })
            .transpose()?;
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
        let contact = contact::evaluate(contact::ContactInput {
            assignments,
            tiles,
            records: &records,
            exact_ground: &cpu,
            ground_denominator: fixture.ground_denominator(),
            scene,
            settings: self.terrain_renderer.lod_settings(),
            projection,
        })?;
        let mesh_read_count = skeletal.visible_count();
        let skeletal_timing = skeletal.gpu_timing();
        let terrain_total_ms = terrain.total_gpu_ms();
        Ok(CompositionProbe {
            revision: COMPOSITION_REVISION,
            order: self.composition.order,
            pair: self.composition.status_json(),
            canonical_objects,
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

fn canonical_object_evidence(
    source_namespace: ObjectSourceNamespace,
    stable_seed_namespace: ObjectSourceNamespace,
    assignments: &[crate::terrain::TerrainAssignment],
    records: &[Vec<crate::resident::InstanceRecord>],
    projection: super::super::terrain::TerrainProjection,
) -> Result<CanonicalObjectEvidence> {
    ensure!(
        projection.is_canonical() && assignments.len() == records.len(),
        "canonical object evidence requires one projected payload per region"
    );
    let mut content_hash = Sha256::new();
    let mut seed_hash = Sha256::new();
    let mut semantic_ids = std::collections::BTreeSet::new();
    let mut stable_seeds = std::collections::BTreeSet::new();
    let mut mismatch_count = 0;
    let mut entries = Vec::with_capacity(assignments.len());
    for (index, (assignment, region_records)) in assignments.iter().zip(records).enumerate() {
        let global = assignment
            .global_region
            .context("canonical object assignment has no signed region")?;
        let stable_seed = canonical_stable_seed(stable_seed_namespace, global);
        let semantic_region_id = projection.region_id(index, assignment.region_id)?;
        let object_id = crate::load::REGION_OBJECT_ID_BASE
            .checked_add(semantic_region_id)
            .and_then(|value| value.checked_add(1))
            .context("canonical object ID overflowed")?;
        let render_offset_regions = projection.render_offset(index)?;
        mismatch_count += region_records
            .iter()
            .filter(|record| record.region_id != stable_seed)
            .count();
        semantic_ids.insert(semantic_region_id);
        stable_seeds.insert(stable_seed);
        content_hash.update(source_namespace.as_bytes());
        content_hash.update(global.x.to_le_bytes());
        content_hash.update(global.z.to_le_bytes());
        content_hash.update(stable_seed.to_le_bytes());
        content_hash.update(crate::resident::as_bytes(region_records));
        seed_hash.update(stable_seed.to_le_bytes());
        entries.push(CanonicalObjectEntry {
            active_index: index as u32,
            global_region: global,
            semantic_region_id,
            object_id,
            stable_seed,
            render_offset_regions,
        });
    }
    Ok(CanonicalObjectEvidence {
        revision: "canonical-generated-object-v1",
        source_namespace,
        entry_count: entries.len(),
        semantic_collision_count: entries.len() - semantic_ids.len(),
        stable_seed_collision_count: entries.len() - stable_seeds.len(),
        mismatch_count,
        content_sha256: format!("{:x}", content_hash.finalize()),
        stable_seed_sha256: format!("{:x}", seed_hash.finalize()),
        entries,
    })
}

fn boundary_probe(
    assignments: &[crate::terrain::TerrainAssignment],
    records: &[Vec<crate::resident::InstanceRecord>],
    ground: &[i32],
    fixture: CompositionFixture,
    projection: super::super::terrain::TerrainProjection,
) -> Result<BoundaryProbe> {
    if fixture == CompositionFixture::CellCenter {
        return Ok(BoundaryProbe {
            logical_neighbor_edges: 0,
            pair_comparisons: 0,
            position_mismatch_count: 0,
            ground_mismatch_count: 0,
            first_mismatch: None,
        });
    }
    let active = assignments
        .iter()
        .enumerate()
        .map(|(index, assignment)| {
            projection
                .region_id(index, assignment.region_id)
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
    for (first_index, assignment) in assignments.iter().enumerate() {
        let region_id = projection.region_id(first_index, assignment.region_id)?;
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
                let first_position =
                    projection.position(first_index, records[first_index][first_local].position)?;
                let second_position = projection
                    .position(second_index, records[second_index][second_local].position)?;
                let position_matches = first_position[0].to_bits() == second_position[0].to_bits()
                    && first_position[2].to_bits() == second_position[2].to_bits();
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
