use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

mod global;
mod payload;

pub use global::{
    GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_MAGIC, GLOBAL_VERSION, GlobalEdgeMismatch,
    GlobalEdgeValidation, GlobalPackMetadata, GlobalRegion, GlobalTerrainPack, GlobalTerrainRead,
    GlobalTerrainTile, encode_global_tile, validate_global_neighbor_edges, write_global_pack,
};
use payload::{decode_payload, encode_payload};
pub use payload::{encode_tile, validate_tile};

pub const MAGIC: [u8; 8] = *b"WLTRN001";
pub const VERSION: u32 = 1;
pub const HEADER_BYTES: u32 = 64;
pub const INDEX_ENTRY_BYTES: u32 = 56;
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

#[derive(Clone, Debug)]
struct IndexEntry {
    payload_offset: u64,
    sha256: [u8; 32],
}

pub struct TerrainPack {
    file: File,
    metadata: PackMetadata,
    entries: BTreeMap<u32, IndexEntry>,
}

pub struct TerrainRead {
    pub tile: TerrainTile,
    pub payload: [u8; PAYLOAD_BYTES as usize],
    pub payload_bytes: u32,
    pub sha256: String,
    pub read_ms: f64,
    pub verify_ms: f64,
}

impl TerrainPack {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("failed to open terrain pack {}", path.display()))?;
        let actual_file_bytes = file
            .metadata()
            .context("failed to inspect terrain pack")?
            .len();
        let mut header = [0u8; HEADER_BYTES as usize];
        file.read_exact(&mut header)
            .context("terrain pack header is truncated")?;
        let mut index_hash = Sha256::new();
        index_hash.update(header);

        ensure!(header[0..8] == MAGIC, "terrain pack magic is invalid");
        let version = u32_at(&header, 8);
        ensure!(
            version == VERSION,
            "unsupported terrain pack version {version}"
        );
        ensure!(
            u32_at(&header, 12) == HEADER_BYTES,
            "terrain pack header size is invalid"
        );
        let region_count = u32_at(&header, 16);
        ensure!(region_count > 0, "terrain pack contains no regions");
        ensure!(
            u32_at(&header, 20) == INDEX_ENTRY_BYTES,
            "terrain pack index entry size is invalid"
        );
        ensure!(
            u32_at(&header, 24) == PAYLOAD_BYTES,
            "terrain pack payload size is invalid"
        );
        ensure!(
            u32_at(&header, 28) == WORLD_REGION_SIDE,
            "terrain pack world side is invalid"
        );
        ensure!(
            u64_at(&header, 32) == u64::from(HEADER_BYTES),
            "terrain pack index offset is invalid"
        );
        ensure!(
            u64_at(&header, 56) == 0,
            "terrain pack reserved header is nonzero"
        );

        let index_bytes = u64::from(region_count) * u64::from(INDEX_ENTRY_BYTES);
        let expected_payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
        let payload_offset = u64_at(&header, 40);
        ensure!(
            payload_offset == expected_payload_offset,
            "terrain pack payload offset is invalid"
        );
        let payload_bytes = u64::from(region_count) * u64::from(PAYLOAD_BYTES);
        let file_bytes = u64_at(&header, 48);
        ensure!(
            file_bytes == payload_offset + payload_bytes,
            "terrain pack file size declaration is invalid"
        );
        ensure!(
            actual_file_bytes == file_bytes,
            "terrain pack file size does not match its header"
        );

        let mut entries = BTreeMap::new();
        let mut previous = None;
        for index in 0..region_count {
            let mut bytes = [0u8; INDEX_ENTRY_BYTES as usize];
            file.read_exact(&mut bytes)
                .context("terrain pack index is truncated")?;
            index_hash.update(bytes);
            let region_id = u32_at(&bytes, 0);
            ensure!(
                region_id < WORLD_REGION_SIDE * WORLD_REGION_SIDE,
                "terrain region {region_id} is outside the world"
            );
            if let Some(previous) = previous {
                ensure!(
                    region_id > previous,
                    "terrain pack IDs are not sorted and unique"
                );
            }
            previous = Some(region_id);
            ensure!(
                u32_at(&bytes, 4) == 0,
                "terrain region {region_id} has unknown flags"
            );
            let offset = u64_at(&bytes, 8);
            let expected_offset = payload_offset + u64::from(index) * u64::from(PAYLOAD_BYTES);
            ensure!(
                offset.is_multiple_of(PAYLOAD_ALIGNMENT),
                "terrain region {region_id} payload is not aligned"
            );
            ensure!(
                offset == expected_offset,
                "terrain region {region_id} payload range is noncanonical"
            );
            ensure!(
                u32_at(&bytes, 16) == PAYLOAD_BYTES,
                "terrain region {region_id} payload size is invalid"
            );
            ensure!(
                u32_at(&bytes, 20) == 0,
                "terrain region {region_id} index reserved field is nonzero"
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

    pub fn read_region(&mut self, region_id: u32) -> Result<TerrainRead> {
        let entry = self
            .entries
            .get(&region_id)
            .with_context(|| format!("terrain region {region_id} is absent from the pack"))?
            .clone();
        let read_start = std::time::Instant::now();
        self.file
            .seek(SeekFrom::Start(entry.payload_offset))
            .with_context(|| format!("failed to seek terrain region {region_id}"))?;
        let mut bytes = [0u8; PAYLOAD_BYTES as usize];
        self.file
            .read_exact(&mut bytes)
            .with_context(|| format!("terrain region {region_id} payload is truncated"))?;
        let read_ms = read_start.elapsed().as_secs_f64() * 1_000.0;
        let verify_start = std::time::Instant::now();
        let actual_sha256 = Sha256::digest(bytes);
        ensure!(
            actual_sha256.as_slice() == entry.sha256,
            "terrain region {region_id} payload checksum mismatch"
        );
        let tile = decode_payload(&bytes)?;
        ensure!(
            tile.region_id == region_id,
            "terrain region {region_id} payload declares region {}",
            tile.region_id
        );
        Ok(TerrainRead {
            tile,
            payload: bytes,
            payload_bytes: PAYLOAD_BYTES,
            sha256: hex(&entry.sha256),
            read_ms,
            verify_ms: verify_start.elapsed().as_secs_f64() * 1_000.0,
        })
    }
}

pub fn write_pack(
    path: impl AsRef<Path>,
    tiles: impl IntoIterator<Item = TerrainTile>,
) -> Result<PackMetadata> {
    let path = path.as_ref();
    let mut tiles = tiles.into_iter().collect::<Vec<_>>();
    tiles.sort_by_key(|tile| tile.region_id);
    ensure!(!tiles.is_empty(), "cannot write an empty terrain pack");
    for pair in tiles.windows(2) {
        ensure!(
            pair[0].region_id != pair[1].region_id,
            "duplicate terrain region ID {}",
            pair[0].region_id
        );
    }
    for tile in &tiles {
        validate_tile(tile)?;
    }
    let edge_validation = validate_neighbor_edges(tiles.iter());
    ensure!(
        edge_validation.mismatch_count == 0,
        "terrain pack contains mismatched neighboring edges"
    );

    let region_count = u32::try_from(tiles.len()).context("too many terrain regions")?;
    let index_bytes = u64::from(region_count) * u64::from(INDEX_ENTRY_BYTES);
    let payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
    let payload_bytes = u64::from(region_count) * u64::from(PAYLOAD_BYTES);
    let file_bytes = payload_offset + payload_bytes;
    let encoded = tiles
        .iter()
        .map(encode_payload)
        .map(|bytes| {
            let sha256 = Sha256::digest(bytes).into();
            (bytes, sha256)
        })
        .collect::<Vec<([u8; PAYLOAD_BYTES as usize], [u8; 32])>>();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = File::create(path)
        .with_context(|| format!("failed to create terrain pack {}", path.display()))?;
    let mut header = Vec::with_capacity(HEADER_BYTES as usize);
    header.extend_from_slice(&MAGIC);
    push_u32(&mut header, VERSION);
    push_u32(&mut header, HEADER_BYTES);
    push_u32(&mut header, region_count);
    push_u32(&mut header, INDEX_ENTRY_BYTES);
    push_u32(&mut header, PAYLOAD_BYTES);
    push_u32(&mut header, WORLD_REGION_SIDE);
    push_u64(&mut header, u64::from(HEADER_BYTES));
    push_u64(&mut header, payload_offset);
    push_u64(&mut header, file_bytes);
    push_u64(&mut header, 0);
    file.write_all(&header)
        .context("failed to write terrain pack header")?;

    for (index, (tile, (_, sha256))) in tiles.iter().zip(&encoded).enumerate() {
        push_u32_to(&mut file, tile.region_id)?;
        push_u32_to(&mut file, 0)?;
        push_u64_to(
            &mut file,
            payload_offset + index as u64 * u64::from(PAYLOAD_BYTES),
        )?;
        push_u32_to(&mut file, PAYLOAD_BYTES)?;
        push_u32_to(&mut file, 0)?;
        file.write_all(sha256)?;
    }
    let position = file.stream_position()?;
    ensure!(
        position <= payload_offset,
        "terrain pack index exceeded payload offset"
    );
    file.write_all(&vec![0u8; (payload_offset - position) as usize])?;
    for (bytes, _) in encoded {
        file.write_all(&bytes)?;
    }
    file.flush().context("failed to flush terrain pack")?;
    drop(file);

    let pack = TerrainPack::open(path)?;
    Ok(pack.metadata().clone())
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
