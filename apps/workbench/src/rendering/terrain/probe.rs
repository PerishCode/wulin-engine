use anyhow::{Context, Result, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::load::LoadConfig;
use crate::scene::SceneState;
use crate::terrain::{GlobalTerrainConfig, TerrainAssignment, TerrainSourceNamespace};

use super::{
    LOD_STATS_BYTES, PATCH_GROUP_COUNT, QUERY_COUNT, STATS_BYTES, TERRAIN_REVISION, TerrainRenderer,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainProbe {
    revision: &'static str,
    config: LoadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_addressing: Option<GlobalAddressEvidence>,
    generation: u64,
    active_mapping: Vec<TerrainAssignment>,
    active_mapping_sha256: String,
    payload_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_content: Option<GlobalContentEvidence>,
    cpu_edges: terrain_format::EdgeValidation,
    gpu_edges: GpuEdges,
    geometry: TerrainGeometry,
    submission: TerrainSubmission,
    resources: TerrainResources,
    timing: TerrainTiming,
    lod: TerrainLodEvidence,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GlobalAddressEvidence {
    config: GlobalTerrainConfig,
    mapping_sha256: String,
    entry_count: usize,
    duplicate_global_count: usize,
    mismatch_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GlobalContentEvidence {
    source_namespace: TerrainSourceNamespace,
    content_sha256: String,
    region_count: usize,
}

impl TerrainProbe {
    pub(in crate::rendering) fn total_gpu_ms(&self) -> f64 {
        self.timing.total_ms
    }
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TerrainLodEvidence {
    oracle: super::lod::TerrainLodOracle,
    gpu: GpuLod,
    submission: LodSubmission,
    resources: LodResources,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GpuLod {
    lod_counts: [u32; 3],
    patch_edges: Option<u32>,
    same_lod_edges: Option<u32>,
    transition_edges: Option<u32>,
    adjusted_vertices: Option<u32>,
    sample_comparisons: Option<u32>,
    max_lod_delta: Option<u32>,
    mismatch_count: Option<u32>,
    first_mismatch: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LodSubmission {
    dispatch_count: u32,
    dispatch_groups: [u32; 3],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LodResources {
    stats_bytes: u64,
}

impl TerrainRenderer {
    pub unsafe fn read_probe(&self, scene: &SceneState) -> Result<TerrainProbe> {
        let snapshot = self
            .published
            .as_ref()
            .context("terrain is not published")?;
        let stats = unsafe { super::super::resident::read_values::<u32>(&self.stats_readback, 8) }?;
        let seams = unsafe { super::super::resident::read_values::<u32>(&self.seams_readback, 8) }?;
        let lod_stats =
            unsafe { super::super::resident::read_values::<u32>(&self.lod_stats_readback, 16) }?;
        let timestamps = unsafe {
            super::super::resident::read_values::<u64>(
                &self.timestamp_readback,
                QUERY_COUNT as usize,
            )
        }?;
        let lod_oracle = super::lod::evaluate(
            snapshot.config,
            &snapshot.active,
            &snapshot.tiles,
            scene.camera(),
            self.lod_settings,
        )?;
        let oracle_patches = lod_oracle.geometry.patches;
        let oracle_vertices = lod_oracle.geometry.vertices;
        let oracle_triangles = lod_oracle.geometry.triangles;
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
        ensure!(
            lod_stats[..3] == lod_oracle.lod_counts,
            "terrain GPU LOD counts mismatch"
        );
        if self.lod_settings.enabled {
            ensure!(
                lod_stats[3] == lod_oracle.edges.patch_edges,
                "terrain LOD edge count mismatch"
            );
            ensure!(
                lod_stats[4] == lod_oracle.edges.same_lod_edges,
                "terrain same-LOD edge count mismatch"
            );
            ensure!(
                lod_stats[5] == lod_oracle.edges.transition_edges,
                "terrain transition edge count mismatch"
            );
            ensure!(
                lod_stats[6] == lod_oracle.edges.adjusted_vertices,
                "terrain adjusted vertex count mismatch"
            );
            ensure!(
                lod_stats[7] == lod_oracle.edges.sample_comparisons,
                "terrain LOD sample count mismatch"
            );
            ensure!(
                lod_stats[8] == lod_oracle.edges.max_lod_delta,
                "terrain maximum LOD delta mismatch"
            );
            ensure!(lod_stats[9] == 0, "terrain LOD geometric mismatch");
        }
        let mut mapping_hash = Sha256::new();
        for entry in &snapshot.active {
            mapping_hash.update(entry.slot.to_le_bytes());
            mapping_hash.update(entry.region_id.to_le_bytes());
        }
        let global_addressing = snapshot
            .global_config
            .map(|config| global_evidence(config, &snapshot.active))
            .transpose()?;
        let mut payload_hash = Sha256::new();
        for tile in &snapshot.tiles {
            payload_hash.update(terrain_format::encode_tile(tile)?);
        }
        let global_content = snapshot
            .report
            .source_namespace
            .map(|namespace| global_content_evidence(namespace, &snapshot.active, &snapshot.tiles))
            .transpose()?;
        let milliseconds = |start: usize, end: usize| {
            timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
                / self.timestamp_frequency as f64
        };
        Ok(TerrainProbe {
            revision: TERRAIN_REVISION,
            config: snapshot.config,
            global_addressing,
            generation: snapshot.generation,
            active_mapping: snapshot.active.clone(),
            active_mapping_sha256: format!("{:x}", mapping_hash.finalize()),
            payload_sha256: format!("{:x}", payload_hash.finalize()),
            global_content,
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
            lod: TerrainLodEvidence {
                oracle: lod_oracle,
                gpu: GpuLod {
                    lod_counts: [lod_stats[0], lod_stats[1], lod_stats[2]],
                    patch_edges: self.lod_settings.enabled.then_some(lod_stats[3]),
                    same_lod_edges: self.lod_settings.enabled.then_some(lod_stats[4]),
                    transition_edges: self.lod_settings.enabled.then_some(lod_stats[5]),
                    adjusted_vertices: self.lod_settings.enabled.then_some(lod_stats[6]),
                    sample_comparisons: self.lod_settings.enabled.then_some(lod_stats[7]),
                    max_lod_delta: self.lod_settings.enabled.then_some(lod_stats[8]),
                    mismatch_count: self.lod_settings.enabled.then_some(lod_stats[9]),
                    first_mismatch: (self.lod_settings.enabled && lod_stats[10] != u32::MAX)
                        .then_some(lod_stats[10]),
                },
                submission: LodSubmission {
                    dispatch_count: u32::from(self.lod_settings.enabled),
                    dispatch_groups: [PATCH_GROUP_COUNT, 2, 1],
                },
                resources: LodResources {
                    stats_bytes: LOD_STATS_BYTES,
                },
            },
        })
    }
}

fn global_content_evidence(
    source_namespace: TerrainSourceNamespace,
    active: &[TerrainAssignment],
    tiles: &[terrain_format::TerrainTile],
) -> Result<GlobalContentEvidence> {
    ensure!(
        active.len() == tiles.len(),
        "canonical terrain content assignment count mismatch"
    );
    let mut hash = Sha256::new();
    for (assignment, tile) in active.iter().zip(tiles) {
        let global = assignment
            .global_region
            .context("canonical terrain content has no global assignment")?;
        hash.update(global.x.to_le_bytes());
        hash.update(global.z.to_le_bytes());
        for height in tile.heights {
            hash.update(height.to_le_bytes());
        }
        hash.update(tile.materials);
    }
    Ok(GlobalContentEvidence {
        source_namespace,
        content_sha256: format!("{:x}", hash.finalize()),
        region_count: tiles.len(),
    })
}

fn global_evidence(
    config: GlobalTerrainConfig,
    active: &[TerrainAssignment],
) -> Result<GlobalAddressEvidence> {
    let expected = config.addressed_regions()?;
    let mut mapping_hash = Sha256::new();
    let mut globals = std::collections::BTreeSet::new();
    let mut mismatch_count = 0;
    for (index, assignment) in active.iter().enumerate() {
        let global = assignment
            .global_region
            .context("global terrain assignment has no signed key")?;
        mapping_hash.update(global.x.to_le_bytes());
        mapping_hash.update(global.z.to_le_bytes());
        mapping_hash.update(assignment.region_id.to_le_bytes());
        if expected.get(index).is_none_or(|region| {
            region.global_region != global || region.local_region_id != assignment.region_id
        }) {
            mismatch_count += 1;
        }
        globals.insert(global);
    }
    mismatch_count += expected.len().abs_diff(active.len());
    Ok(GlobalAddressEvidence {
        config,
        mapping_sha256: format!("{:x}", mapping_hash.finalize()),
        entry_count: active.len(),
        duplicate_global_count: active.len() - globals.len(),
        mismatch_count,
    })
}
