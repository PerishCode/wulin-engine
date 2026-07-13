use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use region_format::{InstanceRecord, RECORDS_PER_REGION, file_sha256, write_pack};
use serde::Serialize;

const WORLD_REGION_SIDE: u32 = 128;
const ACTIVE_RADIUS: u32 = 2;
const REGION_SIDE_METERS: f32 = 16.0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CookReport {
    schema_version: u32,
    source_revision: &'static str,
    output: String,
    file_sha256: String,
    metadata: region_format::PackMetadata,
    centers: [[u32; 2]; 4],
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let output = PathBuf::from(args.next().context("usage: region-cooker <output.wlr>")?);
    if args.next().is_some() {
        bail!("usage: region-cooker <output.wlr>");
    }
    let centers = [[64, 64], [65, 64], [65, 65], [96, 96]];
    let mut region_ids = BTreeSet::new();
    for [center_x, center_z] in centers {
        for offset_z in 0..=ACTIVE_RADIUS * 2 {
            for offset_x in 0..=ACTIVE_RADIUS * 2 {
                let x = center_x + offset_x - ACTIVE_RADIUS;
                let z = center_z + offset_z - ACTIVE_RADIUS;
                region_ids.insert(z * WORLD_REGION_SIDE + x);
            }
        }
    }
    let regions = region_ids
        .into_iter()
        .map(|region_id| (region_id, generate_region(region_id)));
    let metadata = write_pack(&output, regions)?;
    let file_sha256 = file_sha256(&output)?;
    let report = CookReport {
        schema_version: 1,
        source_revision: "procedural-region-v1",
        output: output.display().to_string(),
        file_sha256,
        metadata,
        centers,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn generate_region(region_id: u32) -> Vec<InstanceRecord> {
    let region_x = region_id % WORLD_REGION_SIDE;
    let region_z = region_id / WORLD_REGION_SIDE;
    (0..RECORDS_PER_REGION)
        .map(|local_index| {
            let local_x = local_index % 32;
            let local_z = local_index / 32;
            let position_x = (region_x as i32 - 64) as f32 * REGION_SIDE_METERS
                + ((local_x as f32 + 0.5) / 32.0 - 0.5) * REGION_SIDE_METERS;
            let position_z = (region_z as i32 - 64) as f32 * REGION_SIDE_METERS
                + ((local_z as f32 + 0.5) / 32.0 - 0.5) * REGION_SIDE_METERS;
            let reference = region_id * RECORDS_PER_REGION + local_index;
            InstanceRecord {
                position: [position_x, 0.0, position_z],
                height: instance_height(reference),
                region_id,
            }
        })
        .collect()
}

fn instance_height(reference: u32) -> f32 {
    let mut value = reference
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    value = (value >> 22) ^ value;
    0.7 + (value & 1023) as f32 / 1023.0 * 2.3
}
