use anyhow::{Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::load::LoadConfig;
use crate::scene::Camera;
use crate::terrain::TerrainAssignment;

use super::projection::TerrainProjection;

pub(super) const LOD_REVISION: &str = "gpu-terrain-lod-v1";
pub(in crate::rendering) const PATCHES_PER_REGION_SIDE: u32 = 4;
pub(in crate::rendering) const PATCH_CELL_SIDE: u32 = 8;
const LOD_LEVEL_COUNT: usize = 3;
const PATCH_WORLD_SIDE_METERS: f32 = 4.0;
const WORLD_PATCH_ORIGIN_METERS: f32 = -1_032.0;
pub(super) const NEAR_PATCH_RADIUS: u32 = 2;
pub(super) const MIDDLE_PATCH_RADIUS: u32 = 6;

#[derive(Clone, Copy)]
struct Patch<'a> {
    tile: &'a terrain_format::TerrainTile,
    patch_x: u32,
    patch_z: u32,
    global_x: i32,
    global_z: i32,
    lod: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TerrainLodOracle {
    pub revision: &'static str,
    pub near_patch_radius: u32,
    pub middle_patch_radius: u32,
    pub camera_patch: [i32; 2],
    pub lod_sha256: String,
    pub lod_counts: [u32; LOD_LEVEL_COUNT],
    pub geometry: LodGeometry,
    pub edges: LodEdges,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LodGeometry {
    pub patches: u32,
    pub vertices: u32,
    pub triangles: u32,
    pub baseline_vertices: u32,
    pub baseline_triangles: u32,
    pub reduced_vertices: u32,
    pub reduced_triangles: u32,
    pub vertex_reduction_percent: f64,
    pub triangle_reduction_percent: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LodEdges {
    pub patch_edges: u32,
    pub same_lod_edges: u32,
    pub transition_edges: u32,
    pub adjusted_vertices: u32,
    pub sample_comparisons: u32,
    pub max_lod_delta: u32,
    pub mismatch_count: u32,
    pub first_mismatch: Option<LodEdgeMismatch>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LodEdgeMismatch {
    patch: [i32; 2],
    neighbor_patch: [i32; 2],
    axis: &'static str,
    sample: u32,
    denominator: u32,
    value_numerator: i32,
    neighbor_numerator: i32,
}

pub(super) fn camera_patch(camera: Camera) -> [i32; 2] {
    [
        ((camera.position[0] - WORLD_PATCH_ORIGIN_METERS) / PATCH_WORLD_SIDE_METERS).floor() as i32,
        ((camera.position[2] - WORLD_PATCH_ORIGIN_METERS) / PATCH_WORLD_SIDE_METERS).floor() as i32,
    ]
}

pub(super) fn evaluate(
    config: LoadConfig,
    active: &[TerrainAssignment],
    tiles: &[terrain_format::TerrainTile],
    camera: Camera,
    projection: TerrainProjection,
) -> Result<TerrainLodOracle> {
    ensure!(
        active.len() == tiles.len(),
        "terrain LOD snapshot shape mismatch"
    );
    let region_side = config.active_radius * 2 + 1;
    let patch_side = region_side * PATCHES_PER_REGION_SIDE;
    let camera_patch = camera_patch(camera);
    let patches = build_patches(
        active,
        tiles,
        region_side,
        patch_side,
        camera_patch,
        projection,
    )?;
    let mut lod_counts = [0u32; LOD_LEVEL_COUNT];
    let mut vertices = 0;
    let mut triangles = 0;
    let mut lod_hash = Sha256::new();
    for patch in &patches {
        lod_counts[patch.lod as usize] += 1;
        let cell_side = PATCH_CELL_SIDE >> patch.lod;
        vertices += (cell_side + 1).pow(2);
        triangles += cell_side.pow(2) * 2;
        lod_hash.update(patch.global_x.to_le_bytes());
        lod_hash.update(patch.global_z.to_le_bytes());
        lod_hash.update(patch.lod.to_le_bytes());
    }
    let edges = evaluate_edges(&patches, patch_side);
    ensure!(
        edges.max_lod_delta <= 1,
        "terrain LOD oracle produced an adjacent delta greater than one"
    );
    let patch_count = patches.len() as u32;
    let baseline_vertices = patch_count * 81;
    let baseline_triangles = patch_count * 128;
    Ok(TerrainLodOracle {
        revision: LOD_REVISION,
        near_patch_radius: NEAR_PATCH_RADIUS,
        middle_patch_radius: MIDDLE_PATCH_RADIUS,
        camera_patch,
        lod_sha256: format!("{:x}", lod_hash.finalize()),
        lod_counts,
        geometry: LodGeometry {
            patches: patch_count,
            vertices,
            triangles,
            baseline_vertices,
            baseline_triangles,
            reduced_vertices: baseline_vertices - vertices,
            reduced_triangles: baseline_triangles - triangles,
            vertex_reduction_percent: reduction_percent(baseline_vertices, vertices),
            triangle_reduction_percent: reduction_percent(baseline_triangles, triangles),
        },
        edges,
    })
}

fn build_patches<'a>(
    active: &[TerrainAssignment],
    tiles: &'a [terrain_format::TerrainTile],
    region_side: u32,
    patch_side: u32,
    camera: [i32; 2],
    projection: TerrainProjection,
) -> Result<Vec<Patch<'a>>> {
    let mut patches = Vec::with_capacity((patch_side * patch_side) as usize);
    for grid_z in 0..patch_side {
        for grid_x in 0..patch_side {
            let region_index =
                (grid_z / PATCHES_PER_REGION_SIDE) * region_side + grid_x / PATCHES_PER_REGION_SIDE;
            let assignment = active
                .get(region_index as usize)
                .ok_or_else(|| anyhow::anyhow!("terrain LOD active mapping is incomplete"))?;
            let tile = &tiles[region_index as usize];
            ensure!(
                assignment.region_id == tile.region_id,
                "terrain LOD tile does not match active mapping"
            );
            let projected_region = projection.region_id(region_index as usize)?;
            let region_x = (projected_region % terrain_format::WORLD_REGION_SIDE) as i32;
            let region_z = (projected_region / terrain_format::WORLD_REGION_SIDE) as i32;
            let local_x = grid_x % PATCHES_PER_REGION_SIDE;
            let local_z = grid_z % PATCHES_PER_REGION_SIDE;
            let global_x = region_x * PATCHES_PER_REGION_SIDE as i32 + local_x as i32;
            let global_z = region_z * PATCHES_PER_REGION_SIDE as i32 + local_z as i32;
            patches.push(Patch {
                tile,
                patch_x: local_x * PATCH_CELL_SIDE,
                patch_z: local_z * PATCH_CELL_SIDE,
                global_x,
                global_z,
                lod: select_lod(global_x, global_z, camera),
            });
        }
    }
    Ok(patches)
}

pub(in crate::rendering) fn selected_lod(patch_x: i32, patch_z: i32, camera: Camera) -> u32 {
    select_lod(patch_x, patch_z, camera_patch(camera))
}

fn select_lod(patch_x: i32, patch_z: i32, camera: [i32; 2]) -> u32 {
    let distance = patch_x.abs_diff(camera[0]).max(patch_z.abs_diff(camera[1]));
    if distance <= NEAR_PATCH_RADIUS {
        0
    } else if distance <= MIDDLE_PATCH_RADIUS {
        1
    } else {
        2
    }
}

fn evaluate_edges(patches: &[Patch<'_>], patch_side: u32) -> LodEdges {
    let mut result = LodEdges {
        patch_edges: 0,
        same_lod_edges: 0,
        transition_edges: 0,
        adjusted_vertices: 0,
        sample_comparisons: 0,
        max_lod_delta: 0,
        mismatch_count: 0,
        first_mismatch: None,
    };
    for z in 0..patch_side {
        for x in 0..patch_side {
            let index = (z * patch_side + x) as usize;
            if x + 1 < patch_side {
                compare_edge(&mut result, patches[index], patches[index + 1], "x");
            }
            if z + 1 < patch_side {
                compare_edge(
                    &mut result,
                    patches[index],
                    patches[index + patch_side as usize],
                    "z",
                );
            }
        }
    }
    result
}

fn compare_edge(result: &mut LodEdges, patch: Patch<'_>, neighbor: Patch<'_>, axis: &'static str) {
    result.patch_edges += 1;
    let delta = patch.lod.abs_diff(neighbor.lod);
    result.max_lod_delta = result.max_lod_delta.max(delta);
    if delta == 0 {
        result.same_lod_edges += 1;
    } else {
        result.transition_edges += 1;
        let fine_step = 1u32 << patch.lod.min(neighbor.lod);
        let coarse_step = 1u32 << patch.lod.max(neighbor.lod);
        result.adjusted_vertices += (0..=PATCH_CELL_SIDE)
            .step_by(fine_step as usize)
            .filter(|sample| sample % coarse_step != 0)
            .count() as u32;
    }
    let denominator = 1u32 << patch.lod.max(neighbor.lod);
    for sample in 0..=PATCH_CELL_SIDE {
        let value = edge_numerator(patch, axis, true, sample, denominator);
        let neighbor_value = edge_numerator(neighbor, axis, false, sample, denominator);
        result.sample_comparisons += 1;
        if value != neighbor_value {
            result.mismatch_count += 1;
            result.first_mismatch.get_or_insert(LodEdgeMismatch {
                patch: [patch.global_x, patch.global_z],
                neighbor_patch: [neighbor.global_x, neighbor.global_z],
                axis,
                sample,
                denominator,
                value_numerator: value,
                neighbor_numerator: neighbor_value,
            });
        }
    }
}

fn edge_numerator(
    patch: Patch<'_>,
    axis: &str,
    positive_edge: bool,
    sample: u32,
    step: u32,
) -> i32 {
    let base = sample / step * step;
    let next = (base + step).min(PATCH_CELL_SIDE);
    let remainder = sample - base;
    let sample_height = |along: u32| {
        let (x, z) = match axis {
            "x" => (
                patch.patch_x + if positive_edge { PATCH_CELL_SIDE } else { 0 },
                patch.patch_z + along,
            ),
            _ => (
                patch.patch_x + along,
                patch.patch_z + if positive_edge { PATCH_CELL_SIDE } else { 0 },
            ),
        };
        patch.tile.heights[(z as usize * terrain_format::SAMPLE_SIDE) + x as usize] as i32
    };
    sample_height(base) * (step - remainder) as i32 + sample_height(next) * remainder as i32
}

fn reduction_percent(baseline: u32, emitted: u32) -> f64 {
    f64::from(baseline - emitted) * 100.0 / f64::from(baseline)
}
