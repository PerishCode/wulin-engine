use animation_catalog::{Catalog as AnimationCatalog, unpack_bytes};
use anyhow::{Context, Result, ensure};
use glam::Vec3;
use meshlet_catalog::Catalog as MeshletCatalog;
use surface_catalog::{Catalog as SurfaceCatalog, TEXTURE_SIDE, decode_octahedral};

use crate::rendering::ActorRenderProjection;
use crate::rendering::terrain::TerrainProjection;
use crate::resident::{InstanceRecord, PresentationRecord, canonical_stable_key};

use super::super::oracle::pose_phase;
use super::super::renderer::SkeletalSettings;
use super::super::resources::ACTOR_CANDIDATE_INDEX;
use super::super::resources::actor::ActorVisibleCandidate;
use super::probe::SurfaceSample;
use super::renderer::SurfaceSettings;

pub struct OracleResult {
    pub expected_rgba8: u32,
    pub expected_texel: Option<[u32; 2]>,
    pub maximum_channel_delta: u32,
    pub expected_shadowed: Option<bool>,
    pub expected_shadow_texel: Option<[u32; 2]>,
}

#[derive(Clone, Copy)]
pub struct OracleInput<'a> {
    pub surface: &'a SurfaceCatalog,
    pub mesh: &'a MeshletCatalog,
    pub animation: &'a AnimationCatalog,
    pub skeletal_settings: SkeletalSettings,
    pub surface_settings: SurfaceSettings,
    pub instance_records: &'a [Vec<InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<PresentationRecord>],
    pub projection: TerrainProjection,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub shadow_depth: &'a [u8],
    pub background_color: [f32; 4],
    pub actor: Option<ActorRenderProjection>,
    pub object_target: Option<crate::rendering::ProjectedObjectTarget>,
}

pub fn shade(sample: &SurfaceSample, input: OracleInput<'_>) -> Result<OracleResult> {
    let (expected, expected_texel, expected_shadowed, expected_shadow_texel) =
        if let (Some(candidate), Some(primitive), Some(barycentrics)) = (
            sample.candidate_index,
            sample.primitive_index,
            sample.barycentrics,
        ) {
            let (color, texel, shadowed, shadow_texel) =
                shade_visible(candidate, primitive, barycentrics, sample, input)?;
            (color, Some(texel), Some(shadowed), Some(shadow_texel))
        } else {
            (pack_rgba8(input.background_color), None, None, None)
        };
    Ok(OracleResult {
        expected_rgba8: expected,
        expected_texel,
        maximum_channel_delta: channel_delta(expected, sample.rgba8),
        expected_shadowed,
        expected_shadow_texel,
    })
}

fn shade_visible(
    candidate: u32,
    primitive_index: u32,
    barycentrics: [f32; 3],
    sample: &SurfaceSample,
    input: OracleInput<'_>,
) -> Result<(u32, [u32; 2], bool, [u32; 2])> {
    let target_kind = if candidate == ACTOR_CANDIDATE_INDEX {
        None
    } else if let Some(target) = input.object_target {
        let active_index = (candidate / 1024) as usize;
        let local_index = (candidate % 1024) as usize;
        (active_index == target.active_index as usize
            && input
                .local_ids
                .get(active_index)
                .and_then(|ids| ids.get(local_index))
                .is_some_and(|local_id| *local_id == target.authored_local_id))
        .then_some(target.kind)
    } else {
        None
    };
    let (position, object_height, presentation, stable_identity) =
        if candidate == ACTOR_CANDIDATE_INDEX {
            let actor = input
                .actor
                .context("surface sample actor candidate has no frame actor")?;
            let visible =
                ActorVisibleCandidate::from_projection(actor, input.skeletal_settings.time_tick)?;
            (
                visible.position,
                visible.height,
                visible.presentation(),
                [visible.stable_identity_low, visible.stable_identity_high],
            )
        } else {
            let region_ordinal = (candidate / 1024) as usize;
            let local_index = (candidate % 1024) as usize;
            let region_records = input
                .instance_records
                .get(region_ordinal)
                .context("surface sample candidate has no active region")?;
            let instance = *region_records
                .get(local_index)
                .context("surface sample candidate has no physical record")?;
            let local_id = *input
                .local_ids
                .get(region_ordinal)
                .and_then(|ids| ids.get(local_index))
                .context("surface sample candidate has no authored local ID")?;
            let presentation = *input
                .presentations
                .get(region_ordinal)
                .and_then(|records| records.get(local_index))
                .context("surface sample candidate has no presentation record")?;
            let ground = input.ground_numerators[candidate as usize] as f32
                / input.ground_denominator as f32;
            let mut position = input
                .projection
                .position(region_ordinal, instance.position)?;
            position[1] += ground;
            (
                position,
                instance.height,
                presentation,
                [canonical_stable_key(instance.region_id, local_id), local_id],
            )
        };
    ensure!(
        sample.stable_identity == Some(stable_identity),
        "surface sample stable identity differs from its candidate address"
    );
    let palette = presentation.is_animated().then(|| {
        input.animation.evaluate_pose_for_archetype(
            presentation.archetype,
            presentation.animation_clip().unwrap(),
            pose_phase(presentation, input.skeletal_settings),
            input.skeletal_settings.bone_count,
            0,
        )
    });
    let primitive = input.surface.primitives[primitive_index as usize];
    let angle = presentation.yaw_q16 as f32 * std::f32::consts::TAU / 65_536.0;
    let (sine, cosine) = angle.sin_cos();
    let mut normals = [Vec3::ZERO; 3];
    let mut world_positions = [Vec3::ZERO; 3];
    let mut uvs = [[0.0; 2]; 3];
    for vertex in 0..3 {
        let vertex_index = primitive.vertex_indices[vertex] as usize;
        let surface = input.surface.vertices[vertex_index];
        let decoded = decode_octahedral([surface.oct_normal_uv[0], surface.oct_normal_uv[1]]);
        let imported = presentation.archetype == meshlet_catalog::IMPORTED_ARCHETYPE;
        let mut local = Vec3::from_array(
            input.mesh.vertices[vertex_index].position[..3]
                .try_into()
                .unwrap(),
        );
        let mut normal = Vec3::from_array(decoded);
        if !imported {
            local.y *= object_height;
            normal.y /= object_height;
        }
        if let Some(palette) = &palette {
            let binding = input.animation.skin_bindings[vertex_index];
            let indices = unpack_bytes(binding.indices);
            let weights = unpack_bytes(binding.weights);
            normal = indices
                .into_iter()
                .zip(weights)
                .fold(Vec3::ZERO, |sum, (bone, weight)| {
                    sum + transform_vector(
                        palette[usize::from(bone) % input.skeletal_settings.bone_count as usize],
                        normal,
                    ) * (f32::from(weight) / 255.0)
                });
            local = indices
                .into_iter()
                .zip(weights)
                .fold(Vec3::ZERO, |sum, (bone, weight)| {
                    sum + transform_point(
                        palette[usize::from(bone) % input.skeletal_settings.bone_count as usize],
                        local,
                    ) * (f32::from(weight) / 255.0)
                });
        }
        if imported {
            local.y *= object_height;
            normal.y /= object_height;
        }
        let rotated = Vec3::new(
            local.x * cosine - local.z * sine,
            local.y,
            local.x * sine + local.z * cosine,
        );
        world_positions[vertex] = Vec3::from_array(position) + rotated;
        normals[vertex] = Vec3::new(
            normal.x * cosine - normal.z * sine,
            normal.y,
            normal.x * sine + normal.z * cosine,
        )
        .normalize();
        uvs[vertex] = [surface.oct_normal_uv[2], surface.oct_normal_uv[3]];
    }
    let bary = Vec3::from_array(barycentrics);
    let normal = (normals[0] * bary.x + normals[1] * bary.y + normals[2] * bary.z).normalize();
    let world_position =
        world_positions[0] * bary.x + world_positions[1] * bary.y + world_positions[2] * bary.z;
    let uv = [
        uvs[0][0] * bary.x + uvs[1][0] * bary.y + uvs[2][0] * bary.z,
        uvs[0][1] * bary.x + uvs[1][1] * bary.y + uvs[2][1] * bary.z,
    ];
    let material_index = presentation.material;
    ensure!(
        sample.material_index == Some(material_index),
        "surface sample material differs from the CPU oracle"
    );
    let material = input.surface.materials[material_index as usize];
    let mip = input.surface_settings.mip_level;
    let side = (TEXTURE_SIDE >> mip).max(1);
    let x = ((wrap_uv(uv[0]) * side as f32) as u32).min(side - 1);
    let y = ((wrap_uv(uv[1]) * side as f32) as u32).min(side - 1);
    let texel_offset = (((material.texture_layer * side + y) * side + x) * 4) as usize;
    let texture = &input.surface.texture_mips[mip as usize][texel_offset..texel_offset + 4];
    let texture_rgb = Vec3::new(texture[0] as f32, texture[1] as f32, texture[2] as f32) / 255.0;
    let light_direction = Vec3::new(-0.45, 0.8, 0.3).normalize();
    let diffuse = normal.dot(light_direction).clamp(0.0, 1.0);
    let shadow_address = super::shadow::address(world_position)
        .context("surface sample is outside the fixed shadow projection")?;
    let stored_depth = super::shadow::stored_depth(input.shadow_depth, shadow_address.texel);
    let shadowed = super::shadow::is_shadowed(shadow_address.receiver_depth, stored_depth);
    let direct_visibility = if shadowed { 0.0 } else { 1.0 };
    let lighting = 0.22 + direct_visibility * diffuse * (0.78 - material.roughness * 0.18);
    let metallic_lift =
        direct_visibility * material.metallic * normal.y.clamp(0.0, 1.0).powi(4) * 0.25;
    let mut color =
        Vec3::from_array(material.base_color[..3].try_into().unwrap()) * texture_rgb * lighting
            + Vec3::splat(metallic_lift);
    if let Some(kind) = target_kind {
        color = match kind {
            crate::runtime::ObjectTargetFeedbackKind::Selected => {
                color * 0.45 + Vec3::new(1.0, 0.62, 0.08) * 0.55
            }
            crate::runtime::ObjectTargetFeedbackKind::Activated => {
                color * 0.30 + Vec3::new(0.12, 1.0, 0.32) * 0.70
            }
            crate::runtime::ObjectTargetFeedbackKind::Rejected => {
                color * 0.30 + Vec3::new(1.0, 0.12, 0.08) * 0.70
            }
        };
    }
    Ok((
        pack_rgba8([
            color.x.clamp(0.0, 1.0),
            color.y.clamp(0.0, 1.0),
            color.z.clamp(0.0, 1.0),
            1.0,
        ]),
        [x, y],
        shadowed,
        shadow_address.texel,
    ))
}

fn transform_point(transform: animation_catalog::Affine, point: Vec3) -> Vec3 {
    Vec3::new(
        Vec3::from_array(transform.rows[0][..3].try_into().unwrap()).dot(point)
            + transform.rows[0][3],
        Vec3::from_array(transform.rows[1][..3].try_into().unwrap()).dot(point)
            + transform.rows[1][3],
        Vec3::from_array(transform.rows[2][..3].try_into().unwrap()).dot(point)
            + transform.rows[2][3],
    )
}

fn transform_vector(transform: animation_catalog::Affine, vector: Vec3) -> Vec3 {
    Vec3::new(
        Vec3::from_array(transform.rows[0][..3].try_into().unwrap()).dot(vector),
        Vec3::from_array(transform.rows[1][..3].try_into().unwrap()).dot(vector),
        Vec3::from_array(transform.rows[2][..3].try_into().unwrap()).dot(vector),
    )
}

fn wrap_uv(value: f32) -> f32 {
    let wrapped = value.fract();
    if wrapped > 0.99999 { 0.0 } else { wrapped }
}

fn pack_rgba8(color: [f32; 4]) -> u32 {
    color
        .into_iter()
        .enumerate()
        .fold(0, |packed, (channel, value)| {
            packed | ((value.clamp(0.0, 1.0) * 255.0).round() as u32) << (channel * 8)
        })
}

fn channel_delta(expected: u32, actual: u32) -> u32 {
    (0..4)
        .map(|channel| {
            let shift = channel * 8;
            ((expected >> shift) & 255).abs_diff((actual >> shift) & 255)
        })
        .max()
        .unwrap()
}
