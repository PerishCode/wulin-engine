use std::fs;

use region_format::{
    GLOBAL_HEADER_BYTES, GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_PAYLOAD_SCHEMA, GLOBAL_REGION_BYTES,
    GlobalRegion, GlobalRegionPack, IDENTITY_PLANE_BYTES, InstanceRecord, PRESENTATION_PLANE_BYTES,
    PresentationRecord, RECORDS_PER_REGION, REGION_BYTES, STATIC_PRESENTATION_ANIMATION,
    canonical_stable_seed, write_global_pack,
};
use sha2::{Digest, Sha256};

fn namespace() -> [u8; 32] {
    Sha256::digest(b"canonical-object-arbitrary-q8-v1").into()
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

fn canonical_payload(
    region: GlobalRegion,
    variant: u32,
) -> (
    GlobalRegion,
    Vec<InstanceRecord>,
    Vec<u32>,
    Vec<PresentationRecord>,
) {
    (
        region,
        records(region, variant),
        (0..RECORDS_PER_REGION).collect(),
        presentations(variant),
    )
}

fn presentations(variant: u32) -> Vec<PresentationRecord> {
    (0..RECORDS_PER_REGION)
        .map(|index| {
            if index.is_multiple_of(4) {
                PresentationRecord::static_object(index % 8, (index + variant) % 64, index * 61)
            } else {
                PresentationRecord::animated(
                    index % 8,
                    (index + variant) % 64,
                    index * 61,
                    (index + 3) % 8,
                    (index + variant) % 64,
                    index ^ variant,
                )
            }
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
        .map(|region| canonical_payload(region, 0))
        .collect::<Vec<_>>();
    let first_metadata = write_global_pack(&path, namespace(), contents.clone()).unwrap();
    let second_metadata = write_global_pack(&second, namespace(), contents.clone()).unwrap();
    assert_eq!(first_metadata.version, 3);
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
    for (region, expected, expected_ids, expected_presentations) in contents {
        let read = pack.read_region(region).unwrap();
        assert_eq!(read.region, region);
        assert_eq!(read.stable_seed, canonical_stable_seed(namespace(), region));
        assert_eq!(read.payload_bytes, GLOBAL_REGION_BYTES);
        assert_eq!(read.records, expected);
        assert_eq!(read.local_ids, expected_ids);
        assert_eq!(read.presentations, expected_presentations);
        assert_eq!(read.payload.len(), GLOBAL_REGION_BYTES as usize);
        assert_eq!(
            pack.region_sha256(region),
            Some(Sha256::digest(&read.payload).into())
        );
    }
    fs::remove_file(path).unwrap();
    fs::remove_file(second).unwrap();
}

#[test]
fn identity_pack_round_trips_reordered_pairs_deterministically() {
    let path = temp_path("identity-round-trip");
    let second = temp_path("identity-deterministic");
    let reordered = temp_path("identity-reordered");
    let region = GlobalRegion::new(1_i64 << 40, -(1_i64 << 40));
    let (records_a, local_ids_a, presentations_a) = permuted_records(region, 73);
    let (records_b, local_ids_b, presentations_b) = permuted_records(region, 419);

    let first_metadata = write_global_pack(
        &path,
        namespace(),
        [(
            region,
            records_a.clone(),
            local_ids_a.clone(),
            presentations_a.clone(),
        )],
    )
    .unwrap();
    let second_metadata = write_global_pack(
        &second,
        namespace(),
        [(
            region,
            records_a.clone(),
            local_ids_a.clone(),
            presentations_a.clone(),
        )],
    )
    .unwrap();
    let reordered_metadata = write_global_pack(
        &reordered,
        namespace(),
        [(
            region,
            records_b.clone(),
            local_ids_b.clone(),
            presentations_b.clone(),
        )],
    )
    .unwrap();

    assert_eq!(first_metadata.payload_schema, GLOBAL_PAYLOAD_SCHEMA);
    assert_eq!(first_metadata.payload_bytes, u64::from(GLOBAL_REGION_BYTES));
    assert_eq!(fs::read(&path).unwrap(), fs::read(&second).unwrap());
    assert_eq!(
        first_metadata.source_namespace_sha256,
        second_metadata.source_namespace_sha256
    );
    assert_ne!(
        first_metadata.source_namespace_sha256,
        reordered_metadata.source_namespace_sha256
    );
    assert_eq!(
        first_metadata.stable_seed_namespace_sha256,
        reordered_metadata.stable_seed_namespace_sha256
    );

    let mut first = GlobalRegionPack::open(&path).unwrap();
    let first_read = first.read_region(region).unwrap();
    assert_eq!(first_read.payload_bytes, GLOBAL_REGION_BYTES);
    assert_eq!(first_read.payload.len(), GLOBAL_REGION_BYTES as usize);
    assert_eq!(first_read.records, records_a);
    assert_eq!(first_read.local_ids, local_ids_a);
    assert_eq!(first_read.presentations, presentations_a);
    assert_eq!(
        &first_read.payload[REGION_BYTES as usize..(REGION_BYTES + IDENTITY_PLANE_BYTES) as usize],
        &encode_local_ids(&first_read.local_ids)
    );
    assert_eq!(
        first_read.payload.len() - REGION_BYTES as usize - IDENTITY_PLANE_BYTES as usize,
        PRESENTATION_PLANE_BYTES as usize
    );

    let mut reordered_pack = GlobalRegionPack::open(&reordered).unwrap();
    let reordered_read = reordered_pack.read_region(region).unwrap();
    assert_eq!(
        keyed_triples(
            &first_read.records,
            &first_read.local_ids,
            &first_read.presentations
        ),
        keyed_triples(
            &reordered_read.records,
            &reordered_read.local_ids,
            &reordered_read.presentations
        )
    );

    fs::remove_file(path).unwrap();
    fs::remove_file(second).unwrap();
    fs::remove_file(reordered).unwrap();
}

#[test]
fn complete_index_changes_source_without_changing_seed_identity() {
    let first = temp_path("source-a");
    let second = temp_path("source-b");
    let region = GlobalRegion::new(-7, 11);
    let first_metadata =
        write_global_pack(&first, namespace(), [canonical_payload(region, 0)]).unwrap();
    let second_metadata =
        write_global_pack(&second, namespace(), [canonical_payload(region, 1)]).unwrap();
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
        [canonical_payload(first, 0), canonical_payload(second, 0)],
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
    let local_ids = (0..RECORDS_PER_REGION).collect::<Vec<_>>();
    assert!(
        write_global_pack(
            &path,
            namespace(),
            [(region, invalid, local_ids, presentations(0))]
        )
        .is_err()
    );
    assert!(write_global_pack(&path, [0; 32], [canonical_payload(region, 0)]).is_err());
}

#[test]
fn presentation_writer_rejects_invalid_fields_and_count() {
    let path = temp_path("presentation-invalid");
    let region = GlobalRegion::new(31, -47);
    let records = records(region, 0);
    let local_ids = (0..RECORDS_PER_REGION).collect::<Vec<_>>();
    let base = presentations(0);
    for (label, mutate) in [
        ("archetype", (0, 8)),
        ("material", (1, 64)),
        ("yaw", (2, 65_536)),
        ("clip", (3, 8)),
        ("phase", (3, 64 << 8)),
    ] {
        let mut invalid = base.clone();
        match mutate.0 {
            0 => invalid[0].archetype = mutate.1,
            1 => invalid[0].material = mutate.1,
            2 => invalid[0].yaw_q16 = mutate.1,
            3 if label == "clip" => invalid[1].animation = mutate.1,
            3 => invalid[1].animation = mutate.1,
            _ => unreachable!(),
        }
        assert!(
            write_global_pack(
                &path,
                namespace(),
                [(region, records.clone(), local_ids.clone(), invalid)]
            )
            .is_err(),
            "{label}"
        );
    }
    let mut static_record = base.clone();
    static_record[0].animation = STATIC_PRESENTATION_ANIMATION;
    write_global_pack(
        &path,
        namespace(),
        [(region, records.clone(), local_ids.clone(), static_record)],
    )
    .unwrap();
    assert!(
        write_global_pack(
            &path,
            namespace(),
            [(region, records, local_ids, base[..base.len() - 1].to_vec())]
        )
        .is_err()
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn identity_writer_and_reader_reject_invalid_local_ids() {
    let path = temp_path("identity-invalid");
    let region = GlobalRegion::new(-17, 29);
    let (records, local_ids, presentations) = permuted_records(region, 31);

    let mut duplicate = local_ids.clone();
    duplicate[1] = duplicate[0];
    assert!(
        write_global_pack(
            &path,
            namespace(),
            [(region, records.clone(), duplicate, presentations.clone())]
        )
        .is_err()
    );
    let mut out_of_range = local_ids.clone();
    out_of_range[0] = RECORDS_PER_REGION;
    assert!(
        write_global_pack(
            &path,
            namespace(),
            [(region, records.clone(), out_of_range, presentations.clone())]
        )
        .is_err()
    );
    assert!(
        write_global_pack(
            &path,
            namespace(),
            [(
                region,
                records.clone(),
                local_ids[..local_ids.len() - 1].to_vec(),
                presentations.clone()
            )]
        )
        .is_err()
    );

    write_global_pack(
        &path,
        namespace(),
        [(region, records, local_ids.clone(), presentations)],
    )
    .unwrap();
    let original = fs::read(&path).unwrap();
    let metadata = GlobalRegionPack::open(&path).unwrap().metadata().clone();
    let identity_offset = metadata.payload_offset as usize + REGION_BYTES as usize;

    let corrupt = temp_path("identity-checksum");
    let mut bytes = original.clone();
    bytes[identity_offset] ^= 1;
    fs::write(&corrupt, bytes).unwrap();
    assert!(
        GlobalRegionPack::open(&corrupt)
            .unwrap()
            .read_region(region)
            .is_err()
    );
    fs::remove_file(corrupt).unwrap();

    for (label, value) in [
        ("identity-duplicate", local_ids[0]),
        ("identity-range", RECORDS_PER_REGION),
    ] {
        let malformed = temp_path(label);
        let mut bytes = original.clone();
        bytes[identity_offset + 4..identity_offset + 8].copy_from_slice(&value.to_le_bytes());
        rewrite_first_payload_checksum(&mut bytes, metadata.payload_offset as usize);
        fs::write(&malformed, bytes).unwrap();
        assert!(
            GlobalRegionPack::open(&malformed)
                .unwrap()
                .read_region(region)
                .is_err(),
            "{label}"
        );
        fs::remove_file(malformed).unwrap();
    }

    let wrong_size = temp_path("identity-size");
    let mut bytes = original;
    bytes[GLOBAL_HEADER_BYTES as usize + 24..GLOBAL_HEADER_BYTES as usize + 28]
        .copy_from_slice(&REGION_BYTES.to_le_bytes());
    fs::write(&wrong_size, bytes).unwrap();
    assert!(GlobalRegionPack::open(&wrong_size).is_err());
    fs::remove_file(wrong_size).unwrap();
    fs::remove_file(path).unwrap();
}

fn permuted_records(
    region: GlobalRegion,
    offset: u32,
) -> (Vec<InstanceRecord>, Vec<u32>, Vec<PresentationRecord>) {
    let source = records(region, 0);
    let presentations = presentations(0);
    let mut output = Vec::with_capacity(RECORDS_PER_REGION as usize);
    let mut local_ids = Vec::with_capacity(RECORDS_PER_REGION as usize);
    let mut presentation_output = Vec::with_capacity(RECORDS_PER_REGION as usize);
    for index in 0..RECORDS_PER_REGION {
        let local_id = (index.wrapping_mul(769).wrapping_add(offset)) % RECORDS_PER_REGION;
        output.push(source[local_id as usize]);
        local_ids.push(local_id);
        presentation_output.push(presentations[local_id as usize]);
    }
    (output, local_ids, presentation_output)
}

fn keyed_triples(
    records: &[InstanceRecord],
    local_ids: &[u32],
    presentations: &[PresentationRecord],
) -> Vec<(u32, InstanceRecord, PresentationRecord)> {
    let mut keyed = records
        .iter()
        .copied()
        .zip(local_ids.iter().copied())
        .zip(presentations.iter().copied())
        .map(|((record, local_id), presentation)| (local_id, record, presentation))
        .collect::<Vec<_>>();
    keyed.sort_by_key(|(local_id, _, _)| *local_id);
    keyed
}

fn encode_local_ids(local_ids: &[u32]) -> Vec<u8> {
    local_ids
        .iter()
        .flat_map(|local_id| local_id.to_le_bytes())
        .collect()
}

fn rewrite_first_payload_checksum(bytes: &mut [u8], payload_offset: usize) {
    let digest =
        Sha256::digest(&bytes[payload_offset..payload_offset + GLOBAL_REGION_BYTES as usize]);
    let checksum_offset = GLOBAL_HEADER_BYTES as usize + 32;
    bytes[checksum_offset..checksum_offset + 32].copy_from_slice(&digest);
}
