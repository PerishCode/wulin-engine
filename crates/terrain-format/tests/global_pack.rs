use std::collections::BTreeMap;
use std::fs;

use terrain_format::{
    CELL_SIDE, GLOBAL_INDEX_ENTRY_BYTES, GlobalRegion, GlobalTerrainPack, GlobalTerrainTile,
    HEIGHT_COUNT, MATERIAL_COUNT, PAYLOAD_BYTES, SAMPLE_SIDE, validate_global_neighbor_edges,
    write_global_pack,
};

fn tile(region: GlobalRegion) -> GlobalTerrainTile {
    let mut heights = [0i16; HEIGHT_COUNT];
    for z in 0..SAMPLE_SIDE {
        for x in 0..SAMPLE_SIDE {
            let global_x = region.x * CELL_SIDE as i64 + x as i64;
            let global_z = region.z * CELL_SIDE as i64 + z as i64;
            heights[z * SAMPLE_SIDE + x] = (global_x + global_z * 2) as i16;
        }
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    for (index, material) in materials.iter_mut().enumerate() {
        *material = (index % 8) as u8;
    }
    GlobalTerrainTile {
        region,
        heights,
        materials,
    }
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "wulin-global-terrain-{name}-{}.wlt",
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
    let expected = regions
        .into_iter()
        .map(|region| (region, tile(region)))
        .collect::<BTreeMap<_, _>>();
    let first_metadata = write_global_pack(&path, expected.values().cloned()).unwrap();
    let second_metadata = write_global_pack(&second, expected.values().cloned()).unwrap();
    assert_eq!(
        first_metadata.source_namespace_sha256,
        second_metadata.source_namespace_sha256
    );
    assert_eq!(fs::read(&path).unwrap(), fs::read(&second).unwrap());

    let mut pack = GlobalTerrainPack::open(&path).unwrap();
    assert_eq!(pack.metadata().region_count, 3);
    for (region, tile) in expected {
        let read = pack.read_region(region).unwrap();
        assert_eq!(read.payload_bytes, PAYLOAD_BYTES);
        assert_eq!(read.tile, tile);
    }
    fs::remove_file(path).unwrap();
    fs::remove_file(second).unwrap();
}

#[test]
fn signed_edges_cross_zero_exactly() {
    let mut right = tile(GlobalRegion::new(0, -1));
    right.heights[0] += 1;
    let left = tile(GlobalRegion::new(-1, -1));
    let validation = validate_global_neighbor_edges([&left, &right]);
    assert_eq!(validation.neighbor_edges, 1);
    assert_eq!(validation.sample_comparisons, SAMPLE_SIDE as u32);
    assert_eq!(validation.mismatch_count, 1);
    assert_eq!(validation.first_mismatch.unwrap().axis, "x");
    assert!(write_global_pack(temp_path("bad-edge"), [left, right]).is_err());
}

#[test]
fn signed_metadata_checksum_and_payload_are_rejected() {
    let source = temp_path("source");
    let region = GlobalRegion::new(-7, 11);
    write_global_pack(&source, [tile(region)]).unwrap();
    let bytes = fs::read(&source).unwrap();
    let cases = [
        ("header", 0usize),
        ("key", 64),
        ("checksum", 64 + 32),
        ("payload-key", bytes.len() - PAYLOAD_BYTES as usize),
        ("padding", bytes.len() - 1),
    ];
    for (name, offset) in cases {
        let path = temp_path(name);
        let mut malformed = bytes.clone();
        malformed[offset] ^= 1;
        fs::write(&path, malformed).unwrap();
        if name == "header" {
            assert!(GlobalTerrainPack::open(&path).is_err());
        } else if name == "key" {
            let mut pack = GlobalTerrainPack::open(&path).unwrap();
            assert!(pack.read_region(region).is_err());
            assert!(
                pack.read_region(GlobalRegion::new(region.x ^ 1, region.z))
                    .is_err()
            );
        } else {
            let mut pack = GlobalTerrainPack::open(&path).unwrap();
            assert!(pack.read_region(region).is_err());
        }
        fs::remove_file(path).unwrap();
    }

    let mut bad_offset = bytes.clone();
    let offset = 64 + 16;
    bad_offset[offset..offset + 8].copy_from_slice(&0u64.to_le_bytes());
    let path = temp_path("offset");
    fs::write(&path, bad_offset).unwrap();
    assert!(GlobalTerrainPack::open(&path).is_err());
    fs::remove_file(path).unwrap();
    fs::remove_file(source).unwrap();

    assert_eq!(GLOBAL_INDEX_ENTRY_BYTES, 64);
}
