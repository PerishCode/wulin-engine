use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use terrain_format::{
    CELL_SIDE, GlobalRegion, GlobalTerrainTile, HEIGHT_COUNT, MATERIAL_COUNT, SAMPLE_SIDE,
    file_sha256, validate_global_neighbor_edges, write_global_pack,
};

const ACTIVE_RADIUS: u32 = 2;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CookReport {
    schema_version: u32,
    source_revision: &'static str,
    variant: u32,
    output: String,
    file_sha256: String,
    metadata: terrain_format::GlobalPackMetadata,
    centers: Vec<GlobalRegion>,
    edge_validation: terrain_format::GlobalEdgeValidation,
    min_height: i16,
    max_height: i16,
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: terrain-cooker <output.wlt> --global-center <i64> <i64> [--global-center <i64> <i64>]... [--variant <u32>]";
    let output = PathBuf::from(args.next().context(usage)?);
    let mut global_centers = Vec::new();
    let mut variant = 0;
    while let Some(flag) = args.next() {
        match flag.to_string_lossy().as_ref() {
            "--global-center" => {
                let x = parse_axis(args.next().context(usage)?, "global x")?;
                let z = parse_axis(args.next().context(usage)?, "global z")?;
                global_centers.push(GlobalRegion::new(x, z));
            }
            "--variant" => {
                variant = parse_axis(args.next().context(usage)?, "variant")?;
            }
            _ => bail!(usage),
        }
    }
    anyhow::ensure!(!global_centers.is_empty(), usage);
    cook(output, global_centers, variant)
}

fn cook(output: PathBuf, centers: Vec<GlobalRegion>, variant: u32) -> Result<()> {
    let radius = i64::from(ACTIVE_RADIUS);
    let mut regions = BTreeSet::new();
    for center in &centers {
        for offset_z in -radius..=radius {
            for offset_x in -radius..=radius {
                regions.insert(GlobalRegion::new(
                    center
                        .x
                        .checked_add(offset_x)
                        .context("signed terrain center X expansion overflowed")?,
                    center
                        .z
                        .checked_add(offset_z)
                        .context("signed terrain center Z expansion overflowed")?,
                ));
            }
        }
    }
    let tiles = regions
        .into_iter()
        .map(|region| generate_signed_tile(region, variant))
        .collect::<Vec<_>>();
    let edge_validation = validate_global_neighbor_edges(&tiles);
    if edge_validation.mismatch_count != 0 {
        bail!("generated signed terrain has mismatched neighboring edges");
    }
    let min_height = tiles
        .iter()
        .flat_map(|tile| tile.heights)
        .min()
        .context("generated signed terrain has no heights")?;
    let max_height = tiles
        .iter()
        .flat_map(|tile| tile.heights)
        .max()
        .context("generated signed terrain has no heights")?;
    let metadata = write_global_pack(&output, tiles)?;
    let report = CookReport {
        schema_version: 2,
        source_revision: "signed-integer-value-noise-v1",
        variant,
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

fn parse_axis<T>(value: std::ffi::OsString, name: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    value
        .to_string_lossy()
        .parse()
        .with_context(|| format!("{name} is invalid"))
}

fn generate_signed_tile(region: GlobalRegion, variant: u32) -> GlobalTerrainTile {
    let mut heights = [0i16; HEIGHT_COUNT];
    for local_z in 0..SAMPLE_SIDE {
        for local_x in 0..SAMPLE_SIDE {
            let global_x = i128::from(region.x) * CELL_SIDE as i128 + local_x as i128;
            let global_z = i128::from(region.z) * CELL_SIDE as i128 + local_z as i128;
            heights[local_z * SAMPLE_SIDE + local_x] =
                signed_terrain_height(global_x, global_z, variant);
        }
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    for local_z in 0..CELL_SIDE {
        for local_x in 0..CELL_SIDE {
            let global_x = i128::from(region.x) * CELL_SIDE as i128 + local_x as i128;
            let global_z = i128::from(region.z) * CELL_SIDE as i128 + local_z as i128;
            let height = signed_terrain_height(global_x, global_z, variant);
            materials[local_z * CELL_SIDE + local_x] =
                signed_material(global_x, global_z, height, variant);
        }
    }
    GlobalTerrainTile {
        region,
        heights,
        materials,
    }
}

fn signed_terrain_height(global_x: i128, global_z: i128, variant: u32) -> i16 {
    let broad = signed_value_noise(global_x, global_z, 6, 640, variant);
    let detail = signed_value_noise(global_x, global_z, 4, 192, variant);
    (broad + detail).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn signed_value_noise(x: i128, z: i128, shift: u32, amplitude: i32, variant: u32) -> i32 {
    let side = 1_i128 << shift;
    let cell_x = x.div_euclid(side);
    let cell_z = z.div_euclid(side);
    let fraction_x = (x.rem_euclid(side) << (16 - shift)) as i64;
    let fraction_z = (z.rem_euclid(side) << (16 - shift)) as i64;
    let smooth_x = smooth_q16(fraction_x);
    let smooth_z = smooth_q16(fraction_z);
    let a = signed_lattice(cell_x, cell_z, amplitude, variant);
    let b = signed_lattice(cell_x + 1, cell_z, amplitude, variant);
    let c = signed_lattice(cell_x, cell_z + 1, amplitude, variant);
    let d = signed_lattice(cell_x + 1, cell_z + 1, amplitude, variant);
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

fn signed_lattice(x: i128, z: i128, amplitude: i32, variant: u32) -> i32 {
    let mut value = 0xcbf2_9ce4_8422_2325_u64;
    for byte in x
        .to_le_bytes()
        .into_iter()
        .chain(z.to_le_bytes())
        .chain(variant.to_le_bytes())
    {
        value ^= u64::from(byte);
        value = value.wrapping_mul(0x100_0000_01b3);
    }
    value ^= value >> 32;
    value = value.wrapping_mul(0xd6e8_feb8_6659_fd93);
    value ^= value >> 32;
    let signed = (value & 0xffff) as i32 - 32_768;
    signed * amplitude / 32_768
}

fn signed_material(global_x: i128, global_z: i128, height: i16, variant: u32) -> u8 {
    let elevation = (i32::from(height) + 1_024).div_euclid(256).clamp(0, 7) as i128;
    (global_x.div_euclid(48) + global_z.div_euclid(40) + elevation + i128::from(variant))
        .rem_euclid(8) as u8
}
