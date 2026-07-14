use std::fs;

use region_format::{
    GLOBAL_HEADER_BYTES, GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_PAYLOAD_SCHEMA, GlobalRegion,
    GlobalRegionPack, InstanceRecord, RECORDS_PER_REGION, REGION_BYTES, canonical_stable_seed,
    write_global_pack,
};
use sha2::{Digest, Sha256};

fn namespace() -> [u8; 32] {
    Sha256::digest(b"canonical-generated-object-arbitrary-q8-v1").into()
}

fn records(region: GlobalRegion, variant: u32) -> Vec<InstanceRecord> {
    let stable_seed = canonical_stable_seed(namespace(), region);
    (0..RECORDS_PER_REGION)
        .map(|index| InstanceRecord {
            position: [
                -8.0 + (index % 32) as f32 * 0.5,
                0.0,
                -8.0 + (index / 32) as f32 * 0.5,
            ],
            height: 0.7 + (index ^ variant) as f32 / RECORDS_PER_REGION as f32,
            region_id: stable_seed,
        })
        .collect()
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "wulin-global-objects-{name}-{}.wlr",
        std::process::id()
    ))
}

#[test]
fn signed_pack_round_trips_and_is_deterministic() {
    let path = temp_path("round-trip");
    let second = temp_path("deterministic");
    let far = 1_i64 << 40;
    let regions = [
        GlobalRegion::new(far, -far),
        GlobalRegion::new(far + 1, -far),
        GlobalRegion::new(-far, far),
    ];
    let contents = regions
        .into_iter()
        .map(|region| (region, records(region, 0)))
        .collect::<Vec<_>>();
    let first_metadata = write_global_pack(&path, namespace(), contents.clone()).unwrap();
    let second_metadata = write_global_pack(&second, namespace(), contents.clone()).unwrap();
    assert_eq!(first_metadata.version, 2);
    assert_eq!(first_metadata.payload_schema, GLOBAL_PAYLOAD_SCHEMA);
    assert_eq!(
        first_metadata.stable_seed_namespace_sha256,
        second_metadata.stable_seed_namespace_sha256
    );
    assert_eq!(
        first_metadata.source_namespace_sha256,
        second_metadata.source_namespace_sha256
    );
    assert_eq!(fs::read(&path).unwrap(), fs::read(&second).unwrap());

    let mut pack = GlobalRegionPack::open(&path).unwrap();
    assert_eq!(pack.metadata().region_count, 3);
    assert_eq!(pack.stable_seed_namespace(), namespace());
    for (region, expected) in contents {
        let read = pack.read_region(region).unwrap();
        assert_eq!(read.region, region);
        assert_eq!(read.stable_seed, canonical_stable_seed(namespace(), region));
        assert_eq!(read.payload_bytes, REGION_BYTES);
        assert_eq!(read.records, expected);
        assert_eq!(read.payload.len(), REGION_BYTES as usize);
    }
    fs::remove_file(path).unwrap();
    fs::remove_file(second).unwrap();
}

#[test]
fn complete_index_changes_source_without_changing_seed_identity() {
    let first = temp_path("source-a");
    let second = temp_path("source-b");
    let region = GlobalRegion::new(-7, 11);
    let first_metadata =
        write_global_pack(&first, namespace(), [(region, records(region, 0))]).unwrap();
    let second_metadata =
        write_global_pack(&second, namespace(), [(region, records(region, 1))]).unwrap();
    assert_eq!(
        first_metadata.stable_seed_namespace_sha256,
        second_metadata.stable_seed_namespace_sha256
    );
    assert_ne!(
        first_metadata.source_namespace_sha256,
        second_metadata.source_namespace_sha256
    );
    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();
}

#[test]
fn signed_metadata_padding_checksum_and_payload_are_rejected() {
    let source = temp_path("source");
    let far = 1_i64 << 40;
    let first = GlobalRegion::new(far, -far);
    let second = GlobalRegion::new(far + 1, -far);
    write_global_pack(
        &source,
        namespace(),
        [(first, records(first, 0)), (second, records(second, 0))],
    )
    .unwrap();
    let bytes = fs::read(&source).unwrap();
    let index_start = GLOBAL_HEADER_BYTES as usize;
    let payload_start = 4_096usize;

    let open_failures = [
        ("magic", 0usize),
        ("version", 8),
        ("schema", 56),
        ("offset", index_start + 16),
        (
            "padding",
            index_start + GLOBAL_INDEX_ENTRY_BYTES as usize * 2,
        ),
    ];
    for (name, offset) in open_failures {
        let path = temp_path(name);
        let mut malformed = bytes.clone();
        malformed[offset] ^= 1;
        fs::write(&path, malformed).unwrap();
        assert!(GlobalRegionPack::open(&path).is_err(), "{name}");
        fs::remove_file(path).unwrap();
    }

    let duplicate = temp_path("duplicate");
    let mut malformed = bytes.clone();
    let second_key = index_start + GLOBAL_INDEX_ENTRY_BYTES as usize;
    let first_key = malformed[index_start..index_start + 16].to_vec();
    malformed[second_key..second_key + 16].copy_from_slice(&first_key);
    fs::write(&duplicate, malformed).unwrap();
    assert!(GlobalRegionPack::open(&duplicate).is_err());
    fs::remove_file(duplicate).unwrap();

    for (name, offset) in [("checksum", index_start + 32), ("payload", payload_start)] {
        let path = temp_path(name);
        let mut malformed = bytes.clone();
        malformed[offset] ^= 1;
        fs::write(&path, malformed).unwrap();
        let mut pack = GlobalRegionPack::open(&path).unwrap();
        assert!(pack.read_region(first).is_err(), "{name}");
        fs::remove_file(path).unwrap();
    }

    fs::remove_file(source).unwrap();
}

#[test]
fn writer_rejects_invalid_seed_identity_and_empty_namespace() {
    let path = temp_path("invalid-seed");
    let region = GlobalRegion::new(3, -9);
    let mut invalid = records(region, 0);
    invalid[0].region_id ^= 1;
    assert!(write_global_pack(&path, namespace(), [(region, invalid)]).is_err());
    assert!(write_global_pack(&path, [0; 32], [(region, records(region, 0))]).is_err());
}
