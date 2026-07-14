use animation_catalog::{BONE_COUNT, CLIP_COUNT, Catalog as AnimationCatalog, unpack_bytes};
use anyhow::{Result, ensure};
use glam::{Vec3, Vec4};
use meshlet_catalog::Catalog;
use serde::Serialize;

use crate::rendering::terrain::TerrainProjection;
use crate::resident::{InstanceRecord, PresentationRecord};
use crate::scene::SceneState;

use super::super::resources::CANDIDATE_CAPACITY;
use super::{
    BOUND_RADIAL_BIAS, BOUND_RADIAL_SCALE, BOUND_VERTICAL_PAD, DEPTH_BIAS, HierarchyMip,
    IMPORTED_BOUND_RADIAL, PIXEL_EXPANSION,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcclusionOracle {
    pub source_visible: u32,
    pub survivors: u32,
    pub occluded: u32,
    pub source_meshlets: u32,
    pub submitted_meshlets: u32,
    pub source_vertices: u32,
    pub submitted_vertices: u32,
    pub source_triangles: u32,
    pub submitted_triangles: u32,
    pub source_skin_influences: u32,
    pub submitted_skin_influences: u32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundProof {
    pub tested_vertex_poses: u64,
    pub maximum_radial_extent: f32,
    pub unique_variant_margin: f32,
    pub maximum_bound_extent: f32,
    pub minimum_radial_slack: f32,
    pub minimum_vertical_pad: f32,
    pub maximum_vertical_pad: f32,
    pub radial_scale: f32,
    pub radial_bias: f32,
    pub imported_radial_bound: f32,
    pub vertical_pad: f32,
}

pub fn validate_fixture_bound(mesh: &Catalog, animation: &AnimationCatalog) -> Result<BoundProof> {
    const BONE_COUNTS: [u32; 4] = [16, 32, 64, 128];
    const HEIGHTS: [f32; 2] = [0.7, 3.0];
    const UNIQUE_VARIANT_MARGIN: f32 = 0.05;
    let mut maximum_radial_extent = 0.0f32;
    let mut minimum_vertical_pad = 0.0f32;
    let mut maximum_vertical_pad = 0.0f32;
    let mut minimum_radial_slack = f32::MAX;
    let mut tested_vertex_poses = 0u64;
    let imported_range =
        mesh.imported.vertex_start..mesh.imported.vertex_start + mesh.imported.vertex_count;
    for clip in 0..CLIP_COUNT {
        for phase in 0..64 {
            for bone_count in BONE_COUNTS {
                let palette = animation.evaluate_pose(clip, phase, bone_count, 0);
                for height in HEIGHTS {
                    for (vertex_index, vertex) in mesh.vertices.iter().enumerate() {
                        let mut local = vertex.position;
                        local[1] *= height;
                        let binding = animation.skin_bindings[vertex_index];
                        let mut skinned = Vec3::ZERO;
                        for (bone, weight) in unpack_bytes(binding.indices)
                            .into_iter()
                            .zip(unpack_bytes(binding.weights))
                        {
                            skinned += Vec3::from_array(
                                palette[usize::from(bone) % bone_count as usize]
                                    .transform_point(local[..3].try_into().unwrap()),
                            ) * (f32::from(weight) / 255.0);
                        }
                        maximum_radial_extent = maximum_radial_extent
                            .max(Vec3::new(skinned.x, 0.0, skinned.z).length());
                        let radial = Vec3::new(skinned.x, 0.0, skinned.z).length();
                        let radial_bound = if imported_range.contains(&(vertex_index as u32)) {
                            IMPORTED_BOUND_RADIAL
                        } else {
                            BOUND_RADIAL_SCALE * height + BOUND_RADIAL_BIAS
                        };
                        minimum_radial_slack =
                            minimum_radial_slack.min(radial_bound - radial - UNIQUE_VARIANT_MARGIN);
                        minimum_vertical_pad = minimum_vertical_pad.max(-skinned.y);
                        maximum_vertical_pad = maximum_vertical_pad.max(skinned.y - height);
                        tested_vertex_poses += 1;
                    }
                }
            }
        }
    }
    let maximum_bound_extent = BOUND_RADIAL_SCALE * HEIGHTS[1] + BOUND_RADIAL_BIAS;
    ensure!(
        minimum_radial_slack >= 0.0,
        "generated skeletal radial extent exceeds affine bound by {}",
        -minimum_radial_slack
    );
    ensure!(
        minimum_vertical_pad <= BOUND_VERTICAL_PAD && maximum_vertical_pad <= BOUND_VERTICAL_PAD,
        "generated skeletal vertical extent [{minimum_vertical_pad}, {maximum_vertical_pad}] exceeds {BOUND_VERTICAL_PAD}"
    );
    ensure!(
        animation.bones.len() == BONE_COUNT as usize,
        "animation hierarchy is incomplete"
    );
    Ok(BoundProof {
        tested_vertex_poses,
        maximum_radial_extent,
        unique_variant_margin: UNIQUE_VARIANT_MARGIN,
        maximum_bound_extent,
        minimum_radial_slack,
        minimum_vertical_pad,
        maximum_vertical_pad,
        radial_scale: BOUND_RADIAL_SCALE,
        radial_bias: BOUND_RADIAL_BIAS,
        imported_radial_bound: IMPORTED_BOUND_RADIAL,
        vertical_pad: BOUND_VERTICAL_PAD,
    })
}

pub struct QueryInput<'a> {
    pub mesh: &'a Catalog,
    pub scene: &'a SceneState,
    pub projection: TerrainProjection,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub instance_records: &'a [Vec<InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<PresentationRecord>],
    pub extent: [u32; 2],
    pub hierarchy: &'a [HierarchyMip],
    pub history_queried: bool,
}

pub fn evaluate(input: QueryInput<'_>) -> Result<(OcclusionOracle, Vec<u32>)> {
    let [width, height] = input.extent;
    ensure!(
        input.instance_records.len() == input.local_ids.len()
            && input.instance_records.len() == input.presentations.len()
            && input.instance_records.len() == input.projection.active_count(),
        "occlusion canonical payload shapes differ"
    );
    ensure!(
        input.ground_numerators.len()
            == input.instance_records.len() * crate::load::INSTANCES_PER_REGION as usize,
        "occlusion ground plane shape differs from the canonical payload"
    );
    let camera = input.projection.camera(input.scene.camera());
    let matrix = crate::scene::view_projection(camera, width as f32 / height as f32);
    let mut oracle = OcclusionOracle::default();
    let mut mask = vec![0; CANDIDATE_CAPACITY as usize];
    for (region_ordinal, instances) in input.instance_records.iter().enumerate() {
        ensure!(
            instances.len() == input.local_ids[region_ordinal].len()
                && instances.len() == input.presentations[region_ordinal].len(),
            "occlusion canonical triple counts differ"
        );
        for (local_index, instance) in instances.iter().copied().enumerate() {
            let candidate = region_ordinal as u32 * 1024 + local_index as u32;
            let logical_index = candidate as usize;
            let ground =
                input.ground_numerators[logical_index] as f32 / input.ground_denominator as f32;
            let mut position = input
                .projection
                .position(region_ordinal, instance.position)?;
            position[1] += ground;
            let center = Vec3::from_array(position) + Vec3::Y * instance.height * 0.5;
            let clip = matrix * center.extend(1.0);
            let presentation = input.presentations[region_ordinal][local_index];
            let archetype = presentation.archetype;
            let visible = clip.w > 0.0
                && clip.x.abs() <= clip.w
                && clip.y.abs() <= clip.w
                && clip.z >= 0.0
                && clip.z <= clip.w;
            if !visible {
                continue;
            }
            let lod = if clip.w < 42.0 {
                0
            } else if clip.w < 70.0 {
                1
            } else {
                2
            };
            let descriptor = input.mesh.lod(archetype, lod);
            let animated = presentation.is_animated();
            oracle.source_visible += 1;
            oracle.source_meshlets += descriptor.meshlet_count;
            oracle.source_vertices += descriptor.vertex_count;
            oracle.source_triangles += descriptor.primitive_count;
            if animated {
                oracle.source_skin_influences += descriptor.vertex_count * 4;
            }
            let occluded = input.history_queried
                && query_occluded(
                    matrix,
                    position,
                    instance.height,
                    archetype,
                    width,
                    height,
                    input.hierarchy,
                );
            if occluded {
                oracle.occluded += 1;
                mask[candidate as usize] = 2;
            } else {
                oracle.survivors += 1;
                oracle.submitted_meshlets += descriptor.meshlet_count;
                oracle.submitted_vertices += descriptor.vertex_count;
                oracle.submitted_triangles += descriptor.primitive_count;
                if animated {
                    oracle.submitted_skin_influences += descriptor.vertex_count * 4;
                }
                mask[candidate as usize] = 1;
            }
        }
    }
    Ok((oracle, mask))
}

fn query_occluded(
    matrix: glam::Mat4,
    position: [f32; 3],
    height: f32,
    archetype: u32,
    width: u32,
    screen_height: u32,
    hierarchy: &[HierarchyMip],
) -> bool {
    let center = Vec3::from_array(position) + Vec3::Y * height * 0.5;
    let half_xz = if archetype == meshlet_catalog::IMPORTED_ARCHETYPE {
        IMPORTED_BOUND_RADIAL
    } else {
        BOUND_RADIAL_SCALE * height + BOUND_RADIAL_BIAS
    };
    let half_y = height * 0.5 + BOUND_VERTICAL_PAD;
    let mut minimum = Vec3::new(width as f32, screen_height as f32, f32::MAX);
    let mut maximum = Vec3::ZERO;
    for corner in 0..8 {
        let sign = Vec3::new(
            if corner & 1 == 0 { -1.0 } else { 1.0 },
            if corner & 2 == 0 { -1.0 } else { 1.0 },
            if corner & 4 == 0 { -1.0 } else { 1.0 },
        );
        let world = center + sign * Vec3::new(half_xz, half_y, half_xz);
        let clip: Vec4 = matrix * world.extend(1.0);
        if clip.w <= 0.0 {
            return false;
        }
        let ndc = clip.truncate() / clip.w;
        if !(0.0..=1.0).contains(&ndc.z) {
            return false;
        }
        let pixel = Vec3::new(
            (ndc.x * 0.5 + 0.5) * width as f32,
            (-ndc.y * 0.5 + 0.5) * screen_height as f32,
            ndc.z,
        );
        minimum = minimum.min(pixel);
        maximum = maximum.max(pixel);
    }
    let last_x = width as i32 - 1;
    let last_y = screen_height as i32 - 1;
    let low_x = (minimum.x.floor() as i32 - PIXEL_EXPANSION as i32).clamp(0, last_x) as u32;
    let low_y = (minimum.y.floor() as i32 - PIXEL_EXPANSION as i32).clamp(0, last_y) as u32;
    let high_x = (maximum.x.ceil() as i32 + PIXEL_EXPANSION as i32).clamp(0, last_x) as u32;
    let high_y = (maximum.y.ceil() as i32 + PIXEL_EXPANSION as i32).clamp(0, last_y) as u32;
    if high_x < low_x || high_y < low_y {
        return false;
    }
    let largest = (high_x - low_x + 1).max(high_y - low_y + 1);
    let mip = if largest <= 1 {
        0
    } else {
        u32::BITS - (largest - 1).leading_zeros()
    }
    .min(hierarchy.len() as u32 - 1);
    let level = &hierarchy[mip as usize];
    let low = [
        (low_x >> mip).min(level.width - 1),
        (low_y >> mip).min(level.height - 1),
    ];
    let high = [
        (high_x >> mip).min(level.width - 1),
        (high_y >> mip).min(level.height - 1),
    ];
    let minimum_depth = [
        read_depth(level, low[0], low[1]),
        read_depth(level, high[0], low[1]),
        read_depth(level, low[0], high[1]),
        read_depth(level, high[0], high[1]),
    ]
    .into_iter()
    .min()
    .unwrap();
    minimum_depth != 0 && maximum.z + DEPTH_BIAS < f32::from_bits(minimum_depth)
}

fn read_depth(mip: &HierarchyMip, x: u32, y: u32) -> u32 {
    let offset = ((y * mip.width + x) * 4) as usize;
    u32::from_le_bytes(mip.bytes[offset..offset + 4].try_into().unwrap())
}
