use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::terrain_query::{
    TERRAIN_QUERY_HEIGHT_DENOMINATOR, TERRAIN_QUERY_POSITION_DENOMINATOR, TerrainQueryPosition,
    TerrainTriangle,
};

use super::super::super::renderer::Renderer;
use super::super::super::terrain::TerrainProjection;
use super::super::authority::{self, TriangleClass};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Probe {
    revision: &'static str,
    snapshot_generation: u64,
    global_config: crate::address::GlobalRegionConfig,
    region_count: u32,
    sample_count: u32,
    position_denominator: i32,
    height_denominator: u32,
    result_sha256: String,
    identity_keyed_sha256: String,
    minimum_height_numerator: i32,
    maximum_height_numerator: i32,
    triangles: TriangleCoverage,
    oracle_mismatch_count: u32,
    first_oracle_mismatch: Option<Mismatch>,
    elapsed_ns: u128,
    per_query_allocation_bytes: u64,
    source_read_count: u32,
    gpu_copy_count: u32,
    gpu_readback_count: u32,
    fence_wait_count: u32,
    synchronization_count: u32,
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
struct Mismatch {
    region: crate::world::RegionCoord,
    local_x_q9: i32,
    local_z_q9: i32,
    query_height_numerator: i32,
    oracle_height_numerator: i32,
    query_triangle: TerrainTriangle,
    oracle_triangle: &'static str,
}

pub(super) fn probe(
    renderer: &Renderer,
    assignments: &[crate::terrain::TerrainAssignment],
    tiles: &[terrain_format::TerrainTile],
    projection: TerrainProjection,
) -> Result<Probe> {
    let started = std::time::Instant::now();
    let global_config = renderer
        .terrain_renderer
        .global_config()
        .context("terrain query probe has no global config")?;
    let snapshot_generation = renderer
        .terrain_renderer
        .published_generation()
        .context("terrain query probe has no published generation")?;
    let mut result_digest = Sha256::new();
    let mut identity_digest = Sha256::new();
    let mut triangles = TriangleCoverage {
        first: 0,
        diagonal: 0,
        second: 0,
    };
    let mut sample_count = 0_u32;
    let mut oracle_mismatch_count = 0_u32;
    let mut first_oracle_mismatch = None;
    let mut minimum_height_numerator = i32::MAX;
    let mut maximum_height_numerator = i32::MIN;

    for (active_index, (assignment, tile)) in assignments.iter().zip(tiles).enumerate() {
        ensure!(
            assignment.region_id == tile.region_id,
            "terrain query probe tile identity disagrees with its assignment"
        );
        let semantic_region_id = projection.region_id(active_index)?;
        let semantic_x = semantic_region_id % crate::load::MAX_REGION_SIDE;
        let semantic_z = semantic_region_id / crate::load::MAX_REGION_SIDE;
        let minimum_x = (semantic_x as i32 - 64) * 16 - 8;
        let minimum_z = (semantic_z as i32 - 64) * 16 - 8;
        for cell_z in 0..terrain_format::CELL_SIDE as i32 {
            for cell_x in 0..terrain_format::CELL_SIDE as i32 {
                for offset_q9 in [64_i32, 128, 192] {
                    let local_x_q9 = -4096 + cell_x * 256 + offset_q9;
                    let local_z_q9 = -4096 + cell_z * 256 + offset_q9;
                    let position = TerrainQueryPosition::new(
                        assignment.global_region,
                        local_x_q9,
                        local_z_q9,
                    )?;
                    let query = renderer.query_terrain_height(position)?;
                    let tile_x_q9 = local_x_q9 + 4096;
                    let tile_z_q9 = local_z_q9 + 4096;
                    let oracle_position = [
                        minimum_x as f32
                            + tile_x_q9 as f32 / TERRAIN_QUERY_POSITION_DENOMINATOR as f32,
                        0.0,
                        minimum_z as f32
                            + tile_z_q9 as f32 / TERRAIN_QUERY_POSITION_DENOMINATOR as f32,
                    ];
                    let (oracle_height, oracle_triangle) =
                        authority::sample_ground(tile, oracle_position, semantic_region_id);
                    let triangle_code = count_triangle(&mut triangles, query.triangle);
                    let oracle_matches = query.height_numerator == oracle_height
                        && matches!(
                            (query.triangle, oracle_triangle),
                            (TerrainTriangle::First, TriangleClass::First)
                                | (TerrainTriangle::Diagonal, TriangleClass::Diagonal)
                                | (TerrainTriangle::Second, TriangleClass::Second)
                        );
                    if !oracle_matches {
                        oracle_mismatch_count += 1;
                        first_oracle_mismatch.get_or_insert(Mismatch {
                            region: assignment.global_region,
                            local_x_q9,
                            local_z_q9,
                            query_height_numerator: query.height_numerator,
                            oracle_height_numerator: oracle_height,
                            query_triangle: query.triangle,
                            oracle_triangle: triangle_name(oracle_triangle),
                        });
                    }

                    result_digest.update(query.height_numerator.to_le_bytes());
                    result_digest.update([triangle_code]);
                    identity_digest.update(assignment.global_region.x.to_le_bytes());
                    identity_digest.update(assignment.global_region.z.to_le_bytes());
                    identity_digest.update(local_x_q9.to_le_bytes());
                    identity_digest.update(local_z_q9.to_le_bytes());
                    identity_digest.update(query.height_numerator.to_le_bytes());
                    identity_digest.update([triangle_code]);
                    minimum_height_numerator = minimum_height_numerator.min(query.height_numerator);
                    maximum_height_numerator = maximum_height_numerator.max(query.height_numerator);
                    sample_count += 1;
                }
            }
        }
    }
    ensure!(
        oracle_mismatch_count == 0,
        "canonical terrain query differs from the independent grounding oracle"
    );

    Ok(Probe {
        revision: "exact-canonical-terrain-query-v1",
        snapshot_generation,
        global_config,
        region_count: assignments.len() as u32,
        sample_count,
        position_denominator: TERRAIN_QUERY_POSITION_DENOMINATOR,
        height_denominator: TERRAIN_QUERY_HEIGHT_DENOMINATOR,
        result_sha256: format!("{:x}", result_digest.finalize()),
        identity_keyed_sha256: format!("{:x}", identity_digest.finalize()),
        minimum_height_numerator,
        maximum_height_numerator,
        triangles,
        oracle_mismatch_count,
        first_oracle_mismatch,
        elapsed_ns: started.elapsed().as_nanos(),
        per_query_allocation_bytes: 0,
        source_read_count: 0,
        gpu_copy_count: 0,
        gpu_readback_count: 0,
        fence_wait_count: 0,
        synchronization_count: 0,
    })
}

fn count_triangle(coverage: &mut TriangleCoverage, triangle: TerrainTriangle) -> u8 {
    match triangle {
        TerrainTriangle::First => {
            coverage.first += 1;
            0
        }
        TerrainTriangle::Diagonal => {
            coverage.diagonal += 1;
            1
        }
        TerrainTriangle::Second => {
            coverage.second += 1;
            2
        }
    }
}

fn triangle_name(triangle: TriangleClass) -> &'static str {
    match triangle {
        TriangleClass::First => "first",
        TriangleClass::Diagonal => "diagonal",
        TriangleClass::Second => "second",
    }
}
