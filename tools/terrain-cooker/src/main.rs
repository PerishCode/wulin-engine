use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use terrain_format::{
    CELL_SIDE, HEIGHT_COUNT, MATERIAL_COUNT, SAMPLE_SIDE, TerrainTile, WORLD_REGION_SIDE,
    file_sha256, validate_neighbor_edges, write_pack,
};

const ACTIVE_RADIUS: u32 = 2;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CookReport {
    schema_version: u32,
    source_revision: &'static str,
    output: String,
    file_sha256: String,
    metadata: terrain_format::PackMetadata,
    centers: Vec<[u32; 2]>,
    edge_validation: terrain_format::EdgeValidation,
    min_height: i16,
    max_height: i16,
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: terrain-cooker <output.wlt> [--center <x> <z>]...";
    let output = PathBuf::from(args.next().context(usage)?);
    let mut centers = Vec::new();
    while let Some(flag) = args.next() {
        if flag != "--center" {
            bail!(usage);
        }
        let x = parse_center_axis(args.next().context(usage)?, "x")?;
        let z = parse_center_axis(args.next().context(usage)?, "z")?;
        validate_center(x, z)?;
        centers.push([x, z]);
    }
    if centers.is_empty() {
        centers = vec![[64, 64], [65, 64], [65, 65], [96, 96]];
    }
    let mut region_ids = BTreeSet::new();
    for &[center_x, center_z] in &centers {
        for offset_z in 0..=ACTIVE_RADIUS * 2 {
            for offset_x in 0..=ACTIVE_RADIUS * 2 {
                let x = center_x + offset_x - ACTIVE_RADIUS;
                let z = center_z + offset_z - ACTIVE_RADIUS;
                region_ids.insert(z * WORLD_REGION_SIDE + x);
            }
        }
    }
    let tiles = region_ids
        .into_iter()
        .map(generate_tile)
        .collect::<Vec<_>>();
    let edge_validation = validate_neighbor_edges(&tiles);
    if edge_validation.mismatch_count != 0 {
        bail!("generated terrain has mismatched neighboring edges");
    }
    let min_height = tiles
        .iter()
        .flat_map(|tile| tile.heights)
        .min()
        .context("generated terrain has no heights")?;
    let max_height = tiles
        .iter()
        .flat_map(|tile| tile.heights)
        .max()
        .context("generated terrain has no heights")?;
    let metadata = write_pack(&output, tiles)?;
    let report = CookReport {
        schema_version: 1,
        source_revision: "integer-value-noise-v1",
        output: output.display().to_string(),
        file_sha256: file_sha256(&output)?,
        metadata,
        centers,
        edge_validation,
        min_height,
        max_height,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn parse_center_axis(value: std::ffi::OsString, name: &str) -> Result<u32> {
    value
        .to_string_lossy()
        .parse()
        .with_context(|| format!("center {name} must be an unsigned integer"))
}

fn validate_center(x: u32, z: u32) -> Result<()> {
    let minimum = ACTIVE_RADIUS;
    let maximum = WORLD_REGION_SIDE - ACTIVE_RADIUS - 1;
    if !(minimum..=maximum).contains(&x) || !(minimum..=maximum).contains(&z) {
        bail!("center must keep the active radius inside the terrain world");
    }
    Ok(())
}

fn generate_tile(region_id: u32) -> TerrainTile {
    let region_x = region_id % WORLD_REGION_SIDE;
    let region_z = region_id / WORLD_REGION_SIDE;
    let mut heights = [0i16; HEIGHT_COUNT];
    for local_z in 0..SAMPLE_SIDE {
        for local_x in 0..SAMPLE_SIDE {
            let global_x = region_x * CELL_SIDE as u32 + local_x as u32;
            let global_z = region_z * CELL_SIDE as u32 + local_z as u32;
            heights[local_z * SAMPLE_SIDE + local_x] = terrain_height(global_x, global_z);
        }
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    for local_z in 0..CELL_SIDE {
        for local_x in 0..CELL_SIDE {
            let global_x = region_x * CELL_SIDE as u32 + local_x as u32;
            let global_z = region_z * CELL_SIDE as u32 + local_z as u32;
            let height = terrain_height(global_x, global_z);
            materials[local_z * CELL_SIDE + local_x] = material(global_x, global_z, height);
        }
    }
    TerrainTile {
        region_id,
        heights,
        materials,
    }
}

fn terrain_height(global_x: u32, global_z: u32) -> i16 {
    let broad = value_noise(global_x, global_z, 6, 640);
    let detail = value_noise(global_x, global_z, 4, 192);
    (broad + detail).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn value_noise(x: u32, z: u32, shift: u32, amplitude: i32) -> i32 {
    let mask = (1u32 << shift) - 1;
    let cell_x = x >> shift;
    let cell_z = z >> shift;
    let fraction_x = ((x & mask) << (16 - shift)) as i64;
    let fraction_z = ((z & mask) << (16 - shift)) as i64;
    let smooth_x = smooth_q16(fraction_x);
    let smooth_z = smooth_q16(fraction_z);
    let a = lattice(cell_x, cell_z, amplitude);
    let b = lattice(cell_x + 1, cell_z, amplitude);
    let c = lattice(cell_x, cell_z + 1, amplitude);
    let d = lattice(cell_x + 1, cell_z + 1, amplitude);
    let top = lerp_q16(a, b, smooth_x);
    let bottom = lerp_q16(c, d, smooth_x);
    lerp_q16(top, bottom, smooth_z)
}

fn smooth_q16(value: i64) -> i64 {
    let square = (value * value) >> 16;
    (square * (3 * 65_536 - 2 * value)) >> 16
}

fn lerp_q16(a: i32, b: i32, amount: i64) -> i32 {
    (i64::from(a) + ((i64::from(b - a) * amount) >> 16)) as i32
}

fn lattice(x: u32, z: u32, amplitude: i32) -> i32 {
    let mut value = x.wrapping_mul(0x9e37_79b9) ^ z.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    let signed = (value & 0xffff) as i32 - 32_768;
    signed * amplitude / 32_768
}

fn material(global_x: u32, global_z: u32, height: i16) -> u8 {
    let elevation = (i32::from(height) + 1_024).div_euclid(256).clamp(0, 7) as u32;
    ((global_x / 48 + global_z / 40 + elevation) & 7) as u8
}
