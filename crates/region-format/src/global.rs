use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    InstanceRecord, PAYLOAD_ALIGNMENT, PRESENTATION_ANIMATION_CLIP_COUNT,
    PRESENTATION_ANIMATION_PHASE_COUNT, PRESENTATION_ARCHETYPE_COUNT, PRESENTATION_BYTES,
    PRESENTATION_MATERIAL_COUNT, PresentationRecord, RECORD_BYTES, RECORDS_PER_REGION,
    REGION_BYTES, align_up, decode_presentation, decode_record, encode_presentation, encode_record,
    hex, push_u32, push_u32_to, push_u64, push_u64_to, u32_at, u64_at,
};

pub const GLOBAL_MAGIC: [u8; 8] = *b"WLRGN003";
pub const GLOBAL_VERSION: u32 = 3;
pub const GLOBAL_HEADER_BYTES: u32 = 96;
pub const GLOBAL_INDEX_ENTRY_BYTES: u32 = 64;
pub const GLOBAL_PAYLOAD_SCHEMA: u32 = 3;
pub const IDENTITY_BYTES: u32 = 4;
pub const IDENTITY_PLANE_BYTES: u32 = RECORDS_PER_REGION * IDENTITY_BYTES;
pub const PRESENTATION_PLANE_BYTES: u32 = RECORDS_PER_REGION * PRESENTATION_BYTES;
pub const GLOBAL_REGION_BYTES: u32 = REGION_BYTES + IDENTITY_PLANE_BYTES + PRESENTATION_PLANE_BYTES;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalRegion {
    pub x: i64,
    pub z: i64,
}

impl GlobalRegion {
    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPackMetadata {
    pub version: u32,
    pub addressing: &'static str,
    pub payload_schema: u32,
    pub region_count: u32,
    pub index_bytes: u64,
    pub payload_offset: u64,
    pub payload_bytes: u64,
    pub file_bytes: u64,
    pub payload_alignment: u64,
    pub stable_seed_namespace_sha256: String,
    pub source_namespace_sha256: String,
}

#[derive(Clone, Debug)]
struct GlobalIndexEntry {
    payload_offset: u64,
    payload_bytes: u32,
    sha256: [u8; 32],
}

pub struct GlobalRegionPack {
    file: File,
    metadata: GlobalPackMetadata,
    stable_seed_namespace: [u8; 32],
    source_namespace: [u8; 32],
    entries: BTreeMap<GlobalRegion, GlobalIndexEntry>,
}

pub struct GlobalRegionRead {
    pub region: GlobalRegion,
    pub stable_seed: u32,
    pub records: Vec<InstanceRecord>,
    pub local_ids: Vec<u32>,
    pub presentations: Vec<PresentationRecord>,
    pub payload: Vec<u8>,
    pub payload_bytes: u32,
    pub sha256: String,
    pub read_ms: f64,
    pub verify_ms: f64,
}

impl GlobalRegionPack {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("failed to open signed object pack {}", path.display()))?;
        let actual_file_bytes = file
            .metadata()
            .context("failed to inspect signed object pack")?
            .len();
        let mut header = [0u8; GLOBAL_HEADER_BYTES as usize];
        file.read_exact(&mut header)
            .context("signed object pack header is truncated")?;
        let mut namespace = Sha256::new();
        namespace.update(header);

        ensure!(
            header[0..8] == GLOBAL_MAGIC,
            "signed object pack magic is invalid"
        );
        let version = u32_at(&header, 8);
        ensure!(
            version == GLOBAL_VERSION,
            "unsupported signed object pack version {version}"
        );
        ensure!(
            u32_at(&header, 12) == GLOBAL_HEADER_BYTES,
            "signed object pack header size is invalid"
        );
        let region_count = u32_at(&header, 16);
        ensure!(region_count > 0, "signed object pack contains no regions");
        ensure!(
            u32_at(&header, 20) == GLOBAL_INDEX_ENTRY_BYTES,
            "signed object pack index entry size is invalid"
        );
        ensure!(
            u32_at(&header, 24) == RECORDS_PER_REGION,
            "signed object pack record count is invalid"
        );
        ensure!(
            u32_at(&header, 28) == RECORD_BYTES,
            "signed object pack record size is invalid"
        );
        ensure!(
            u64_at(&header, 32) == u64::from(GLOBAL_HEADER_BYTES),
            "signed object pack index offset is invalid"
        );
        let payload_schema = u32_at(&header, 56);
        ensure!(
            payload_schema == GLOBAL_PAYLOAD_SCHEMA,
            "unsupported signed object payload schema {payload_schema}"
        );
        let region_payload_bytes = GLOBAL_REGION_BYTES;
        ensure!(
            u32_at(&header, 60) == 0,
            "signed object pack has unknown flags"
        );
        let mut stable_seed_namespace = [0u8; 32];
        stable_seed_namespace.copy_from_slice(&header[64..96]);
        ensure!(
            stable_seed_namespace != [0; 32],
            "signed object stable-seed namespace is zero"
        );

        let index_bytes = u64::from(region_count) * u64::from(GLOBAL_INDEX_ENTRY_BYTES);
        let expected_payload_offset = align_up(u64::from(GLOBAL_HEADER_BYTES) + index_bytes);
        let payload_offset = u64_at(&header, 40);
        ensure!(
            payload_offset == expected_payload_offset,
            "signed object pack payload offset is invalid"
        );
        let payload_bytes = u64::from(region_count) * u64::from(region_payload_bytes);
        let file_bytes = u64_at(&header, 48);
        ensure!(
            file_bytes == payload_offset + payload_bytes,
            "signed object pack file size declaration is invalid"
        );
        ensure!(
            actual_file_bytes == file_bytes,
            "signed object pack file size does not match its header"
        );

        let mut entries = BTreeMap::new();
        let mut previous = None;
        for index in 0..region_count {
            let mut bytes = [0u8; GLOBAL_INDEX_ENTRY_BYTES as usize];
            file.read_exact(&mut bytes)
                .context("signed object pack index is truncated")?;
            namespace.update(bytes);
            let region = GlobalRegion::new(i64_at(&bytes, 0), i64_at(&bytes, 8));
            let order = (region.z, region.x);
            if let Some(previous) = previous {
                ensure!(
                    order > previous,
                    "signed object pack keys are not sorted and unique"
                );
            }
            previous = Some(order);
            let offset = u64_at(&bytes, 16);
            let expected_offset =
                payload_offset + u64::from(index) * u64::from(region_payload_bytes);
            ensure!(
                offset.is_multiple_of(PAYLOAD_ALIGNMENT),
                "signed object region ({},{}) payload is not aligned",
                region.x,
                region.z
            );
            ensure!(
                offset == expected_offset,
                "signed object region ({},{}) payload range is noncanonical",
                region.x,
                region.z
            );
            ensure!(
                u32_at(&bytes, 24) == region_payload_bytes,
                "signed object region ({},{}) payload size is invalid",
                region.x,
                region.z
            );
            ensure!(
                u32_at(&bytes, 28) == 0,
                "signed object region ({},{}) has unknown flags",
                region.x,
                region.z
            );
            let mut sha256 = [0u8; 32];
            sha256.copy_from_slice(&bytes[32..64]);
            entries.insert(
                region,
                GlobalIndexEntry {
                    payload_offset: offset,
                    payload_bytes: region_payload_bytes,
                    sha256,
                },
            );
        }
        let index_end = u64::from(GLOBAL_HEADER_BYTES) + index_bytes;
        let mut padding = vec![0u8; (payload_offset - index_end) as usize];
        file.read_exact(&mut padding)
            .context("signed object pack alignment padding is truncated")?;
        ensure!(
            padding.iter().all(|byte| *byte == 0),
            "signed object pack alignment padding is nonzero"
        );

        let source_namespace: [u8; 32] = namespace.finalize().into();
        Ok(Self {
            file,
            metadata: GlobalPackMetadata {
                version,
                addressing: "signed-region-v1",
                payload_schema,
                region_count,
                index_bytes,
                payload_offset,
                payload_bytes,
                file_bytes,
                payload_alignment: PAYLOAD_ALIGNMENT,
                stable_seed_namespace_sha256: hex(&stable_seed_namespace),
                source_namespace_sha256: hex(&source_namespace),
            },
            stable_seed_namespace,
            source_namespace,
            entries,
        })
    }

    pub fn metadata(&self) -> &GlobalPackMetadata {
        &self.metadata
    }

    pub fn contains(&self, region: GlobalRegion) -> bool {
        self.entries.contains_key(&region)
    }

    pub fn stable_seed_namespace(&self) -> [u8; 32] {
        self.stable_seed_namespace
    }

    pub fn source_namespace(&self) -> [u8; 32] {
        self.source_namespace
    }

    pub fn regions(&self) -> impl Iterator<Item = GlobalRegion> + '_ {
        self.entries.keys().copied()
    }

    pub fn region_sha256(&self, region: GlobalRegion) -> Option<[u8; 32]> {
        self.entries.get(&region).map(|entry| entry.sha256)
    }

    pub fn read_region(&mut self, region: GlobalRegion) -> Result<GlobalRegionRead> {
        let entry = self.entries.get(&region).with_context(|| {
            format!(
                "signed object region ({},{}) is absent from the pack",
                region.x, region.z
            )
        })?;
        let read_start = std::time::Instant::now();
        self.file
            .seek(SeekFrom::Start(entry.payload_offset))
            .with_context(|| {
                format!(
                    "failed to seek signed object region ({},{})",
                    region.x, region.z
                )
            })?;
        let mut payload = vec![0u8; entry.payload_bytes as usize];
        self.file.read_exact(&mut payload).with_context(|| {
            format!(
                "signed object region ({},{}) payload is truncated",
                region.x, region.z
            )
        })?;
        let read_ms = read_start.elapsed().as_secs_f64() * 1_000.0;
        let verify_start = std::time::Instant::now();
        let actual_sha256 = Sha256::digest(&payload);
        ensure!(
            actual_sha256.as_slice() == entry.sha256,
            "signed object region ({},{}) payload checksum mismatch",
            region.x,
            region.z
        );
        let stable_seed = canonical_stable_seed(self.stable_seed_namespace, region);
        let mut records = Vec::with_capacity(RECORDS_PER_REGION as usize);
        for bytes in payload[..REGION_BYTES as usize].chunks_exact(RECORD_BYTES as usize) {
            let record = decode_record(bytes);
            ensure!(
                record.region_id == stable_seed,
                "signed object region ({},{}) payload contains stable seed {} instead of {stable_seed}",
                region.x,
                region.z,
                record.region_id
            );
            validate_record(region, &record)?;
            records.push(record);
        }
        let mut local_ids = Vec::with_capacity(RECORDS_PER_REGION as usize);
        let identity_end = (REGION_BYTES + IDENTITY_PLANE_BYTES) as usize;
        for bytes in
            payload[REGION_BYTES as usize..identity_end].chunks_exact(IDENTITY_BYTES as usize)
        {
            local_ids.push(u32_at(bytes, 0));
        }
        validate_local_ids(region, &local_ids)?;
        let mut presentations = Vec::with_capacity(RECORDS_PER_REGION as usize);
        for bytes in payload[identity_end..].chunks_exact(PRESENTATION_BYTES as usize) {
            presentations.push(decode_presentation(bytes));
        }
        validate_presentations(region, &presentations)?;
        Ok(GlobalRegionRead {
            region,
            stable_seed,
            records,
            local_ids,
            presentations,
            payload,
            payload_bytes: entry.payload_bytes,
            sha256: hex(&entry.sha256),
            read_ms,
            verify_ms: verify_start.elapsed().as_secs_f64() * 1_000.0,
        })
    }
}

pub fn write_global_pack(
    path: impl AsRef<Path>,
    stable_seed_namespace: [u8; 32],
    regions: impl IntoIterator<
        Item = (
            GlobalRegion,
            Vec<InstanceRecord>,
            Vec<u32>,
            Vec<PresentationRecord>,
        ),
    >,
) -> Result<GlobalPackMetadata> {
    let path = path.as_ref();
    ensure!(
        stable_seed_namespace != [0; 32],
        "cannot write a zero stable-seed namespace"
    );
    let mut regions = regions.into_iter().collect::<Vec<_>>();
    regions.sort_by_key(|(region, _, _, _)| (region.z, region.x));
    ensure!(
        !regions.is_empty(),
        "cannot write an empty signed object pack"
    );
    for pair in regions.windows(2) {
        ensure!(
            pair[0].0 != pair[1].0,
            "duplicate signed object region ({},{})",
            pair[0].0.x,
            pair[0].0.z
        );
    }

    let region_count = u32::try_from(regions.len()).context("too many signed object regions")?;
    let index_bytes = u64::from(region_count) * u64::from(GLOBAL_INDEX_ENTRY_BYTES);
    let payload_offset = align_up(u64::from(GLOBAL_HEADER_BYTES) + index_bytes);
    let region_payload_bytes = GLOBAL_REGION_BYTES;
    let payload_bytes = u64::from(region_count) * u64::from(region_payload_bytes);
    let file_bytes = payload_offset + payload_bytes;
    let mut encoded = Vec::with_capacity(regions.len());
    for (region, records, local_ids, presentations) in &regions {
        ensure!(
            records.len() == RECORDS_PER_REGION as usize,
            "signed object region ({},{}) must contain {RECORDS_PER_REGION} records",
            region.x,
            region.z
        );
        let stable_seed = canonical_stable_seed(stable_seed_namespace, *region);
        let mut bytes = Vec::with_capacity(REGION_BYTES as usize);
        for record in records {
            ensure!(
                record.region_id == stable_seed,
                "signed object region ({},{}) contains a mismatched stable seed",
                region.x,
                region.z
            );
            validate_record(*region, record)?;
            encode_record(record, &mut bytes);
        }
        validate_local_ids(*region, local_ids)?;
        for local_id in local_ids {
            bytes.extend_from_slice(&local_id.to_le_bytes());
        }
        validate_presentations(*region, presentations)?;
        for presentation in presentations {
            encode_presentation(presentation, &mut bytes);
        }
        debug_assert_eq!(bytes.len(), region_payload_bytes as usize);
        let sha256: [u8; 32] = Sha256::digest(&bytes).into();
        encoded.push((bytes, sha256));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = File::create(path)
        .with_context(|| format!("failed to create signed object pack {}", path.display()))?;
    let mut header = Vec::with_capacity(GLOBAL_HEADER_BYTES as usize);
    header.extend_from_slice(&GLOBAL_MAGIC);
    push_u32(&mut header, GLOBAL_VERSION);
    push_u32(&mut header, GLOBAL_HEADER_BYTES);
    push_u32(&mut header, region_count);
    push_u32(&mut header, GLOBAL_INDEX_ENTRY_BYTES);
    push_u32(&mut header, RECORDS_PER_REGION);
    push_u32(&mut header, RECORD_BYTES);
    push_u64(&mut header, u64::from(GLOBAL_HEADER_BYTES));
    push_u64(&mut header, payload_offset);
    push_u64(&mut header, file_bytes);
    push_u32(&mut header, GLOBAL_PAYLOAD_SCHEMA);
    push_u32(&mut header, 0);
    header.extend_from_slice(&stable_seed_namespace);
    debug_assert_eq!(header.len(), GLOBAL_HEADER_BYTES as usize);
    file.write_all(&header)
        .context("failed to write signed object pack header")?;

    for (index, ((region, _, _, _), (_, sha256))) in regions.iter().zip(&encoded).enumerate() {
        push_i64_to(&mut file, region.x)?;
        push_i64_to(&mut file, region.z)?;
        push_u64_to(
            &mut file,
            payload_offset + index as u64 * u64::from(region_payload_bytes),
        )?;
        push_u32_to(&mut file, region_payload_bytes)?;
        push_u32_to(&mut file, 0)?;
        file.write_all(sha256)?;
    }
    let position = file.stream_position()?;
    ensure!(
        position <= payload_offset,
        "signed object pack index exceeded payload offset"
    );
    file.write_all(&vec![0u8; (payload_offset - position) as usize])?;
    for (bytes, _) in encoded {
        file.write_all(&bytes)?;
    }
    file.flush().context("failed to flush signed object pack")?;
    drop(file);

    let pack = GlobalRegionPack::open(path)?;
    Ok(pack.metadata().clone())
}

fn validate_local_ids(region: GlobalRegion, local_ids: &[u32]) -> Result<()> {
    ensure!(
        local_ids.len() == RECORDS_PER_REGION as usize,
        "signed object region ({},{}) must contain {RECORDS_PER_REGION} local IDs",
        region.x,
        region.z
    );
    let mut seen = vec![false; RECORDS_PER_REGION as usize];
    for local_id in local_ids {
        ensure!(
            *local_id < RECORDS_PER_REGION,
            "signed object region ({},{}) local ID {local_id} exceeds capacity",
            region.x,
            region.z
        );
        ensure!(
            !std::mem::replace(&mut seen[*local_id as usize], true),
            "signed object region ({},{}) contains duplicate local ID {local_id}",
            region.x,
            region.z
        );
    }
    Ok(())
}

fn validate_presentations(
    region: GlobalRegion,
    presentations: &[PresentationRecord],
) -> Result<()> {
    ensure!(
        presentations.len() == RECORDS_PER_REGION as usize,
        "signed object region ({},{}) must contain {RECORDS_PER_REGION} presentation records",
        region.x,
        region.z
    );
    for presentation in presentations {
        ensure!(
            presentation.archetype < PRESENTATION_ARCHETYPE_COUNT,
            "signed object region ({},{}) contains invalid presentation archetype {}",
            region.x,
            region.z,
            presentation.archetype
        );
        ensure!(
            presentation.material < PRESENTATION_MATERIAL_COUNT,
            "signed object region ({},{}) contains invalid presentation material {}",
            region.x,
            region.z,
            presentation.material
        );
        ensure!(
            presentation.yaw_q16 <= u16::MAX.into(),
            "signed object region ({},{}) contains invalid presentation yaw {}",
            region.x,
            region.z,
            presentation.yaw_q16
        );
        if presentation.is_animated() {
            ensure!(
                presentation.animation_clip().unwrap() < PRESENTATION_ANIMATION_CLIP_COUNT,
                "signed object region ({},{}) contains invalid animation clip",
                region.x,
                region.z
            );
            ensure!(
                presentation.animation_phase_offset().unwrap() < PRESENTATION_ANIMATION_PHASE_COUNT,
                "signed object region ({},{}) contains invalid animation phase offset",
                region.x,
                region.z
            );
        }
    }
    Ok(())
}

pub fn canonical_stable_seed(namespace: [u8; 32], region: GlobalRegion) -> u32 {
    let mut digest = Sha256::new();
    digest.update(namespace);
    digest.update(region.x.to_le_bytes());
    digest.update(region.z.to_le_bytes());
    let hash = digest.finalize();
    u32::from_le_bytes(hash[..4].try_into().expect("SHA-256 prefix has four bytes"))
}

fn validate_record(region: GlobalRegion, record: &InstanceRecord) -> Result<()> {
    ensure!(
        record.position.iter().all(|value| value.is_finite()) && record.height.is_finite(),
        "signed object region ({},{}) contains non-finite record data",
        region.x,
        region.z
    );
    Ok(())
}

fn i64_at(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("i64 slice"))
}

fn push_i64_to(writer: &mut impl Write, value: i64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}
