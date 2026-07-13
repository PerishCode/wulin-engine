use std::collections::BTreeMap;
use std::fs;

use terrain_format::{
    CELL_SIDE, HEIGHT_COUNT, MATERIAL_COUNT, PAYLOAD_BYTES, SAMPLE_SIDE, TerrainPack, TerrainTile,
    WORLD_REGION_SIDE, validate_neighbor_edges, write_pack,
};

fn tile(region_id: u32) -> TerrainTile {
    let region_x = region_id % WORLD_REGION_SIDE;
    let region_z = region_id / WORLD_REGION_SIDE;
    let mut heights = [0i16; HEIGHT_COUNT];
    for z in 0..SAMPLE_SIDE {
        for x in 0..SAMPLE_SIDE {
            heights[z * SAMPLE_SIDE + x] = (region_x as i32 * CELL_SIDE as i32
                + x as i32
                + (region_z as i32 * CELL_SIDE as i32 + z as i32) * 2)
                as i16;
        }
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    for (index, material) in materials.iter_mut().enumerate() {
        *material = (index % 8) as u8;
    }
    TerrainTile {
        region_id,
        heights,
        materials,
    }
}

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("wulin-terrain-{name}-{}.wlt", std::process::id()))
}

#[test]
fn canonical_pack_round_trips() {
    let path = temp_path("round-trip");
    let ids = [64 * WORLD_REGION_SIDE + 64, 64 * WORLD_REGION_SIDE + 65];
    let expected = ids
        .into_iter()
        .map(|id| (id, tile(id)))
        .collect::<BTreeMap<_, _>>();
    write_pack(&path, expected.values().cloned()).unwrap();
    let mut pack = TerrainPack::open(&path).unwrap();
    assert_eq!(pack.metadata().region_count, 2);
    for (region_id, tile) in expected {
        let read = pack.read_region(region_id).unwrap();
        assert_eq!(read.payload_bytes, PAYLOAD_BYTES);
        assert_eq!(read.tile, tile);
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn mismatched_shared_edge_is_rejected() {
    let left_id = 64 * WORLD_REGION_SIDE + 64;
    let mut right = tile(left_id + 1);
    right.heights[0] += 1;
    let left = tile(left_id);
    let validation = validate_neighbor_edges([&left, &right]);
    assert_eq!(validation.neighbor_edges, 1);
    assert_eq!(validation.sample_comparisons, SAMPLE_SIDE as u32);
    assert_eq!(validation.mismatch_count, 1);
    assert_eq!(validation.first_mismatch.unwrap().axis, "x");
    assert!(write_pack(temp_path("bad-edge"), [left, right]).is_err());
}

#[test]
fn malformed_header_checksum_and_padding_are_rejected() {
    let source = temp_path("source");
    let region_id = 64 * WORLD_REGION_SIDE + 64;
    write_pack(&source, [tile(region_id)]).unwrap();
    let bytes = fs::read(&source).unwrap();
    let cases = [
        ("header", 0usize),
        ("checksum", 64 + 24),
        ("padding", bytes.len() - 1),
    ];
    for (name, offset) in cases {
        let path = temp_path(name);
        let mut malformed = bytes.clone();
        malformed[offset] ^= 1;
        fs::write(&path, malformed).unwrap();
        if name == "header" {
            assert!(TerrainPack::open(&path).is_err());
        } else {
            let mut pack = TerrainPack::open(&path).unwrap();
            assert!(pack.read_region(region_id).is_err());
        }
        fs::remove_file(path).unwrap();
    }
    fs::remove_file(source).unwrap();
}
