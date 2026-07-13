use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use region_format::*;
use sha2::{Digest, Sha256};

static NEXT_PATH: AtomicU64 = AtomicU64::new(1);

#[test]
fn canonical_round_trip() -> Result<()> {
    let path = test_path("round-trip");
    let metadata = write_pack(&path, [(9, records(9)), (3, records(3))])?;
    assert_eq!(metadata.region_count, 2);
    assert_eq!(metadata.payload_bytes, 2 * u64::from(REGION_BYTES));
    let mut pack = RegionPack::open(&path)?;
    assert_eq!(pack.read_region(3)?.records, records(3));
    assert_eq!(pack.read_region(9)?.records, records(9));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn malformed_metadata_fails() -> Result<()> {
    let source = test_path("source");
    write_pack(&source, [(3, records(3)), (9, records(9))])?;
    let original = fs::read(&source)?;

    reject_u32(&original, "version", 8, 2)?;
    reject_u32(&original, "header-size", 12, 63)?;
    reject_u32(&original, "index-size", 20, 55)?;
    reject_u32(&original, "record-count", 24, 1)?;
    reject_u32(&original, "record-size", 28, 24)?;
    reject_u32(&original, "entry-count", HEADER_BYTES as usize + 4, 1)?;
    reject_u32(&original, "payload-size", HEADER_BYTES as usize + 16, 1)?;
    reject_u32(&original, "flags", HEADER_BYTES as usize + 20, 1)?;
    reject_u64(&original, "index-offset", 32, 0)?;
    reject_u64(&original, "payload-offset", 40, 0)?;
    reject_u64(&original, "declared-size", 48, 1)?;
    reject_u64(&original, "reserved", 56, 1)?;
    reject_u64(&original, "unaligned", HEADER_BYTES as usize + 8, 1)?;

    let magic = test_path("magic");
    let mut bytes = original.clone();
    bytes[0] = b'X';
    fs::write(&magic, bytes)?;
    assert!(RegionPack::open(&magic).is_err());
    fs::remove_file(magic)?;

    let duplicate = test_path("duplicate");
    let mut bytes = original.clone();
    let second = HEADER_BYTES as usize + INDEX_ENTRY_BYTES as usize;
    bytes[second..second + 4].copy_from_slice(&3u32.to_le_bytes());
    fs::write(&duplicate, bytes)?;
    assert!(RegionPack::open(&duplicate).is_err());
    fs::remove_file(duplicate)?;

    let unsorted = test_path("unsorted");
    let mut bytes = original.clone();
    bytes[HEADER_BYTES as usize..HEADER_BYTES as usize + 4].copy_from_slice(&10u32.to_le_bytes());
    fs::write(&unsorted, bytes)?;
    assert!(RegionPack::open(&unsorted).is_err());
    fs::remove_file(unsorted)?;

    let overlap = test_path("overlap");
    let mut bytes = original.clone();
    let first_offset = bytes[HEADER_BYTES as usize + 8..HEADER_BYTES as usize + 16].to_vec();
    bytes[second + 8..second + 16].copy_from_slice(&first_offset);
    fs::write(&overlap, bytes)?;
    assert!(RegionPack::open(&overlap).is_err());
    fs::remove_file(overlap)?;

    let truncated = test_path("truncated");
    fs::write(&truncated, &original[..original.len() - 1])?;
    assert!(RegionPack::open(&truncated).is_err());
    fs::remove_file(truncated)?;
    fs::remove_file(source)?;
    Ok(())
}

#[test]
fn invalid_payloads_fail() -> Result<()> {
    let source = test_path("payload-source");
    let metadata = write_pack(&source, [(3, records(3))])?;
    let original = fs::read(&source)?;

    let corrupt = test_path("checksum");
    let mut bytes = original.clone();
    bytes[metadata.payload_offset as usize] ^= 1;
    fs::write(&corrupt, bytes)?;
    assert!(RegionPack::open(&corrupt)?.read_region(3).is_err());
    fs::remove_file(corrupt)?;

    let wrong_id = test_path("wrong-id");
    let mut bytes = original;
    let region_offset = metadata.payload_offset as usize + 16;
    bytes[region_offset..region_offset + 4].copy_from_slice(&4u32.to_le_bytes());
    let digest = Sha256::digest(
        &bytes[metadata.payload_offset as usize
            ..metadata.payload_offset as usize + REGION_BYTES as usize],
    );
    bytes[HEADER_BYTES as usize + 24..HEADER_BYTES as usize + 56].copy_from_slice(&digest);
    fs::write(&wrong_id, bytes)?;
    assert!(RegionPack::open(&wrong_id)?.read_region(3).is_err());
    fs::remove_file(wrong_id)?;
    fs::remove_file(source)?;
    Ok(())
}

fn reject_u32(source: &[u8], label: &str, offset: usize, value: u32) -> Result<()> {
    let path = test_path(label);
    let mut bytes = source.to_vec();
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    fs::write(&path, bytes)?;
    assert!(RegionPack::open(&path).is_err(), "{label} was accepted");
    fs::remove_file(path)?;
    Ok(())
}

fn reject_u64(source: &[u8], label: &str, offset: usize, value: u64) -> Result<()> {
    let path = test_path(label);
    let mut bytes = source.to_vec();
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    fs::write(&path, bytes)?;
    assert!(RegionPack::open(&path).is_err(), "{label} was accepted");
    fs::remove_file(path)?;
    Ok(())
}

fn records(region_id: u32) -> Vec<InstanceRecord> {
    (0..RECORDS_PER_REGION)
        .map(|index| InstanceRecord {
            position: [index as f32, 0.0, region_id as f32],
            height: 1.0 + index as f32 / RECORDS_PER_REGION as f32,
            region_id,
        })
        .collect()
}

fn test_path(label: &str) -> std::path::PathBuf {
    let id = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "wulin-region-format-{label}-{}-{id}.wlr",
        std::process::id()
    ))
}
