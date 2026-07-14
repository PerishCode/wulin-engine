use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::async_resident::{ObjectSourceNamespace, canonical_stable_seed};
use crate::rendering::async_resident::ActivePayloadReadback;
use crate::rendering::terrain::TerrainProjection;
use crate::resident::InstanceRecord;
use crate::terrain::TerrainAssignment;
use crate::world::RegionCoord;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CanonicalObjectEvidence {
    revision: &'static str,
    source_namespace: ObjectSourceNamespace,
    entry_count: usize,
    semantic_collision_count: usize,
    stable_seed_collision_count: usize,
    mismatch_count: usize,
    local_id_count: usize,
    local_id_duplicate_count: usize,
    content_sha256: String,
    identity_keyed_sha256: String,
    stable_key_sha256: String,
    stable_seed_sha256: String,
    payload_authority: CookedPayloadAuthority,
    entries: Vec<CanonicalObjectEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CookedPayloadAuthority {
    revision: &'static str,
    payload_schema: u32,
    region_count: usize,
    record_count: usize,
    copy_count: u32,
    readback_bytes: u64,
    allocation_bytes: u64,
    probe_count: u64,
    total_copy_count: u64,
    chunk_mismatch_count: usize,
    expected_index_sha256: String,
    observed_index_sha256: String,
    payload_sha256: String,
    record_readback_bytes: u64,
    record_allocation_bytes: u64,
    record_copy_count: u32,
    identity_readback_bytes: u64,
    identity_allocation_bytes: u64,
    identity_copy_count: u32,
    identity_probe_count: u64,
    identity_total_copy_count: u64,
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

pub(super) fn canonical_object_evidence(
    source_namespace: ObjectSourceNamespace,
    stable_seed_namespace: ObjectSourceNamespace,
    assignments: &[TerrainAssignment],
    records: &[Vec<InstanceRecord>],
    projection: TerrainProjection,
    readback: &ActivePayloadReadback,
) -> Result<CanonicalObjectEvidence> {
    ensure!(
        assignments.len() == records.len(),
        "canonical object evidence requires one projected payload per region"
    );
    let mut content_hash = Sha256::new();
    let mut identity_keyed_hash = Sha256::new();
    let mut stable_key_hash = Sha256::new();
    let mut seed_hash = Sha256::new();
    let mut semantic_ids = BTreeSet::new();
    let mut stable_seeds = BTreeSet::new();
    let mut mismatch_count = 0;
    let mut local_id_count = 0;
    let mut local_id_duplicate_count = 0;
    let mut entries = Vec::with_capacity(assignments.len());
    for (index, (assignment, region_records)) in assignments.iter().zip(records).enumerate() {
        let local_ids = &readback.local_ids[index];
        ensure!(
            local_ids.len() == region_records.len(),
            "canonical object identity page differs from its record page"
        );
        let global = assignment.global_region;
        let stable_seed = canonical_stable_seed(stable_seed_namespace, global);
        let semantic_region_id = projection.region_id(index)?;
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
        let unique_local_ids = local_ids.iter().copied().collect::<BTreeSet<_>>();
        local_id_count += local_ids.len();
        local_id_duplicate_count += local_ids.len() - unique_local_ids.len();
        let mut keyed = local_ids
            .iter()
            .copied()
            .zip(region_records.iter())
            .collect::<Vec<_>>();
        keyed.sort_by_key(|(local_id, _)| *local_id);
        identity_keyed_hash.update(global.x.to_le_bytes());
        identity_keyed_hash.update(global.z.to_le_bytes());
        for (local_id, record) in keyed {
            identity_keyed_hash.update(local_id.to_le_bytes());
            identity_keyed_hash.update(crate::resident::as_bytes(std::slice::from_ref(record)));
            stable_key_hash
                .update(crate::resident::canonical_stable_key(stable_seed, local_id).to_le_bytes());
        }
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
        revision: "canonical-object-authority-v1",
        source_namespace,
        entry_count: entries.len(),
        semantic_collision_count: entries.len() - semantic_ids.len(),
        stable_seed_collision_count: entries.len() - stable_seeds.len(),
        mismatch_count,
        local_id_count,
        local_id_duplicate_count,
        content_sha256: format!("{:x}", content_hash.finalize()),
        identity_keyed_sha256: format!("{:x}", identity_keyed_hash.finalize()),
        stable_key_sha256: format!("{:x}", stable_key_hash.finalize()),
        stable_seed_sha256: format!("{:x}", seed_hash.finalize()),
        payload_authority: cooked_payload_authority(assignments, records, readback)?,
        entries,
    })
}

fn cooked_payload_authority(
    assignments: &[TerrainAssignment],
    records: &[Vec<InstanceRecord>],
    readback: &ActivePayloadReadback,
) -> Result<CookedPayloadAuthority> {
    let expected = &readback.expected_checksums;
    ensure!(
        assignments.len() == records.len() && expected.len() == records.len(),
        "cooked payload authority shapes differ"
    );
    let mut expected_index = Sha256::new();
    let mut observed_index = Sha256::new();
    let mut payload = Sha256::new();
    let mut chunk_mismatch_count = 0;
    for (active_index, ((assignment, region_records), expected_checksum)) in
        assignments.iter().zip(records).zip(expected).enumerate()
    {
        let global = assignment.global_region;
        let bytes = crate::resident::as_bytes(region_records);
        let identity_bytes = crate::resident::as_bytes(&readback.local_ids[active_index]);
        let mut combined = Sha256::new();
        combined.update(bytes);
        combined.update(identity_bytes);
        let observed_checksum: [u8; 32] = combined.finalize().into();
        chunk_mismatch_count += usize::from(observed_checksum != *expected_checksum);
        for digest in [&mut expected_index, &mut observed_index] {
            digest.update(global.x.to_le_bytes());
            digest.update(global.z.to_le_bytes());
        }
        expected_index.update(expected_checksum);
        observed_index.update(observed_checksum);
        payload.update(bytes);
        payload.update(identity_bytes);
    }
    ensure!(
        chunk_mismatch_count == 0,
        "published cooked object pages differ from the pack index"
    );
    Ok(CookedPayloadAuthority {
        revision: "cooked-object-payload-authority-v2",
        payload_schema: region_format::GLOBAL_PAYLOAD_SCHEMA,
        region_count: records.len(),
        record_count: records.iter().map(Vec::len).sum(),
        copy_count: readback.copy_count + readback.identity_copy_count,
        readback_bytes: readback.readback_bytes + readback.identity_readback_bytes,
        allocation_bytes: readback.allocation_bytes + readback.identity_allocation_bytes,
        probe_count: readback.probe_count,
        total_copy_count: readback.total_copy_count + readback.identity_total_copy_count,
        chunk_mismatch_count,
        expected_index_sha256: format!("{:x}", expected_index.finalize()),
        observed_index_sha256: format!("{:x}", observed_index.finalize()),
        payload_sha256: format!("{:x}", payload.finalize()),
        record_readback_bytes: readback.readback_bytes,
        record_allocation_bytes: readback.allocation_bytes,
        record_copy_count: readback.copy_count,
        identity_readback_bytes: readback.identity_readback_bytes,
        identity_allocation_bytes: readback.identity_allocation_bytes,
        identity_copy_count: readback.identity_copy_count,
        identity_probe_count: readback.identity_probe_count,
        identity_total_copy_count: readback.identity_total_copy_count,
    })
}
