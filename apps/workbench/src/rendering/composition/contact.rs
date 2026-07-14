use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::resident::InstanceRecord;
use crate::scene::SceneState;
use crate::terrain::TerrainAssignment;

use super::super::terrain::{
    PATCH_CELL_SIDE, PATCHES_PER_REGION_SIDE, TerrainLodSettings, TerrainProjection, selected_lod,
};

const Q18_DENOMINATOR: u32 = 262_144;
const CONTACT_THRESHOLD_NUMERATOR: u32 = 32_768;
const PATCH_Q8_SIDE: u32 = PATCH_CELL_SIDE * 256;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ContactProbe {
    revision: &'static str,
    denominator: u32,
    sample_count: u32,
    owner_patch_lod_counts: [u32; 3],
    selected_surface_sha256: String,
    residual_sha256: String,
    identity_keyed_selected_surface_sha256: String,
    identity_keyed_residual_sha256: String,
    negative_count: u32,
    zero_count: u32,
    positive_count: u32,
    minimum_residual_numerator: i32,
    maximum_residual_numerator: i32,
    maximum_absolute_numerator: u32,
    absolute_p50_numerator: u32,
    absolute_p95_numerator: u32,
    absolute_p99_numerator: u32,
    maximum_absolute_meters: f64,
    absolute_p50_meters: f64,
    absolute_p95_meters: f64,
    absolute_p99_meters: f64,
    threshold_numerator: u32,
    threshold_meters: f64,
    exceedance_count: u32,
    first_exceedance: Option<ContactExceedance>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContactExceedance {
    region_id: u32,
    local_index: u32,
    owner_patch_lod: u32,
    exact_ground_q18: i32,
    selected_surface_q18: i32,
    residual_q18: i32,
}

pub(super) struct ContactInput<'a> {
    pub assignments: &'a [TerrainAssignment],
    pub tiles: &'a [terrain_format::TerrainTile],
    pub records: &'a [Vec<InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub exact_ground: &'a [i32],
    pub ground_denominator: u32,
    pub scene: &'a SceneState,
    pub settings: TerrainLodSettings,
    pub projection: TerrainProjection,
}

pub(super) fn evaluate(input: ContactInput<'_>) -> Result<ContactProbe> {
    let ContactInput {
        assignments,
        tiles,
        records,
        local_ids,
        exact_ground,
        ground_denominator,
        scene,
        settings,
        projection,
    } = input;
    ensure!(
        assignments.len() == tiles.len()
            && assignments.len() == records.len()
            && assignments.len() == local_ids.len(),
        "composition contact snapshot shape mismatch"
    );
    let expected_samples = records.iter().map(Vec::len).sum::<usize>();
    ensure!(
        exact_ground.len() == expected_samples,
        "composition contact ground shape mismatch"
    );
    ensure!(
        Q18_DENOMINATOR.is_multiple_of(ground_denominator),
        "composition contact ground denominator is incompatible with Q18"
    );
    let ground_scale = (Q18_DENOMINATOR / ground_denominator) as i32;
    let active_patches = active_patch_set(assignments, projection)?;
    let camera = projection.camera(scene.camera());
    let mut selected_hash = Sha256::new();
    let mut residual_hash = Sha256::new();
    let mut lod_counts = [0; 3];
    let mut absolute = Vec::with_capacity(expected_samples);
    let mut negative_count = 0;
    let mut zero_count = 0;
    let mut positive_count = 0;
    let mut minimum = i32::MAX;
    let mut maximum = i32::MIN;
    let mut exceedance_count = 0;
    let mut first_exceedance = None;
    let mut identity_evidence = Vec::with_capacity(expected_samples);

    for (active_index, ((assignment, tile), region_records)) in
        assignments.iter().zip(tiles).zip(records).enumerate()
    {
        ensure!(
            assignment.region_id == tile.region_id,
            "composition contact tile mapping mismatch"
        );
        let region_id = projection.region_id(active_index, assignment.region_id)?;
        for (local_index, record) in region_records.iter().enumerate() {
            let local_id = local_ids[active_index][local_index];
            let position = projection.position(active_index, record.position)?;
            let q8 = position_q8(position, region_id)?;
            let (selected_q18, owner_lod) =
                selected_surface_q18(tile, region_id, q8, &active_patches, camera, settings);
            let logical_index =
                active_index * crate::load::INSTANCES_PER_REGION as usize + local_index;
            let exact_q18 = exact_ground[logical_index] * ground_scale;
            let residual = selected_q18 - exact_q18;
            lod_counts[owner_lod as usize] += 1;
            selected_hash.update(selected_q18.to_le_bytes());
            residual_hash.update(residual.to_le_bytes());
            identity_evidence.push((active_index as u32, local_id, selected_q18, residual));
            absolute.push(residual.unsigned_abs());
            minimum = minimum.min(residual);
            maximum = maximum.max(residual);
            match residual.cmp(&0) {
                std::cmp::Ordering::Less => negative_count += 1,
                std::cmp::Ordering::Equal => zero_count += 1,
                std::cmp::Ordering::Greater => positive_count += 1,
            }
            if residual.unsigned_abs() > CONTACT_THRESHOLD_NUMERATOR {
                exceedance_count += 1;
                first_exceedance.get_or_insert(ContactExceedance {
                    region_id,
                    local_index: local_id,
                    owner_patch_lod: owner_lod,
                    exact_ground_q18: exact_q18,
                    selected_surface_q18: selected_q18,
                    residual_q18: residual,
                });
            }
        }
    }
    absolute.sort_unstable();
    let p50 = percentile(&absolute, 50);
    let p95 = percentile(&absolute, 95);
    let p99 = percentile(&absolute, 99);
    let maximum_absolute = absolute.last().copied().unwrap_or(0);
    let meters = |value: u32| f64::from(value) / f64::from(Q18_DENOMINATOR);
    identity_evidence.sort_by_key(|(active_index, local_id, _, _)| (*active_index, *local_id));
    let mut identity_selected = Sha256::new();
    let mut identity_residual = Sha256::new();
    for (active_index, local_id, selected, residual) in identity_evidence {
        for digest in [&mut identity_selected, &mut identity_residual] {
            digest.update(active_index.to_le_bytes());
            digest.update(local_id.to_le_bytes());
        }
        identity_selected.update(selected.to_le_bytes());
        identity_residual.update(residual.to_le_bytes());
    }
    Ok(ContactProbe {
        revision: "lod-terrain-contact-v1",
        denominator: Q18_DENOMINATOR,
        sample_count: expected_samples as u32,
        owner_patch_lod_counts: lod_counts,
        selected_surface_sha256: format!("{:x}", selected_hash.finalize()),
        residual_sha256: format!("{:x}", residual_hash.finalize()),
        identity_keyed_selected_surface_sha256: format!("{:x}", identity_selected.finalize()),
        identity_keyed_residual_sha256: format!("{:x}", identity_residual.finalize()),
        negative_count,
        zero_count,
        positive_count,
        minimum_residual_numerator: minimum,
        maximum_residual_numerator: maximum,
        maximum_absolute_numerator: maximum_absolute,
        absolute_p50_numerator: p50,
        absolute_p95_numerator: p95,
        absolute_p99_numerator: p99,
        maximum_absolute_meters: meters(maximum_absolute),
        absolute_p50_meters: meters(p50),
        absolute_p95_meters: meters(p95),
        absolute_p99_meters: meters(p99),
        threshold_numerator: CONTACT_THRESHOLD_NUMERATOR,
        threshold_meters: meters(CONTACT_THRESHOLD_NUMERATOR),
        exceedance_count,
        first_exceedance,
    })
}

fn active_patch_set(
    assignments: &[TerrainAssignment],
    projection: TerrainProjection,
) -> Result<BTreeSet<(i32, i32)>> {
    let mut patches = BTreeSet::new();
    for (index, assignment) in assignments.iter().enumerate() {
        let region_id = projection.region_id(index, assignment.region_id)?;
        let region_x = (region_id % terrain_format::WORLD_REGION_SIDE) as i32;
        let region_z = (region_id / terrain_format::WORLD_REGION_SIDE) as i32;
        for patch_z in 0..PATCHES_PER_REGION_SIDE {
            for patch_x in 0..PATCHES_PER_REGION_SIDE {
                patches.insert((
                    region_x * PATCHES_PER_REGION_SIDE as i32 + patch_x as i32,
                    region_z * PATCHES_PER_REGION_SIDE as i32 + patch_z as i32,
                ));
            }
        }
    }
    Ok(patches)
}

fn position_q8(position: [f32; 3], region_id: u32) -> Result<[u32; 2]> {
    let region_x = (region_id % terrain_format::WORLD_REGION_SIDE) as i32;
    let region_z = (region_id / terrain_format::WORLD_REGION_SIDE) as i32;
    let minimum_x = ((region_x - 64) * 16 - 8) as f32;
    let minimum_z = ((region_z - 64) * 16 - 8) as f32;
    let convert = |value: f32, minimum: f32| -> Result<u32> {
        ensure!(
            value.is_finite(),
            "composition contact position is not finite"
        );
        let q8 = ((value - minimum) * 512.0).round() as i32;
        ensure!(
            (0..=terrain_format::CELL_SIDE as i32 * 256).contains(&q8),
            "composition contact position is outside its owning region"
        );
        Ok(q8 as u32)
    };
    Ok([
        convert(position[0], minimum_x)?,
        convert(position[2], minimum_z)?,
    ])
}

fn selected_surface_q18(
    tile: &terrain_format::TerrainTile,
    region_id: u32,
    q8: [u32; 2],
    active_patches: &BTreeSet<(i32, i32)>,
    camera: crate::scene::Camera,
    settings: TerrainLodSettings,
) -> (i32, u32) {
    let patch_x = (q8[0] / PATCH_Q8_SIDE).min(PATCHES_PER_REGION_SIDE - 1);
    let patch_z = (q8[1] / PATCH_Q8_SIDE).min(PATCHES_PER_REGION_SIDE - 1);
    let region_x = (region_id % terrain_format::WORLD_REGION_SIDE) as i32;
    let region_z = (region_id / terrain_format::WORLD_REGION_SIDE) as i32;
    let global_patch = (
        region_x * PATCHES_PER_REGION_SIDE as i32 + patch_x as i32,
        region_z * PATCHES_PER_REGION_SIDE as i32 + patch_z as i32,
    );
    let owner_lod = selected_lod(global_patch.0, global_patch.1, camera, settings);
    let x_edge = q8[0].is_multiple_of(PATCH_Q8_SIDE);
    let z_edge = q8[1].is_multiple_of(PATCH_Q8_SIDE);
    if x_edge && z_edge {
        return (height(tile, q8[0] / 256, q8[1] / 256) * 1_024, owner_lod);
    }
    if x_edge {
        let direction = if q8[0] == terrain_format::CELL_SIDE as u32 * 256 {
            1
        } else {
            -1
        };
        let neighbor = (global_patch.0 + direction, global_patch.1);
        let edge_lod = active_patches.get(&neighbor).map_or(owner_lod, |_| {
            selected_lod(neighbor.0, neighbor.1, camera, settings)
        });
        return (
            edge_q18(tile, q8, patch_z, true, owner_lod.max(edge_lod)),
            owner_lod,
        );
    }
    if z_edge {
        let direction = if q8[1] == terrain_format::CELL_SIDE as u32 * 256 {
            1
        } else {
            -1
        };
        let neighbor = (global_patch.0, global_patch.1 + direction);
        let edge_lod = active_patches.get(&neighbor).map_or(owner_lod, |_| {
            selected_lod(neighbor.0, neighbor.1, camera, settings)
        });
        return (
            edge_q18(tile, q8, patch_x, false, owner_lod.max(edge_lod)),
            owner_lod,
        );
    }
    (
        triangle_q18(tile, q8, patch_x, patch_z, owner_lod),
        owner_lod,
    )
}

fn triangle_q18(
    tile: &terrain_format::TerrainTile,
    q8: [u32; 2],
    patch_x: u32,
    patch_z: u32,
    lod: u32,
) -> i32 {
    let step = 1 << lod;
    let denominator = step * 256;
    let patch_origin = [patch_x * PATCH_Q8_SIDE, patch_z * PATCH_Q8_SIDE];
    let cell_x = ((q8[0] - patch_origin[0]) / denominator * step).min(PATCH_CELL_SIDE - step);
    let cell_z = ((q8[1] - patch_origin[1]) / denominator * step).min(PATCH_CELL_SIDE - step);
    let x = patch_x * PATCH_CELL_SIDE + cell_x;
    let z = patch_z * PATCH_CELL_SIDE + cell_z;
    let u = q8[0] - patch_origin[0] - cell_x * 256;
    let v = q8[1] - patch_origin[1] - cell_z * 256;
    let sum = u + v;
    let h00 = height(tile, x, z);
    let h10 = height(tile, x + step, z);
    let h01 = height(tile, x, z + step);
    let h11 = height(tile, x + step, z + step);
    let weighted = if sum <= denominator {
        h00 * (denominator - sum) as i32 + h10 * u as i32 + h01 * v as i32
    } else {
        h10 * (denominator - v) as i32
            + h01 * (denominator - u) as i32
            + h11 * (sum - denominator) as i32
    };
    weighted * (4 / step) as i32
}

fn edge_q18(
    tile: &terrain_format::TerrainTile,
    q8: [u32; 2],
    patch: u32,
    vertical: bool,
    lod: u32,
) -> i32 {
    let step = 1 << lod;
    let denominator = step * 256;
    let along = if vertical { q8[1] } else { q8[0] } - patch * PATCH_Q8_SIDE;
    let base = (along / denominator * step).min(PATCH_CELL_SIDE - step);
    let fraction = along - base * 256;
    let fixed = if vertical { q8[0] / 256 } else { q8[1] / 256 };
    let first = if vertical {
        height(tile, fixed, patch * PATCH_CELL_SIDE + base)
    } else {
        height(tile, patch * PATCH_CELL_SIDE + base, fixed)
    };
    let second = if vertical {
        height(tile, fixed, patch * PATCH_CELL_SIDE + base + step)
    } else {
        height(tile, patch * PATCH_CELL_SIDE + base + step, fixed)
    };
    (first * (denominator - fraction) as i32 + second * fraction as i32) * (4 / step) as i32
}

fn height(tile: &terrain_format::TerrainTile, x: u32, z: u32) -> i32 {
    i32::from(tile.heights[z as usize * terrain_format::SAMPLE_SIDE + x as usize])
}

fn percentile(sorted: &[u32], percent: usize) -> u32 {
    let rank = sorted.len().saturating_mul(percent).div_ceil(100);
    sorted.get(rank.saturating_sub(1)).copied().unwrap_or(0)
}
