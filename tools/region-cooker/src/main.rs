use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use canonical_object_fixture::{Fixture, generate_region as generate_canonical_region};
use region_cooker::{IdentityOrder, reorder_identity_records};
use region_format::{
    GlobalRegion, InstanceRecord, RECORDS_PER_REGION, file_sha256, write_global_identity_pack,
    write_global_pack, write_pack,
};
use serde::Serialize;

const WORLD_REGION_SIDE: u32 = 128;
const ACTIVE_RADIUS: u32 = 2;
const REGION_SIDE_METERS: f32 = 16.0;
const AUTHORITY_REVISION: &str = "canonical-authored-object-authority-q9-v1";

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SignedCookReport {
    schema_version: u32,
    source_revision: &'static str,
    output: String,
    file_sha256: String,
    metadata: region_format::GlobalPackMetadata,
    centers: Vec<GlobalRegion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    identity_order: Option<&'static str>,
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: region-cooker <output.wlr> [--authority] [--identity-order <a|b>] [--global-center <i64> <i64>]...";
    let output = PathBuf::from(args.next().context(usage)?);
    let mut global_centers = Vec::new();
    let mut authority = false;
    let mut identity_order = None;
    while let Some(flag) = args.next() {
        match flag.to_string_lossy().as_ref() {
            "--authority" if !authority => authority = true,
            "--identity-order" if identity_order.is_none() => {
                let value = args.next().context(usage)?;
                identity_order = Some(IdentityOrder::parse(&value.to_string_lossy())?);
            }
            "--global-center" => {
                let x = parse_axis(args.next().context(usage)?, "global x")?;
                let z = parse_axis(args.next().context(usage)?, "global z")?;
                global_centers.push(GlobalRegion::new(x, z));
            }
            _ => bail!(usage),
        }
    }
    if !global_centers.is_empty() {
        return cook_signed(output, global_centers, authority, identity_order);
    }
    anyhow::ensure!(
        !authority && identity_order.is_none(),
        "--authority and --identity-order require at least one --global-center"
    );
    cook_local(output)
}

fn cook_local(output: PathBuf) -> Result<()> {
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

fn cook_signed(
    output: PathBuf,
    centers: Vec<GlobalRegion>,
    authority: bool,
    identity_order: Option<IdentityOrder>,
) -> Result<()> {
    anyhow::ensure!(
        identity_order.is_none() || authority,
        "--identity-order requires --authority"
    );
    let radius = i64::from(ACTIVE_RADIUS);
    let mut regions = BTreeSet::new();
    for center in &centers {
        for offset_z in -radius..=radius {
            for offset_x in -radius..=radius {
                regions.insert(GlobalRegion::new(
                    center
                        .x
                        .checked_add(offset_x)
                        .context("signed object center X expansion overflowed")?,
                    center
                        .z
                        .checked_add(offset_z)
                        .context("signed object center Z expansion overflowed")?,
                ));
            }
        }
    }
    let fixture = Fixture::ArbitraryQ8;
    let regions = regions.into_iter().collect::<Vec<_>>();
    let metadata = if let Some(order) = identity_order {
        let payloads = regions.into_iter().map(|region| {
            let records = generate_authority_region(region, fixture);
            let (records, local_ids) = reorder_identity_records(records, order);
            (region, records, local_ids)
        });
        write_global_identity_pack(&output, fixture.stable_seed_namespace(), payloads)?
    } else {
        let payloads = regions.into_iter().map(|region| {
            let records = if authority {
                generate_authority_region(region, fixture)
            } else {
                generate_canonical_region(region, fixture)
            };
            (region, records)
        });
        write_global_pack(&output, fixture.stable_seed_namespace(), payloads)?
    };
    let report = SignedCookReport {
        schema_version: 1,
        source_revision: identity_order.map_or_else(
            || {
                if authority {
                    AUTHORITY_REVISION
                } else {
                    fixture.revision()
                }
            },
            IdentityOrder::revision,
        ),
        output: output.display().to_string(),
        file_sha256: file_sha256(&output)?,
        metadata,
        centers,
        identity_order: identity_order.map(IdentityOrder::label),
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn generate_authority_region(region: GlobalRegion, fixture: Fixture) -> Vec<InstanceRecord> {
    let mut records = generate_canonical_region(region, fixture);
    for (local_index, record) in records.iter_mut().enumerate() {
        let local_x = local_index as u32 % 32;
        let local_z = local_index as u32 / 32;
        if local_x == 0 || local_x == 31 || local_z == 0 || local_z == 31 {
            continue;
        }
        let key = record.region_id
            ^ (local_index as u32).wrapping_mul(747_796_405)
            ^ (region.x as u32).rotate_left(7)
            ^ (region.z as u32).rotate_right(11);
        let fractions = [32_u32, 96, 160, 224];
        let u = fractions[(key & 3) as usize];
        let v = fractions[((key >> 5) & 3) as usize];
        record.position[0] = -8.0 + (local_x * 256 + u) as f32 / 512.0;
        record.position[2] = -8.0 + (local_z * 256 + v) as f32 / 512.0;
        record.height = 0.65 + ((key.rotate_left(13) & 2047) as f32 / 2047.0) * 2.6;
    }
    records
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
