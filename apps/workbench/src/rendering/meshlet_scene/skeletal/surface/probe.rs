use animation_catalog::Catalog as AnimationCatalog;
use anyhow::{Result, ensure};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use windows::Win32::Graphics::Direct3D12::ID3D12Resource;

use crate::load::LoadConfig;
use crate::rendering::meshlet_scene::skeletal::SkeletalProbe;
use crate::rendering::resident::read_values;

use super::super::renderer::SkeletalSettings;
use super::oracle::{self, OracleInput};
use super::renderer::{SURFACE_REVISION, SurfaceSettings};
use super::resources::{CANDIDATE_CAPACITY, SAMPLE_COUNT, SurfaceResources};

const SAMPLE_PIXELS: [[u32; 2]; 6] = [
    [640, 360],
    [600, 600],
    [320, 500],
    [960, 500],
    [480, 420],
    [800, 420],
];
const SHADE_TOLERANCE: u32 = 2;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceStats {
    pub resolved_pixels: u32,
    pub visible_pixels: u32,
    pub background_pixels: u32,
    pub observed_material_mask: [u32; 2],
    pub observed_material_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceSample {
    pub pixel: [u32; 2],
    pub payload: [u32; 2],
    pub candidate_index: Option<u32>,
    pub primitive_index: Option<u32>,
    pub barycentrics: Option<[f32; 3]>,
    pub visible_index: Option<u32>,
    pub stable_key: Option<u32>,
    pub material_index: Option<u32>,
    pub mip_level: u32,
    pub rgba8: u32,
    pub expected_rgba8: u32,
    pub texel: Option<[u32; 2]>,
    pub expected_texel: Option<[u32; 2]>,
    pub maximum_channel_delta: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceProbe {
    pub revision: &'static str,
    pub settings: Value,
    pub skeletal: SkeletalProbe,
    pub surface_catalog_sha256: String,
    pub visibility_sha256: String,
    pub visibility_width: u32,
    pub visibility_height: u32,
    pub visibility_row_pitch: u32,
    pub visibility_readback_bytes: u64,
    pub visibility_cpu_copy_ms: f64,
    pub stats: SurfaceStats,
    pub samples: Vec<SurfaceSample>,
    pub maximum_sample_channel_delta: u32,
    pub sample_channel_tolerance: u32,
    pub invalid_payload_count: u32,
    pub visibility_dispatch_count: u32,
    pub resolve_dispatch_count: u32,
    pub resolve_groups: [u32; 3],
    pub gpu_visibility_ms: f64,
    pub gpu_resolve_ms: f64,
    pub gpu_total_ms: f64,
}

pub struct ProbeInput<'a> {
    pub resources: &'a SurfaceResources,
    pub settings: SurfaceSettings,
    pub settings_json: Value,
    pub skeletal: SkeletalProbe,
    pub animation_catalog: &'a AnimationCatalog,
    pub skeletal_settings: SkeletalSettings,
    pub config: LoadConfig,
    pub background_color: [f32; 4],
    pub timestamp_readback: &'a ID3D12Resource,
    pub timestamp_frequency: u64,
    pub width: u32,
    pub height: u32,
}

pub unsafe fn read(input: ProbeInput<'_>) -> Result<SurfaceProbe> {
    let visibility = unsafe { input.resources.visibility_readback.read() }?;
    ensure!(
        visibility.width == input.width && visibility.height == input.height,
        "surface visibility dimensions differ from the fixed resolve dimensions"
    );
    let stats_words = unsafe { read_values::<u32>(&input.resources.stats_readback, 8) }?;
    let sample_words =
        unsafe { read_values::<u32>(&input.resources.sample_readback, SAMPLE_COUNT as usize * 8) }?;
    let timestamps = unsafe { read_values::<u64>(input.timestamp_readback, 6) }?;
    let (visible_pixels, invalid_payload_count) = validate_visibility(
        &visibility.bytes,
        input.resources.catalog.primitives.len() as u32,
    );
    let total_pixels = input.width * input.height;
    ensure!(
        stats_words[0] == total_pixels,
        "surface resolve pixel count differs from the fixed screen extent"
    );
    ensure!(
        stats_words[1] == visible_pixels,
        "surface visible-pixel counter differs from the visibility attachment"
    );
    ensure!(
        stats_words[1] + stats_words[2] == total_pixels,
        "surface visible and background counters do not cover the screen"
    );
    ensure!(
        invalid_payload_count == 0,
        "surface visibility attachment contains invalid payloads"
    );
    validate_material_mask(
        stats_words[3],
        stats_words[4],
        input.settings.material_count,
    )?;
    let mut samples = decode_samples(
        &sample_words,
        &visibility.bytes,
        input.width,
        input.settings,
        input.resources.catalog.primitives.len() as u32,
    )?;
    let mut maximum_sample_channel_delta = 0;
    for sample in &mut samples {
        let result = oracle::shade(
            sample,
            OracleInput {
                surface: &input.resources.catalog,
                animation: input.animation_catalog,
                skeletal_settings: input.skeletal_settings,
                surface_settings: input.settings,
                config: input.config,
                background_color: input.background_color,
            },
        )?;
        sample.expected_rgba8 = result.expected_rgba8;
        sample.expected_texel = result.expected_texel;
        sample.maximum_channel_delta = result.maximum_channel_delta;
        ensure!(
            sample.texel == sample.expected_texel,
            "surface sample {:?} GPU texel {:?} differs from CPU texel {:?}",
            sample.pixel,
            sample.texel,
            sample.expected_texel
        );
        maximum_sample_channel_delta =
            maximum_sample_channel_delta.max(result.maximum_channel_delta);
    }
    if maximum_sample_channel_delta > SHADE_TOLERANCE {
        let worst = samples
            .iter()
            .max_by_key(|sample| sample.maximum_channel_delta)
            .unwrap();
        anyhow::bail!(
            "surface sample {:?} GPU {:#010x} CPU {:#010x} channel delta {} exceeds {}",
            worst.pixel,
            worst.rgba8,
            worst.expected_rgba8,
            worst.maximum_channel_delta,
            SHADE_TOLERANCE
        );
    }
    let milliseconds = |start: usize, end: usize| {
        timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
            / input.timestamp_frequency as f64
    };
    Ok(SurfaceProbe {
        revision: SURFACE_REVISION,
        settings: input.settings_json,
        skeletal: input.skeletal,
        surface_catalog_sha256: input.resources.catalog_sha256.clone(),
        visibility_sha256: format!("{:x}", Sha256::digest(&visibility.bytes)),
        visibility_width: visibility.width,
        visibility_height: visibility.height,
        visibility_row_pitch: visibility.row_pitch,
        visibility_readback_bytes: visibility.allocation_bytes,
        visibility_cpu_copy_ms: visibility.row_copy_ms,
        stats: SurfaceStats {
            resolved_pixels: stats_words[0],
            visible_pixels: stats_words[1],
            background_pixels: stats_words[2],
            observed_material_mask: [stats_words[3], stats_words[4]],
            observed_material_count: stats_words[3].count_ones() + stats_words[4].count_ones(),
        },
        samples,
        maximum_sample_channel_delta,
        sample_channel_tolerance: SHADE_TOLERANCE,
        invalid_payload_count,
        visibility_dispatch_count: 1,
        resolve_dispatch_count: 1,
        resolve_groups: [input.width.div_ceil(8), input.height.div_ceil(8), 1],
        gpu_visibility_ms: milliseconds(3, 4),
        gpu_resolve_ms: milliseconds(4, 5),
        gpu_total_ms: milliseconds(0, 5),
    })
}

fn validate_visibility(bytes: &[u8], primitive_count: u32) -> (u32, u32) {
    let mut visible = 0;
    let mut invalid = 0;
    for pixel in bytes.chunks_exact(8) {
        let word0 = u32::from_le_bytes(pixel[..4].try_into().unwrap());
        let word1 = u32::from_le_bytes(pixel[4..].try_into().unwrap());
        if word0 == 0 {
            invalid += u32::from(word1 != 0);
            continue;
        }
        visible += 1;
        let candidate = word0 & 0x7fff;
        let primitive = (word0 >> 15) & 0xffff;
        invalid += u32::from(
            word0 & 0x8000_0000 != 0
                || candidate == 0
                || candidate > CANDIDATE_CAPACITY
                || primitive >= primitive_count,
        );
    }
    (visible, invalid)
}

fn validate_material_mask(low: u32, high: u32, material_count: u32) -> Result<()> {
    let valid_low = if material_count >= 32 {
        u32::MAX
    } else {
        (1u32 << material_count) - 1
    };
    let valid_high = if material_count <= 32 {
        0
    } else if material_count == 64 {
        u32::MAX
    } else {
        (1u32 << (material_count - 32)) - 1
    };
    ensure!(
        low & !valid_low == 0 && high & !valid_high == 0,
        "surface resolve observed a material outside the configured range"
    );
    Ok(())
}

fn decode_samples(
    words: &[u32],
    visibility: &[u8],
    width: u32,
    settings: SurfaceSettings,
    primitive_count: u32,
) -> Result<Vec<SurfaceSample>> {
    SAMPLE_PIXELS
        .into_iter()
        .enumerate()
        .map(|(index, pixel)| {
            let words = &words[index * 8..index * 8 + 8];
            let byte_offset = ((pixel[1] * width + pixel[0]) * 8) as usize;
            let attachment = [
                u32::from_le_bytes(visibility[byte_offset..byte_offset + 4].try_into().unwrap()),
                u32::from_le_bytes(
                    visibility[byte_offset + 4..byte_offset + 8]
                        .try_into()
                        .unwrap(),
                ),
            ];
            ensure!(
                [words[0], words[1]] == attachment,
                "surface sample payload differs from the visibility attachment"
            );
            let visible = words[0] != 0;
            let candidate = (words[0] & 0x7fff).wrapping_sub(1);
            let primitive = (words[0] >> 15) & 0xffff;
            ensure!(
                !visible || (candidate < CANDIDATE_CAPACITY && primitive < primitive_count),
                "surface sample references an invalid candidate or primitive"
            );
            ensure!(
                words[5] == settings.mip_level,
                "surface sample mip differs from the configured mip"
            );
            if visible {
                ensure!(
                    words[2] != u32::MAX
                        && words[3] != u32::MAX
                        && words[4] < settings.material_count,
                    "visible surface sample metadata is invalid"
                );
            } else {
                ensure!(
                    words[2..5].iter().all(|word| *word == u32::MAX),
                    "background surface sample contains visible metadata"
                );
            }
            let barycentrics = visible.then(|| {
                let x = (words[1] & 0xffff) as f32 / 65_535.0;
                let y = (words[1] >> 16) as f32 / 65_535.0;
                let z = (1.0 - x - y).max(0.0);
                let sum = (x + y + z).max(0.00001);
                [x / sum, y / sum, z / sum]
            });
            Ok(SurfaceSample {
                pixel,
                payload: attachment,
                candidate_index: visible.then_some(candidate),
                primitive_index: visible.then_some(primitive),
                barycentrics,
                visible_index: visible.then_some(words[2]),
                stable_key: visible.then_some(words[3]),
                material_index: visible.then_some(words[4]),
                mip_level: words[5],
                rgba8: words[6],
                expected_rgba8: 0,
                texel: visible.then_some([words[7] & 0xffff, words[7] >> 16]),
                expected_texel: None,
                maximum_channel_delta: 0,
            })
        })
        .collect()
}
