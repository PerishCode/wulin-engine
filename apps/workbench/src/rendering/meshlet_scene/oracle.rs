use anyhow::Result;
use glam::{Vec3, Vec4};
use meshlet_catalog::Catalog;
use serde::Serialize;

use crate::load::LoadConfig;
use crate::resident::{active_region_ids, generate_region};
use crate::scene::SceneState;

use super::renderer::MeshletSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadCounts {
    pub visible: u32,
    pub rejected: u32,
    pub lod_counts: [u32; 3],
    pub meshlets: u32,
    pub emitted_vertices: u32,
    pub emitted_triangles: u32,
    pub observed_archetype_mask: u32,
}

pub fn evaluate(
    catalog: &Catalog,
    settings: MeshletSettings,
    config: LoadConfig,
    scene: &SceneState,
    width: u32,
    height: u32,
) -> Result<WorkloadCounts> {
    let matrix = scene.view_projection(width as f32 / height as f32);
    let mut counts = WorkloadCounts {
        visible: 0,
        rejected: 0,
        lod_counts: [0; 3],
        meshlets: 0,
        emitted_vertices: 0,
        emitted_triangles: 0,
        observed_archetype_mask: 0,
    };
    for region_id in active_region_ids(config)? {
        for (local_index, instance) in generate_region(region_id).into_iter().enumerate() {
            let center = Vec3::from_array(instance.position) + Vec3::Y * instance.height * 0.5;
            let clip = matrix * Vec4::new(center.x, center.y, center.z, 1.0);
            let stable_key = region_id * 1024 + local_index as u32;
            let archetype = stable_key & 7;
            let visible = clip.w > 0.0
                && clip.x.abs() <= clip.w
                && clip.y.abs() <= clip.w
                && clip.z >= 0.0
                && clip.z <= clip.w
                && settings.archetype_mask & (1 << archetype) != 0;
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
        }
    }
    Ok(counts)
}
