use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::load::LoadConfig;
use crate::terrain::TerrainAssignment;

use super::{
    PATCH_GROUP_COUNT, PATCHES_PER_REGION, QUERY_COUNT, STATS_BYTES, TERRAIN_REVISION,
    TRIANGLES_PER_PATCH, TerrainRenderer, VERTICES_PER_PATCH,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainProbe {
    revision: &'static str,
    config: LoadConfig,
    generation: u64,
    active_mapping: Vec<TerrainAssignment>,
    active_mapping_sha256: String,
    payload_sha256: String,
    cpu_edges: terrain_format::EdgeValidation,
    gpu_edges: GpuEdges,
    geometry: TerrainGeometry,
    submission: TerrainSubmission,
    resources: TerrainResources,
    timing: TerrainTiming,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GpuEdges {
    neighbor_edges: u32,
    sample_comparisons: u32,
    mismatch_count: u32,
    first_mismatch: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TerrainGeometry {
    fixed_patch_groups: u32,
    emitted_patches: u32,
    inactive_groups: u32,
    vertices: u32,
    triangles: u32,
    min_height: i32,
    max_height: i32,
    oracle_patches: u32,
    oracle_vertices: u32,
    oracle_triangles: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TerrainSubmission {
    mesh_dispatch_count: u32,
    mesh_dispatch_groups: [u32; 3],
    seam_dispatch_count: u32,
    seam_dispatch_groups: [u32; 3],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TerrainResources {
    cache_capacity: usize,
    active_capacity: usize,
    payload_bytes: u32,
    stats_bytes: u64,
    seam_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TerrainTiming {
    seam_ms: f64,
    raster_ms: f64,
    total_ms: f64,
}

impl TerrainRenderer {
    pub unsafe fn read_probe(&self) -> Result<TerrainProbe> {
        let snapshot = self
            .published
            .as_ref()
            .context("terrain is not published")?;
        let stats = unsafe { super::super::resident::read_values::<u32>(&self.stats_readback, 8) }?;
        let seams = unsafe { super::super::resident::read_values::<u32>(&self.seams_readback, 8) }?;
        let timestamps = unsafe {
            super::super::resident::read_values::<u64>(
                &self.timestamp_readback,
                QUERY_COUNT as usize,
            )
        }?;
        let active_count = snapshot.config.active_region_count();
        let oracle_patches = active_count * PATCHES_PER_REGION;
        let oracle_vertices = oracle_patches * VERTICES_PER_PATCH;
        let oracle_triangles = oracle_patches * TRIANGLES_PER_PATCH;
        ensure!(
            stats[0] == PATCH_GROUP_COUNT,
            "terrain fixed patch group count mismatch"
        );
        ensure!(
            stats[1] == oracle_patches,
            "terrain emitted patch count mismatch"
        );
        ensure!(
            stats[2] == PATCH_GROUP_COUNT - oracle_patches,
            "terrain inactive group count mismatch"
        );
        ensure!(stats[3] == oracle_vertices, "terrain vertex count mismatch");
        ensure!(
            stats[4] == oracle_triangles,
            "terrain triangle count mismatch"
        );
        let cpu_edges = terrain_format::validate_neighbor_edges(&snapshot.tiles);
        ensure!(
            cpu_edges.mismatch_count == 0,
            "published terrain CPU edge mismatch"
        );
        ensure!(
            seams[0] == cpu_edges.neighbor_edges,
            "terrain GPU edge count mismatch"
        );
        ensure!(
            seams[1] == cpu_edges.sample_comparisons,
            "terrain GPU comparison count mismatch"
        );
        ensure!(seams[2] == 0, "terrain GPU edge mismatch");
        let mut mapping_hash = Sha256::new();
        for entry in &snapshot.active {
            mapping_hash.update(entry.slot.to_le_bytes());
            mapping_hash.update(entry.region_id.to_le_bytes());
        }
        let mut payload_hash = Sha256::new();
        for tile in &snapshot.tiles {
            payload_hash.update(terrain_format::encode_tile(tile)?);
        }
        let milliseconds = |start: usize, end: usize| {
            timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
                / self.timestamp_frequency as f64
        };
        Ok(TerrainProbe {
            revision: TERRAIN_REVISION,
            config: snapshot.config,
            generation: snapshot.generation,
            active_mapping: snapshot.active.clone(),
            active_mapping_sha256: format!("{:x}", mapping_hash.finalize()),
            payload_sha256: format!("{:x}", payload_hash.finalize()),
            cpu_edges,
            gpu_edges: GpuEdges {
                neighbor_edges: seams[0],
                sample_comparisons: seams[1],
                mismatch_count: seams[2],
                first_mismatch: (seams[3] != u32::MAX).then_some(seams[3]),
            },
            geometry: TerrainGeometry {
                fixed_patch_groups: stats[0],
                emitted_patches: stats[1],
                inactive_groups: stats[2],
                vertices: stats[3],
                triangles: stats[4],
                min_height: stats[6] as i32,
                max_height: stats[7] as i32,
                oracle_patches,
                oracle_vertices,
                oracle_triangles,
            },
            submission: TerrainSubmission {
                mesh_dispatch_count: 1,
                mesh_dispatch_groups: [PATCH_GROUP_COUNT, 1, 1],
                seam_dispatch_count: 1,
                seam_dispatch_groups: [25, 2, 1],
            },
            resources: TerrainResources {
                cache_capacity: super::cache::TERRAIN_CACHE_CAPACITY,
                active_capacity: super::cache::TERRAIN_ACTIVE_CAPACITY,
                payload_bytes: terrain_format::PAYLOAD_BYTES,
                stats_bytes: STATS_BYTES,
                seam_bytes: STATS_BYTES,
            },
            timing: TerrainTiming {
                seam_ms: milliseconds(0, 1),
                raster_ms: milliseconds(1, 2),
                total_ms: milliseconds(0, 2),
            },
        })
    }
}
