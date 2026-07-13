use std::collections::BTreeSet;

use anyhow::Result;
use glam::{Vec3, Vec4};
use meshlet_catalog::Catalog;
use serde::Serialize;

use crate::load::LoadConfig;
use crate::resident::{active_region_ids, generate_region};
use crate::scene::SceneState;

use super::renderer::SkeletalSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadCounts {
    pub visible: u32,
    pub rejected: u32,
    pub animated: u32,
    pub static_count: u32,
    pub active_poses: u32,
    pub reused_poses: u32,
    pub evaluated_bones: u32,
    pub lod_counts: [u32; 3],
    pub meshlets: u32,
    pub emitted_vertices: u32,
    pub emitted_triangles: u32,
    pub skin_influences: u32,
    pub observed_archetype_mask: u32,
}

pub fn evaluate(
    catalog: &Catalog,
    settings: SkeletalSettings,
    config: LoadConfig,
    scene: &SceneState,
    width: u32,
    height: u32,
    ground_numerators: Option<&[i32]>,
) -> Result<WorkloadCounts> {
    let matrix = scene.view_projection(width as f32 / height as f32);
    let mut counts = WorkloadCounts {
        visible: 0,
        rejected: 0,
        animated: 0,
        static_count: 0,
        active_poses: 0,
        reused_poses: 0,
        evaluated_bones: 0,
        lod_counts: [0; 3],
        meshlets: 0,
        emitted_vertices: 0,
        emitted_triangles: 0,
        skin_influences: 0,
        observed_archetype_mask: 0,
    };
    let mut shared_poses = BTreeSet::new();
    for (active_index, region_id) in active_region_ids(config)?.into_iter().enumerate() {
        for (local_index, instance) in generate_region(region_id).into_iter().enumerate() {
            let logical_index =
                active_index * crate::load::INSTANCES_PER_REGION as usize + local_index;
            let ground =
                ground_numerators.map_or(0.0, |values| values[logical_index] as f32 / 512.0);
            let center =
                Vec3::from_array(instance.position) + Vec3::Y * (ground + instance.height * 0.5);
            let clip = matrix * Vec4::new(center.x, center.y, center.z, 1.0);
            let stable_key = region_id * 1024 + local_index as u32;
            let archetype = stable_key & 7;
            let visible = clip.w > 0.0
                && clip.x.abs() <= clip.w
                && clip.y.abs() <= clip.w
                && clip.z >= 0.0
                && clip.z <= clip.w;
            if !visible {
                counts.rejected += 1;
                continue;
            }
            let lod = settings.forced_lod.unwrap_or(if clip.w < 42.0 {
                0
            } else if clip.w < 70.0 {
                1
            } else {
                2
            });
            let descriptor = catalog.lod(archetype, lod);
            counts.visible += 1;
            counts.lod_counts[lod as usize] += 1;
            counts.meshlets += descriptor.meshlet_count;
            counts.emitted_vertices += descriptor.vertex_count;
            counts.emitted_triangles += descriptor.primitive_count;
            counts.observed_archetype_mask |= 1 << archetype;
            if stable_key % 100 < settings.animated_percent {
                counts.animated += 1;
                counts.skin_influences += descriptor.vertex_count * 4;
                if !settings.unique_poses {
                    shared_poses.insert(pose_key(stable_key, archetype, settings));
                }
            } else {
                counts.static_count += 1;
            }
        }
    }
    counts.active_poses = if settings.unique_poses {
        counts.animated
    } else {
        shared_poses.len() as u32
    };
    counts.reused_poses = counts.animated.saturating_sub(counts.active_poses);
    counts.evaluated_bones = counts.active_poses * settings.bone_count;
    Ok(counts)
}

pub fn pose_key(stable_key: u32, archetype: u32, settings: SkeletalSettings) -> u32 {
    archetype * 64 + pose_phase(stable_key, settings)
}

pub fn pose_phase(stable_key: u32, settings: SkeletalSettings) -> u32 {
    let bucket = ((stable_key >> 3) + settings.time_tick) % settings.phase_count;
    bucket * (64 / settings.phase_count)
}
