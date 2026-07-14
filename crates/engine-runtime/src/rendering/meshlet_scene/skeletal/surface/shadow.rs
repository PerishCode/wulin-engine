use anyhow::{Result, ensure};
use glam::{Mat4, Vec3};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::rendering::gpu_capture::CapturedPixels;

use super::pipeline::SURFACE_CONSTANT_COUNT;
use super::probe::SurfaceSample;

pub const REVISION: &str = "camera-visible-directional-shadow-v1";
pub const MAP_SIDE: u32 = 1_024;
pub const FORMAT: &str = "D32_FLOAT";
pub const RECEIVER_BIAS: f32 = 0.0015;
pub const LIGHT_DIRECTION: [f32; 3] = [-0.45, 0.8, 0.3];

const LIGHT_CENTER: [f32; 3] = [0.0, 4.0, 0.0];
const LIGHT_DISTANCE: f32 = 128.0;
const ORTHOGRAPHIC_HALF_EXTENT: f32 = 80.0;
const NEAR_PLANE: f32 = 1.0;
const FAR_PLANE: f32 = 255.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Address {
    pub texel: [u32; 2],
    pub receiver_depth: f32,
}

pub fn light_view_projection() -> Mat4 {
    let direction = Vec3::from_array(LIGHT_DIRECTION).normalize();
    let center = Vec3::from_array(LIGHT_CENTER);
    let eye = center + direction * LIGHT_DISTANCE;
    Mat4::orthographic_rh(
        -ORTHOGRAPHIC_HALF_EXTENT,
        ORTHOGRAPHIC_HALF_EXTENT,
        -ORTHOGRAPHIC_HALF_EXTENT,
        ORTHOGRAPHIC_HALF_EXTENT,
        NEAR_PLANE,
        FAR_PLANE,
    ) * Mat4::look_at_rh(eye, center, Vec3::Y)
}

pub fn address(world: Vec3) -> Option<Address> {
    let ndc = light_view_projection().project_point3(world);
    if !ndc.is_finite()
        || !(-1.0..=1.0).contains(&ndc.x)
        || !(-1.0..=1.0).contains(&ndc.y)
        || !(0.0..=1.0).contains(&ndc.z)
    {
        return None;
    }
    let texel = [
        (((ndc.x * 0.5 + 0.5) * MAP_SIDE as f32) as u32).min(MAP_SIDE - 1),
        (((-ndc.y * 0.5 + 0.5) * MAP_SIDE as f32) as u32).min(MAP_SIDE - 1),
    ];
    Some(Address {
        texel,
        receiver_depth: ndc.z,
    })
}

pub fn stored_depth(bytes: &[u8], texel: [u32; 2]) -> f32 {
    let offset = ((texel[1] * MAP_SIDE + texel[0]) * 4) as usize;
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

pub fn is_shadowed(receiver_depth: f32, stored_depth: f32) -> bool {
    receiver_depth > stored_depth + RECEIVER_BIAS
}

pub fn validate_projection() -> Result<()> {
    for x in [-40.0, 40.0] {
        for y in [-8.0, 24.0] {
            for z in [-40.0, 40.0] {
                ensure!(
                    address(Vec3::new(x, y, z)).is_some(),
                    "canonical render window exceeds the fixed shadow projection"
                );
            }
        }
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShadowProbe {
    pub revision: &'static str,
    pub enabled: bool,
    pub direction: [f32; 3],
    pub light_view_projection_sha256: String,
    pub map_side: u32,
    pub format: &'static str,
    pub map_bytes: u64,
    pub readback_allocation_bytes: u64,
    pub readback_row_pitch: u32,
    pub receiver_bias: f32,
    pub depth_sha256: String,
    pub occupied_texels: u32,
    pub clear_texels: u32,
    pub minimum_occupied_depth: f32,
    pub maximum_occupied_depth: f32,
    pub caster_count: u32,
    pub indirect_dispatch_count: u32,
    pub root_constant_dwords: u32,
    pub descriptor_count: u32,
    pub sample_shadowed_count: u32,
    pub sample_lit_count: u32,
    pub sample_mismatch_count: u32,
    pub gpu_ms: f64,
}

pub fn build_probe(
    depth: &CapturedPixels,
    samples: &[SurfaceSample],
    caster_count: u32,
    gpu_ms: f64,
) -> Result<ShadowProbe> {
    let mut occupied_texels = 0u32;
    let mut minimum_occupied_depth = 1.0f32;
    let mut maximum_occupied_depth = 0.0f32;
    for bytes in depth.bytes.chunks_exact(4) {
        let value = f32::from_le_bytes(bytes.try_into().unwrap());
        ensure!(
            value.is_finite() && (0.0..=1.0).contains(&value),
            "surface shadow depth contains an invalid value"
        );
        if value.to_bits() != 1.0f32.to_bits() {
            occupied_texels += 1;
            minimum_occupied_depth = minimum_occupied_depth.min(value);
            maximum_occupied_depth = maximum_occupied_depth.max(value);
        }
    }
    let total_texels = MAP_SIDE * MAP_SIDE;
    ensure!(
        occupied_texels > 0 && occupied_texels < total_texels,
        "surface shadow depth must contain both occupied and clear texels"
    );
    let sample_shadowed_count = samples
        .iter()
        .filter(|sample| sample.shadowed == Some(true))
        .count() as u32;
    let sample_lit_count = samples
        .iter()
        .filter(|sample| sample.shadowed == Some(false))
        .count() as u32;
    let sample_mismatch_count = samples
        .iter()
        .filter(|sample| {
            sample.shadowed != sample.expected_shadowed
                || sample.shadow_texel != sample.expected_shadow_texel
        })
        .count() as u32;
    let mut matrix_digest = Sha256::new();
    for value in light_view_projection().to_cols_array() {
        matrix_digest.update(value.to_bits().to_le_bytes());
    }
    Ok(ShadowProbe {
        revision: REVISION,
        enabled: true,
        direction: LIGHT_DIRECTION,
        light_view_projection_sha256: format!("{:x}", matrix_digest.finalize()),
        map_side: MAP_SIDE,
        format: FORMAT,
        map_bytes: u64::from(total_texels) * 4,
        readback_allocation_bytes: depth.allocation_bytes,
        readback_row_pitch: depth.row_pitch,
        receiver_bias: RECEIVER_BIAS,
        depth_sha256: format!("{:x}", Sha256::digest(&depth.bytes)),
        occupied_texels,
        clear_texels: total_texels - occupied_texels,
        minimum_occupied_depth,
        maximum_occupied_depth,
        caster_count,
        indirect_dispatch_count: 1,
        root_constant_dwords: SURFACE_CONSTANT_COUNT,
        descriptor_count: 98,
        sample_shadowed_count,
        sample_lit_count,
        sample_mismatch_count,
        gpu_ms,
    })
}
