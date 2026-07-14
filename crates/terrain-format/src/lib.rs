use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};

mod global;
mod payload;

pub use global::{
    GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_MAGIC, GLOBAL_VERSION, GlobalEdgeMismatch,
    GlobalEdgeValidation, GlobalPackMetadata, GlobalRegion, GlobalTerrainPack, GlobalTerrainRead,
    GlobalTerrainTile, encode_global_tile, validate_global_neighbor_edges, write_global_pack,
};
pub use payload::{encode_tile, validate_tile};

pub(crate) const HEADER_BYTES: u32 = 64;
pub const PAYLOAD_BYTES: u32 = 4_096;
pub const PAYLOAD_ALIGNMENT: u64 = 4_096;
pub const WORLD_REGION_SIDE: u32 = 128;
pub const REGION_SIDE_METERS: f32 = 16.0;
pub const CELL_SIDE: usize = 32;
pub const SAMPLE_SIDE: usize = CELL_SIDE + 1;
pub const HEIGHT_COUNT: usize = SAMPLE_SIDE * SAMPLE_SIDE;
pub const MATERIAL_COUNT: usize = CELL_SIDE * CELL_SIDE;
pub const HEIGHT_UNIT_DENOMINATOR: u32 = 256;
pub const HEIGHT_OFFSET: usize = 16;
pub const MATERIAL_OFFSET: usize = 2_196;
pub const MATERIAL_PALETTE_SIZE: u8 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainTile {
    pub region_id: u32,
    pub heights: [i16; HEIGHT_COUNT],
    pub materials: [u8; MATERIAL_COUNT],
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EdgeValidation {
    pub neighbor_edges: u32,
    pub sample_comparisons: u32,
    pub mismatch_count: u32,
    pub first_mismatch: Option<EdgeMismatch>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EdgeMismatch {
    pub region_id: u32,
    pub neighbor_region_id: u32,
    pub axis: &'static str,
    pub sample_index: u32,
    pub value: i16,
    pub neighbor_value: i16,
}

pub fn validate_neighbor_edges<'a>(
    tiles: impl IntoIterator<Item = &'a TerrainTile>,
) -> EdgeValidation {
    let tiles = tiles
        .into_iter()
        .map(|tile| (tile.region_id, tile))
        .collect::<BTreeMap<_, _>>();
    let mut result = EdgeValidation {
        neighbor_edges: 0,
        sample_comparisons: 0,
        mismatch_count: 0,
        first_mismatch: None,
    };
    for (&region_id, tile) in &tiles {
        let x = region_id % WORLD_REGION_SIDE;
        let z = region_id / WORLD_REGION_SIDE;
        if x + 1 < WORLD_REGION_SIDE
            && let Some(neighbor) = tiles.get(&(region_id + 1))
        {
            compare_edge(&mut result, tile, neighbor, "x", |sample| {
                (sample * SAMPLE_SIDE + CELL_SIDE, sample * SAMPLE_SIDE)
            });
        }
        if z + 1 < WORLD_REGION_SIDE
            && let Some(neighbor) = tiles.get(&(region_id + WORLD_REGION_SIDE))
        {
            compare_edge(&mut result, tile, neighbor, "z", |sample| {
                (CELL_SIDE * SAMPLE_SIDE + sample, sample)
            });
        }
    }
    result
}

fn compare_edge(
    result: &mut EdgeValidation,
    tile: &TerrainTile,
    neighbor: &TerrainTile,
    axis: &'static str,
    indices: impl Fn(usize) -> (usize, usize),
) {
    result.neighbor_edges += 1;
    for sample in 0..SAMPLE_SIDE {
        let (left, right) = indices(sample);
        let value = tile.heights[left];
        let neighbor_value = neighbor.heights[right];
        result.sample_comparisons += 1;
        if value != neighbor_value {
            result.mismatch_count += 1;
            result.first_mismatch.get_or_insert(EdgeMismatch {
                region_id: tile.region_id,
                neighbor_region_id: neighbor.region_id,
                axis,
                sample_index: sample as u32,
                value,
                neighbor_value,
            });
        }
    }
}

pub fn file_sha256(path: impl AsRef<Path>) -> Result<String> {
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
