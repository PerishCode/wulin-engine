use anyhow::{Result, ensure};

use super::{
    CELL_SIDE, HEIGHT_COUNT, HEIGHT_OFFSET, HEIGHT_UNIT_DENOMINATOR, MATERIAL_COUNT,
    MATERIAL_OFFSET, MATERIAL_PALETTE_SIZE, PAYLOAD_BYTES, SAMPLE_SIDE, TerrainTile,
    WORLD_REGION_SIDE, u32_at,
};

pub fn validate_tile(tile: &TerrainTile) -> Result<()> {
    ensure!(
        tile.region_id < WORLD_REGION_SIDE * WORLD_REGION_SIDE,
        "terrain region {} is outside the world",
        tile.region_id
    );
    ensure!(
        tile.materials
            .iter()
            .all(|value| *value < MATERIAL_PALETTE_SIZE),
        "terrain region {} has a material outside the fixed palette",
        tile.region_id
    );
    Ok(())
}

pub fn encode_tile(tile: &TerrainTile) -> Result<[u8; PAYLOAD_BYTES as usize]> {
    validate_tile(tile)?;
    Ok(encode_payload(tile))
}

pub(super) fn encode_payload(tile: &TerrainTile) -> [u8; PAYLOAD_BYTES as usize] {
    let mut bytes = [0u8; PAYLOAD_BYTES as usize];
    bytes[0..4].copy_from_slice(&tile.region_id.to_le_bytes());
    bytes[4..8].copy_from_slice(&(SAMPLE_SIDE as u32).to_le_bytes());
    bytes[8..12].copy_from_slice(&(CELL_SIDE as u32).to_le_bytes());
    bytes[12..16].copy_from_slice(&HEIGHT_UNIT_DENOMINATOR.to_le_bytes());
    for (index, height) in tile.heights.iter().enumerate() {
        let offset = HEIGHT_OFFSET + index * 2;
        bytes[offset..offset + 2].copy_from_slice(&height.to_le_bytes());
    }
    bytes[MATERIAL_OFFSET..MATERIAL_OFFSET + MATERIAL_COUNT].copy_from_slice(&tile.materials);
    bytes
}

pub(super) fn decode_payload(bytes: &[u8; PAYLOAD_BYTES as usize]) -> Result<TerrainTile> {
    ensure!(
        u32_at(bytes, 4) == SAMPLE_SIDE as u32,
        "terrain payload sample side is invalid"
    );
    ensure!(
        u32_at(bytes, 8) == CELL_SIDE as u32,
        "terrain payload cell side is invalid"
    );
    ensure!(
        u32_at(bytes, 12) == HEIGHT_UNIT_DENOMINATOR,
        "terrain payload height unit is invalid"
    );
    ensure!(
        bytes[HEIGHT_OFFSET + HEIGHT_COUNT * 2..MATERIAL_OFFSET]
            .iter()
            .all(|byte| *byte == 0),
        "terrain payload alignment padding is nonzero"
    );
    ensure!(
        bytes[MATERIAL_OFFSET + MATERIAL_COUNT..]
            .iter()
            .all(|byte| *byte == 0),
        "terrain payload trailing padding is nonzero"
    );
    let mut heights = [0i16; HEIGHT_COUNT];
    for (index, height) in heights.iter_mut().enumerate() {
        let offset = HEIGHT_OFFSET + index * 2;
        *height = i16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("i16 slice"));
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    materials.copy_from_slice(&bytes[MATERIAL_OFFSET..MATERIAL_OFFSET + MATERIAL_COUNT]);
    let tile = TerrainTile {
        region_id: u32_at(bytes, 0),
        heights,
        materials,
    };
    validate_tile(&tile)?;
    Ok(tile)
}
