use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use canonical_object_fixture::{
    generate_region as generate_canonical_region, stable_seed_namespace,
};
use region_cooker::{IdentityOrder, reorder_identity_records};
use region_format::{GlobalRegion, InstanceRecord, file_sha256, write_global_pack};
use serde::Serialize;

const ACTIVE_RADIUS: u32 = 2;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CookReport {
    schema_version: u32,
    source_revision: &'static str,
    output: String,
    file_sha256: String,
    metadata: region_format::GlobalPackMetadata,
    centers: Vec<GlobalRegion>,
    identity_order: &'static str,
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = "usage: region-cooker <output.wlr> --identity-order <a|b> --global-center <i64> <i64> [--global-center <i64> <i64>]...";
    let output = PathBuf::from(args.next().context(usage)?);
    let mut global_centers = Vec::new();
    let mut identity_order = None;
    while let Some(flag) = args.next() {
        match flag.to_string_lossy().as_ref() {
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
    anyhow::ensure!(!global_centers.is_empty(), usage);
    let identity_order = identity_order.context(usage)?;
    cook(output, global_centers, identity_order)
}

fn cook(output: PathBuf, centers: Vec<GlobalRegion>, identity_order: IdentityOrder) -> Result<()> {
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
    let payloads = regions.into_iter().map(|region| {
        let records = generate_authority_region(region);
        let (records, local_ids) = reorder_identity_records(records, identity_order);
        (region, records, local_ids)
    });
    let metadata = write_global_pack(&output, stable_seed_namespace(), payloads)?;
    let report = CookReport {
        schema_version: 2,
        source_revision: identity_order.revision(),
        output: output.display().to_string(),
        file_sha256: file_sha256(&output)?,
        metadata,
        centers,
        identity_order: identity_order.label(),
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn generate_authority_region(region: GlobalRegion) -> Vec<InstanceRecord> {
    let mut records = generate_canonical_region(region);
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
