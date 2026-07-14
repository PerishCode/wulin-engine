use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

mod global;

pub use global::{
    GLOBAL_HEADER_BYTES, GLOBAL_IDENTITY_PAYLOAD_SCHEMA, GLOBAL_IDENTITY_REGION_BYTES,
    GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_MAGIC, GLOBAL_PAYLOAD_SCHEMA, GLOBAL_VERSION,
    GlobalPackMetadata, GlobalRegion, GlobalRegionPack, GlobalRegionRead, IDENTITY_BYTES,
    IDENTITY_PLANE_BYTES, canonical_stable_seed, write_global_identity_pack, write_global_pack,
};

pub const MAGIC: [u8; 8] = *b"WLRGN001";
pub const VERSION: u32 = 1;
pub const HEADER_BYTES: u32 = 64;
pub const INDEX_ENTRY_BYTES: u32 = 56;
pub const RECORDS_PER_REGION: u32 = 1_024;
pub const RECORD_BYTES: u32 = 20;
pub const REGION_BYTES: u32 = RECORDS_PER_REGION * RECORD_BYTES;
pub const PAYLOAD_ALIGNMENT: u64 = 4_096;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceRecord {
    pub position: [f32; 3],
    pub height: f32,
    pub region_id: u32,
}

const _: [(); RECORD_BYTES as usize] = [(); size_of::<InstanceRecord>()];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackMetadata {
    pub version: u32,
    pub region_count: u32,
    pub index_bytes: u64,
    pub payload_offset: u64,
    pub payload_bytes: u64,
    pub file_bytes: u64,
    pub payload_alignment: u64,
    pub index_sha256: String,
}

#[derive(Clone, Debug)]
struct IndexEntry {
    payload_offset: u64,
    sha256: [u8; 32],
}

pub struct RegionPack {
    file: File,
    metadata: PackMetadata,
    entries: BTreeMap<u32, IndexEntry>,
}

pub struct RegionRead {
    pub region_id: u32,
    pub records: Vec<InstanceRecord>,
    pub payload_bytes: u32,
    pub sha256: String,
    pub read_ms: f64,
    pub verify_ms: f64,
}

impl RegionPack {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("failed to open region pack {}", path.display()))?;
        let actual_file_bytes = file
            .metadata()
            .context("failed to inspect region pack")?
            .len();
        let mut header = [0u8; HEADER_BYTES as usize];
        file.read_exact(&mut header)
            .context("region pack header is truncated")?;
        let mut index_hash = Sha256::new();
        index_hash.update(header);

        ensure!(header[0..8] == MAGIC, "region pack magic is invalid");
        let version = u32_at(&header, 8);
        ensure!(
            version == VERSION,
            "unsupported region pack version {version}"
        );
        ensure!(
            u32_at(&header, 12) == HEADER_BYTES,
            "region pack header size is invalid"
        );
        let region_count = u32_at(&header, 16);
        ensure!(region_count > 0, "region pack contains no regions");
        ensure!(
            u32_at(&header, 20) == INDEX_ENTRY_BYTES,
            "region pack index entry size is invalid"
        );
        ensure!(
            u32_at(&header, 24) == RECORDS_PER_REGION,
            "region pack record count is invalid"
        );
        ensure!(
            u32_at(&header, 28) == RECORD_BYTES,
            "region pack record size is invalid"
        );
        ensure!(
            u64_at(&header, 32) == u64::from(HEADER_BYTES),
            "region pack index offset is invalid"
        );
        ensure!(
            u64_at(&header, 56) == 0,
            "region pack reserved header is nonzero"
        );

        let index_bytes = u64::from(region_count) * u64::from(INDEX_ENTRY_BYTES);
        let expected_payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
        let payload_offset = u64_at(&header, 40);
        ensure!(
            payload_offset == expected_payload_offset,
            "region pack payload offset is invalid"
        );
        let payload_bytes = u64::from(region_count) * u64::from(REGION_BYTES);
        let file_bytes = u64_at(&header, 48);
        ensure!(
            file_bytes == payload_offset + payload_bytes,
            "region pack file size declaration is invalid"
        );
        ensure!(
            actual_file_bytes == file_bytes,
            "region pack file size does not match its header"
        );

        let mut entries = BTreeMap::new();
        let mut previous = None;
        for index in 0..region_count {
            let mut bytes = [0u8; INDEX_ENTRY_BYTES as usize];
            file.read_exact(&mut bytes)
                .context("region pack index is truncated")?;
            index_hash.update(bytes);
            let region_id = u32_at(&bytes, 0);
            if let Some(previous) = previous {
                ensure!(
                    region_id > previous,
                    "region pack IDs are not sorted and unique"
                );
            }
            previous = Some(region_id);
            ensure!(
                u32_at(&bytes, 4) == RECORDS_PER_REGION,
                "region {region_id} record count is invalid"
            );
            let offset = u64_at(&bytes, 8);
            let expected_offset = payload_offset + u64::from(index) * u64::from(REGION_BYTES);
            ensure!(
                offset.is_multiple_of(PAYLOAD_ALIGNMENT),
                "region {region_id} payload is not aligned"
            );
            ensure!(
                offset == expected_offset,
                "region {region_id} payload range is non-canonical"
            );
            ensure!(
                u32_at(&bytes, 16) == REGION_BYTES,
                "region {region_id} payload size is invalid"
            );
            ensure!(
                u32_at(&bytes, 20) == 0,
                "region {region_id} has unknown flags"
            );
            let mut sha256 = [0u8; 32];
            sha256.copy_from_slice(&bytes[24..56]);
            entries.insert(
                region_id,
                IndexEntry {
                    payload_offset: offset,
                    sha256,
                },
            );
        }

        Ok(Self {
            file,
            metadata: PackMetadata {
                version,
                region_count,
                index_bytes,
                payload_offset,
                payload_bytes,
                file_bytes,
                payload_alignment: PAYLOAD_ALIGNMENT,
                index_sha256: format!("{:x}", index_hash.finalize()),
            },
            entries,
        })
    }

    pub fn metadata(&self) -> &PackMetadata {
        &self.metadata
    }

    pub fn contains(&self, region_id: u32) -> bool {
        self.entries.contains_key(&region_id)
    }

    pub fn region_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.entries.keys().copied()
    }

    pub fn read_region(&mut self, region_id: u32) -> Result<RegionRead> {
        let entry = self
            .entries
            .get(&region_id)
            .with_context(|| format!("region {region_id} is absent from the cooked pack"))?
            .clone();
        let read_start = std::time::Instant::now();
        self.file
            .seek(SeekFrom::Start(entry.payload_offset))
            .with_context(|| format!("failed to seek region {region_id}"))?;
        let mut bytes = vec![0u8; REGION_BYTES as usize];
        self.file
            .read_exact(&mut bytes)
            .with_context(|| format!("region {region_id} payload is truncated"))?;
        let read_ms = read_start.elapsed().as_secs_f64() * 1_000.0;
        let verify_start = std::time::Instant::now();
        let actual_sha256 = Sha256::digest(&bytes);
        ensure!(
            actual_sha256.as_slice() == entry.sha256,
            "region {region_id} payload checksum mismatch"
        );

        let mut records = Vec::with_capacity(RECORDS_PER_REGION as usize);
        for bytes in bytes.chunks_exact(RECORD_BYTES as usize) {
            let record = decode_record(bytes);
            ensure!(
                record.region_id == region_id,
                "region {region_id} payload contains record for region {}",
                record.region_id
            );
            ensure!(
                record.position.iter().all(|value| value.is_finite()) && record.height.is_finite(),
                "region {region_id} contains non-finite record data"
            );
            records.push(record);
        }
        Ok(RegionRead {
            region_id,
            records,
            payload_bytes: REGION_BYTES,
            sha256: hex(&entry.sha256),
            read_ms,
            verify_ms: verify_start.elapsed().as_secs_f64() * 1_000.0,
        })
    }
}

pub fn write_pack(
    path: impl AsRef<Path>,
    regions: impl IntoIterator<Item = (u32, Vec<InstanceRecord>)>,
) -> Result<PackMetadata> {
    let path = path.as_ref();
    let mut regions = regions.into_iter().collect::<Vec<_>>();
    regions.sort_by_key(|(region_id, _)| *region_id);
    ensure!(!regions.is_empty(), "cannot write an empty region pack");
    for pair in regions.windows(2) {
        ensure!(pair[0].0 != pair[1].0, "duplicate region ID {}", pair[0].0);
    }

    let region_count = u32::try_from(regions.len()).context("too many regions for pack")?;
    let index_bytes = u64::from(region_count) * u64::from(INDEX_ENTRY_BYTES);
    let payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
    let payload_bytes = u64::from(region_count) * u64::from(REGION_BYTES);
    let file_bytes = payload_offset + payload_bytes;

    let mut encoded = Vec::with_capacity(regions.len());
    for (region_id, records) in &regions {
        ensure!(
            records.len() == RECORDS_PER_REGION as usize,
            "region {region_id} must contain {RECORDS_PER_REGION} records"
        );
        let mut bytes = Vec::with_capacity(REGION_BYTES as usize);
        for record in records {
            ensure!(
                record.region_id == *region_id,
                "region {region_id} contains a mismatched record"
            );
            ensure!(
                record.position.iter().all(|value| value.is_finite()) && record.height.is_finite(),
                "region {region_id} contains non-finite record data"
            );
            encode_record(record, &mut bytes);
        }
        let sha256: [u8; 32] = Sha256::digest(&bytes).into();
        debug_assert_ne!(sha256, [0; 32]);
        encoded.push((bytes, sha256));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = File::create(path)
        .with_context(|| format!("failed to create region pack {}", path.display()))?;
    let mut header = Vec::with_capacity(HEADER_BYTES as usize);
    header.extend_from_slice(&MAGIC);
    push_u32(&mut header, VERSION);
    push_u32(&mut header, HEADER_BYTES);
    push_u32(&mut header, region_count);
    push_u32(&mut header, INDEX_ENTRY_BYTES);
    push_u32(&mut header, RECORDS_PER_REGION);
    push_u32(&mut header, RECORD_BYTES);
    push_u64(&mut header, u64::from(HEADER_BYTES));
    push_u64(&mut header, payload_offset);
    push_u64(&mut header, file_bytes);
    push_u64(&mut header, 0);
    file.write_all(&header)
        .context("failed to write pack header")?;

    for (index, ((region_id, _), (_, sha256))) in regions.iter().zip(&encoded).enumerate() {
        push_u32_to(&mut file, *region_id)?;
        push_u32_to(&mut file, RECORDS_PER_REGION)?;
        push_u64_to(
            &mut file,
            payload_offset + index as u64 * u64::from(REGION_BYTES),
        )?;
        push_u32_to(&mut file, REGION_BYTES)?;
        push_u32_to(&mut file, 0)?;
        file.write_all(sha256)?;
    }
    let position = file.stream_position()?;
    ensure!(
        position <= payload_offset,
        "pack index exceeded payload offset"
    );
    file.write_all(&vec![0u8; (payload_offset - position) as usize])?;
    for (bytes, _) in encoded {
        file.write_all(&bytes)?;
    }
    file.flush().context("failed to flush region pack")?;
    drop(file);

    let pack = RegionPack::open(path)?;
    Ok(pack.metadata().clone())
}

fn encode_record(record: &InstanceRecord, output: &mut Vec<u8>) {
    for value in record.position {
        output.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    output.extend_from_slice(&record.height.to_bits().to_le_bytes());
    output.extend_from_slice(&record.region_id.to_le_bytes());
}

fn decode_record(bytes: &[u8]) -> InstanceRecord {
    InstanceRecord {
        position: [
            f32::from_bits(u32_at(bytes, 0)),
            f32::from_bits(u32_at(bytes, 4)),
            f32::from_bits(u32_at(bytes, 8)),
        ],
        height: f32::from_bits(u32_at(bytes, 12)),
        region_id: u32_at(bytes, 16),
    }
}

pub fn file_sha256(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = [0u8; 64 * 1_024];
    loop {
        let read = file.read(&mut bytes)?;
        if read == 0 {
            break;
        }
        hash.update(&bytes[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn align_up(value: u64) -> u64 {
    value.div_ceil(PAYLOAD_ALIGNMENT) * PAYLOAD_ALIGNMENT
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("u64 slice"))
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_to(writer: &mut impl Write, value: u32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn push_u64_to(writer: &mut impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
