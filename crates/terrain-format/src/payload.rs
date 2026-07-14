use anyhow::{Result, ensure};

use super::{
    CELL_SIDE, HEIGHT_OFFSET, HEIGHT_UNIT_DENOMINATOR, MATERIAL_COUNT, MATERIAL_OFFSET,
    MATERIAL_PALETTE_SIZE, PAYLOAD_BYTES, SAMPLE_SIDE, TerrainTile, WORLD_REGION_SIDE,
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
