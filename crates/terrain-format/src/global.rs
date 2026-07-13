use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    CELL_SIDE, HEADER_BYTES, HEIGHT_COUNT, HEIGHT_OFFSET, MATERIAL_COUNT, MATERIAL_OFFSET,
    MATERIAL_PALETTE_SIZE, PAYLOAD_ALIGNMENT, PAYLOAD_BYTES, SAMPLE_SIDE, align_up, hex, push_u32,
    push_u32_to, push_u64, push_u64_to, u32_at, u64_at,
};

pub const GLOBAL_MAGIC: [u8; 8] = *b"WLTRN002";
pub const GLOBAL_VERSION: u32 = 2;
pub const GLOBAL_INDEX_ENTRY_BYTES: u32 = 64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalRegion {
    pub x: i64,
    pub z: i64,
}

impl GlobalRegion {
    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalTerrainTile {
    pub region: GlobalRegion,
    pub heights: [i16; HEIGHT_COUNT],
    pub materials: [u8; MATERIAL_COUNT],
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPackMetadata {
    pub version: u32,
    pub addressing: &'static str,
    pub region_count: u32,
    pub index_bytes: u64,
    pub payload_offset: u64,
    pub payload_bytes: u64,
    pub file_bytes: u64,
    pub payload_alignment: u64,
    pub source_namespace_sha256: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalEdgeValidation {
    pub neighbor_edges: u32,
    pub sample_comparisons: u32,
    pub mismatch_count: u32,
    pub first_mismatch: Option<GlobalEdgeMismatch>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalEdgeMismatch {
    pub region: GlobalRegion,
    pub neighbor: GlobalRegion,
    pub axis: &'static str,
    pub sample_index: u32,
    pub value: i16,
    pub neighbor_value: i16,
}

#[derive(Clone, Debug)]
struct GlobalIndexEntry {
    payload_offset: u64,
    sha256: [u8; 32],
}

pub struct GlobalTerrainPack {
    file: File,
    metadata: GlobalPackMetadata,
    source_namespace: [u8; 32],
    entries: BTreeMap<GlobalRegion, GlobalIndexEntry>,
}

pub struct GlobalTerrainRead {
    pub tile: GlobalTerrainTile,
    pub payload: [u8; PAYLOAD_BYTES as usize],
    pub payload_bytes: u32,
    pub sha256: String,
    pub read_ms: f64,
    pub verify_ms: f64,
}

impl GlobalTerrainPack {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("failed to open signed terrain pack {}", path.display()))?;
        let actual_file_bytes = file
            .metadata()
            .context("failed to inspect signed terrain pack")?
            .len();
        let mut header = [0u8; HEADER_BYTES as usize];
        file.read_exact(&mut header)
            .context("signed terrain pack header is truncated")?;
        let mut namespace = Sha256::new();
        namespace.update(header);

        ensure!(
            header[0..8] == GLOBAL_MAGIC,
            "signed terrain pack magic is invalid"
        );
        let version = u32_at(&header, 8);
        ensure!(
            version == GLOBAL_VERSION,
            "unsupported signed terrain pack version {version}"
        );
        ensure!(
            u32_at(&header, 12) == HEADER_BYTES,
            "signed terrain pack header size is invalid"
        );
        let region_count = u32_at(&header, 16);
        ensure!(region_count > 0, "signed terrain pack contains no regions");
        ensure!(
            u32_at(&header, 20) == GLOBAL_INDEX_ENTRY_BYTES,
            "signed terrain pack index entry size is invalid"
        );
        ensure!(
            u32_at(&header, 24) == PAYLOAD_BYTES,
            "signed terrain pack payload size is invalid"
        );
        ensure!(
            u32_at(&header, 28) == 0,
            "signed terrain pack has unknown flags"
        );
        ensure!(
            u64_at(&header, 32) == u64::from(HEADER_BYTES),
            "signed terrain pack index offset is invalid"
        );
        ensure!(
            u64_at(&header, 56) == 0,
            "signed terrain pack reserved header is nonzero"
        );

        let index_bytes = u64::from(region_count) * u64::from(GLOBAL_INDEX_ENTRY_BYTES);
        let expected_payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
        let payload_offset = u64_at(&header, 40);
        ensure!(
            payload_offset == expected_payload_offset,
            "signed terrain pack payload offset is invalid"
        );
        let payload_bytes = u64::from(region_count) * u64::from(PAYLOAD_BYTES);
        let file_bytes = u64_at(&header, 48);
        ensure!(
            file_bytes == payload_offset + payload_bytes,
            "signed terrain pack file size declaration is invalid"
        );
        ensure!(
            actual_file_bytes == file_bytes,
            "signed terrain pack file size does not match its header"
        );

        let mut entries = BTreeMap::new();
        let mut previous = None;
        for index in 0..region_count {
            let mut bytes = [0u8; GLOBAL_INDEX_ENTRY_BYTES as usize];
            file.read_exact(&mut bytes)
                .context("signed terrain pack index is truncated")?;
            namespace.update(bytes);
            let region = GlobalRegion::new(i64_at(&bytes, 0), i64_at(&bytes, 8));
            let order = (region.z, region.x);
            if let Some(previous) = previous {
                ensure!(
                    order > previous,
                    "signed terrain pack keys are not sorted and unique"
                );
            }
            previous = Some(order);
            let offset = u64_at(&bytes, 16);
            let expected_offset = payload_offset + u64::from(index) * u64::from(PAYLOAD_BYTES);
            ensure!(
                offset.is_multiple_of(PAYLOAD_ALIGNMENT),
                "signed terrain region ({},{}) payload is not aligned",
                region.x,
                region.z
            );
            ensure!(
                offset == expected_offset,
                "signed terrain region ({},{}) payload range is noncanonical",
                region.x,
                region.z
            );
            ensure!(
                u32_at(&bytes, 24) == PAYLOAD_BYTES,
                "signed terrain region ({},{}) payload size is invalid",
                region.x,
                region.z
            );
            ensure!(
                u32_at(&bytes, 28) == 0,
                "signed terrain region ({},{}) has unknown flags",
                region.x,
                region.z
            );
            let mut sha256 = [0u8; 32];
            sha256.copy_from_slice(&bytes[32..64]);
            entries.insert(
                region,
                GlobalIndexEntry {
                    payload_offset: offset,
                    sha256,
                },
            );
        }

        let source_namespace: [u8; 32] = namespace.finalize().into();
        Ok(Self {
            file,
            metadata: GlobalPackMetadata {
                version,
                addressing: "signed-region-v1",
                region_count,
                index_bytes,
                payload_offset,
                payload_bytes,
                file_bytes,
                payload_alignment: PAYLOAD_ALIGNMENT,
                source_namespace_sha256: hex(&source_namespace),
            },
            source_namespace,
            entries,
        })
    }

    pub fn metadata(&self) -> &GlobalPackMetadata {
        &self.metadata
    }

    pub fn contains(&self, region: GlobalRegion) -> bool {
        self.entries.contains_key(&region)
    }

    pub fn source_namespace(&self) -> [u8; 32] {
        self.source_namespace
    }

    pub fn regions(&self) -> impl Iterator<Item = GlobalRegion> + '_ {
        self.entries.keys().copied()
    }

    pub fn read_region(&mut self, region: GlobalRegion) -> Result<GlobalTerrainRead> {
        let entry = self.entries.get(&region).with_context(|| {
            format!(
                "signed terrain region ({},{}) is absent from the pack",
                region.x, region.z
            )
        })?;
        let read_start = std::time::Instant::now();
        self.file
            .seek(SeekFrom::Start(entry.payload_offset))
            .with_context(|| {
                format!(
                    "failed to seek signed terrain region ({},{})",
                    region.x, region.z
                )
            })?;
        let mut bytes = [0u8; PAYLOAD_BYTES as usize];
        self.file.read_exact(&mut bytes).with_context(|| {
            format!(
                "signed terrain region ({},{}) payload is truncated",
                region.x, region.z
            )
        })?;
        let read_ms = read_start.elapsed().as_secs_f64() * 1_000.0;
        let verify_start = std::time::Instant::now();
        let actual_sha256 = Sha256::digest(bytes);
        ensure!(
            actual_sha256.as_slice() == entry.sha256,
            "signed terrain region ({},{}) payload checksum mismatch",
            region.x,
            region.z
        );
        let tile = decode_global_payload(&bytes)?;
        ensure!(
            tile.region == region,
            "signed terrain payload declares ({},{}) instead of ({},{})",
            tile.region.x,
            tile.region.z,
            region.x,
            region.z
        );
        Ok(GlobalTerrainRead {
            tile,
            payload: bytes,
            payload_bytes: PAYLOAD_BYTES,
            sha256: hex(&entry.sha256),
            read_ms,
            verify_ms: verify_start.elapsed().as_secs_f64() * 1_000.0,
        })
    }
}

pub fn write_global_pack(
    path: impl AsRef<Path>,
    tiles: impl IntoIterator<Item = GlobalTerrainTile>,
) -> Result<GlobalPackMetadata> {
    let path = path.as_ref();
    let mut tiles = tiles.into_iter().collect::<Vec<_>>();
    tiles.sort_by_key(|tile| (tile.region.z, tile.region.x));
    ensure!(
        !tiles.is_empty(),
        "cannot write an empty signed terrain pack"
    );
    for pair in tiles.windows(2) {
        ensure!(
            pair[0].region != pair[1].region,
            "duplicate signed terrain region ({},{})",
            pair[0].region.x,
            pair[0].region.z
        );
    }
    for tile in &tiles {
        validate_global_tile(tile)?;
    }
    let edge_validation = validate_global_neighbor_edges(tiles.iter());
    ensure!(
        edge_validation.mismatch_count == 0,
        "signed terrain pack contains mismatched neighboring edges"
    );

    let region_count = u32::try_from(tiles.len()).context("too many signed terrain regions")?;
    let index_bytes = u64::from(region_count) * u64::from(GLOBAL_INDEX_ENTRY_BYTES);
    let payload_offset = align_up(u64::from(HEADER_BYTES) + index_bytes);
    let payload_bytes = u64::from(region_count) * u64::from(PAYLOAD_BYTES);
    let file_bytes = payload_offset + payload_bytes;
    let encoded = tiles
        .iter()
        .map(encode_global_tile)
        .map(|result| {
            result.map(|bytes| {
                let sha256 = Sha256::digest(bytes).into();
                (bytes, sha256)
            })
        })
        .collect::<Result<Vec<([u8; PAYLOAD_BYTES as usize], [u8; 32])>>>()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = File::create(path)
        .with_context(|| format!("failed to create signed terrain pack {}", path.display()))?;
    let mut header = Vec::with_capacity(HEADER_BYTES as usize);
    header.extend_from_slice(&GLOBAL_MAGIC);
    push_u32(&mut header, GLOBAL_VERSION);
    push_u32(&mut header, HEADER_BYTES);
    push_u32(&mut header, region_count);
    push_u32(&mut header, GLOBAL_INDEX_ENTRY_BYTES);
    push_u32(&mut header, PAYLOAD_BYTES);
    push_u32(&mut header, 0);
    push_u64(&mut header, u64::from(HEADER_BYTES));
    push_u64(&mut header, payload_offset);
    push_u64(&mut header, file_bytes);
    push_u64(&mut header, 0);
    file.write_all(&header)
        .context("failed to write signed terrain pack header")?;

    for (index, (tile, (_, sha256))) in tiles.iter().zip(&encoded).enumerate() {
        push_i64_to(&mut file, tile.region.x)?;
        push_i64_to(&mut file, tile.region.z)?;
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
        "signed terrain pack index exceeded payload offset"
    );
    file.write_all(&vec![0u8; (payload_offset - position) as usize])?;
    for (bytes, _) in encoded {
        file.write_all(&bytes)?;
    }
    file.flush()
        .context("failed to flush signed terrain pack")?;
    drop(file);

    let pack = GlobalTerrainPack::open(path)?;
    Ok(pack.metadata().clone())
}

pub fn encode_global_tile(tile: &GlobalTerrainTile) -> Result<[u8; PAYLOAD_BYTES as usize]> {
    validate_global_tile(tile)?;
    let mut bytes = [0u8; PAYLOAD_BYTES as usize];
    bytes[0..8].copy_from_slice(&tile.region.x.to_le_bytes());
    bytes[8..16].copy_from_slice(&tile.region.z.to_le_bytes());
    for (index, height) in tile.heights.iter().enumerate() {
        let offset = HEIGHT_OFFSET + index * 2;
        bytes[offset..offset + 2].copy_from_slice(&height.to_le_bytes());
    }
    bytes[MATERIAL_OFFSET..MATERIAL_OFFSET + MATERIAL_COUNT].copy_from_slice(&tile.materials);
    Ok(bytes)
}

pub fn validate_global_neighbor_edges<'a>(
    tiles: impl IntoIterator<Item = &'a GlobalTerrainTile>,
) -> GlobalEdgeValidation {
    let tiles = tiles
        .into_iter()
        .map(|tile| (tile.region, tile))
        .collect::<BTreeMap<_, _>>();
    let mut result = GlobalEdgeValidation {
        neighbor_edges: 0,
        sample_comparisons: 0,
        mismatch_count: 0,
        first_mismatch: None,
    };
    for (&region, tile) in &tiles {
        if let Some(x) = region.x.checked_add(1) {
            let neighbor_region = GlobalRegion::new(x, region.z);
            if let Some(neighbor) = tiles.get(&neighbor_region) {
                compare_global_edge(&mut result, tile, neighbor, "x", |sample| {
                    (sample * SAMPLE_SIDE + CELL_SIDE, sample * SAMPLE_SIDE)
                });
            }
        }
        if let Some(z) = region.z.checked_add(1) {
            let neighbor_region = GlobalRegion::new(region.x, z);
            if let Some(neighbor) = tiles.get(&neighbor_region) {
                compare_global_edge(&mut result, tile, neighbor, "z", |sample| {
                    (CELL_SIDE * SAMPLE_SIDE + sample, sample)
                });
            }
        }
    }
    result
}

fn validate_global_tile(tile: &GlobalTerrainTile) -> Result<()> {
    ensure!(
        tile.materials
            .iter()
            .all(|value| *value < MATERIAL_PALETTE_SIZE),
        "signed terrain region ({},{}) has a material outside the fixed palette",
        tile.region.x,
        tile.region.z
    );
    Ok(())
}

fn decode_global_payload(bytes: &[u8; PAYLOAD_BYTES as usize]) -> Result<GlobalTerrainTile> {
    ensure!(
        bytes[HEIGHT_OFFSET + HEIGHT_COUNT * 2..MATERIAL_OFFSET]
            .iter()
            .all(|byte| *byte == 0),
        "signed terrain payload alignment padding is nonzero"
    );
    ensure!(
        bytes[MATERIAL_OFFSET + MATERIAL_COUNT..]
            .iter()
            .all(|byte| *byte == 0),
        "signed terrain payload trailing padding is nonzero"
    );
    let mut heights = [0i16; HEIGHT_COUNT];
    for (index, height) in heights.iter_mut().enumerate() {
        let offset = HEIGHT_OFFSET + index * 2;
        *height = i16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("i16 slice"));
    }
    let mut materials = [0u8; MATERIAL_COUNT];
    materials.copy_from_slice(&bytes[MATERIAL_OFFSET..MATERIAL_OFFSET + MATERIAL_COUNT]);
    let tile = GlobalTerrainTile {
        region: GlobalRegion::new(i64_at(bytes, 0), i64_at(bytes, 8)),
        heights,
        materials,
    };
    validate_global_tile(&tile)?;
    Ok(tile)
}

fn compare_global_edge(
    result: &mut GlobalEdgeValidation,
    tile: &GlobalTerrainTile,
    neighbor: &GlobalTerrainTile,
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
            result.first_mismatch.get_or_insert(GlobalEdgeMismatch {
                region: tile.region,
                neighbor: neighbor.region,
                axis,
                sample_index: sample as u32,
                value,
                neighbor_value,
            });
        }
    }
}

fn i64_at(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("i64 slice"))
}

fn push_i64_to(writer: &mut impl Write, value: i64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}
