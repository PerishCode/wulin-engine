use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::gpu_capture::CapturedPixels;

const OUTPUT_ROOT: &str = "out/captures";

pub struct FrameContext<'a> {
    pub capture_id: &'a str,
    pub collection: &'a str,
    pub frame_index: u64,
    pub clear_color: [f32; 4],
    pub paused: bool,
    pub launched_by_sidecar: bool,
    pub adapter: &'a str,
    pub debug_layer: bool,
    pub device_removed_reason: Option<String>,
    pub last_error: Option<&'a str>,
    pub gpu_readback_ms: f64,
    pub spatial: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameManifest<'a> {
    schema_version: u32,
    capture_id: &'a str,
    collection: &'a str,
    revision: String,
    process_id: u32,
    launched_by_sidecar: bool,
    frame_index: u64,
    state: &'static str,
    clear_color: [f32; 4],
    spatial: Value,
    renderer: RendererManifest<'a>,
    image: ImageManifest,
    artifacts: ArtifactManifest,
    timing: TimingManifest,
    last_error: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RendererManifest<'a> {
    api: &'static str,
    adapter: &'a str,
    feature_level: &'static str,
    format: &'static str,
    debug_layer: bool,
    device_removed_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageManifest {
    width: u32,
    height: u32,
    raw_byte_count: usize,
    row_pitch: u32,
    readback_allocation_bytes: u64,
    pixel_sha256: String,
    png_byte_count: usize,
    png_sha256: String,
    reference_pixel_rgba: [u8; 4],
    different_pixel_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactManifest {
    png: String,
    manifest: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TimingManifest {
    gpu_submission_and_readback_ms: f64,
    row_copy_ms: f64,
    hash_ms: f64,
    encode_ms: f64,
    png_write_ms: f64,
    pre_manifest_write_ms: f64,
}

pub fn write(pixels: CapturedPixels, context: FrameContext<'_>) -> Result<Value> {
    let total_start = Instant::now();
    let output_root = PathBuf::from(OUTPUT_ROOT).join(context.collection);
    fs::create_dir_all(&output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;
    let png_path = output_root.join(format!("{}.png", context.capture_id));
    let manifest_path = output_root.join(format!("{}.json", context.capture_id));

    let hash_start = Instant::now();
    let pixel_sha256 = sha256(&pixels.rgba);
    let reference_pixel_rgba = pixels
        .rgba
        .get(..4)
        .and_then(|value| value.try_into().ok())
        .context("captured frame does not contain a complete reference pixel")?;
    let different_pixel_count = pixels
        .rgba
        .chunks_exact(4)
        .filter(|pixel| *pixel != reference_pixel_rgba)
        .count();
    let hash_ms = elapsed_ms(hash_start);

    let encode_start = Instant::now();
    let png = encode_png(pixels.width, pixels.height, &pixels.rgba)?;
    let png_sha256 = sha256(&png);
    let encode_ms = elapsed_ms(encode_start);

    let write_start = Instant::now();
    fs::write(&png_path, &png)
        .with_context(|| format!("failed to write {}", png_path.display()))?;
    let png_write_ms = elapsed_ms(write_start);

    let manifest = FrameManifest {
        schema_version: 1,
        capture_id: context.capture_id,
        collection: context.collection,
        revision: git_revision(),
        process_id: std::process::id(),
        launched_by_sidecar: context.launched_by_sidecar,
        frame_index: context.frame_index,
        state: if context.paused { "paused" } else { "running" },
        clear_color: context.clear_color,
        spatial: context.spatial,
        renderer: RendererManifest {
            api: "D3D12",
            adapter: context.adapter,
            feature_level: "12_1",
            format: "R8G8B8A8_UNORM",
            debug_layer: context.debug_layer,
            device_removed_reason: context.device_removed_reason,
        },
        image: ImageManifest {
            width: pixels.width,
            height: pixels.height,
            raw_byte_count: pixels.rgba.len(),
            row_pitch: pixels.row_pitch,
            readback_allocation_bytes: pixels.allocation_bytes,
            pixel_sha256,
            png_byte_count: png.len(),
            png_sha256,
            reference_pixel_rgba,
            different_pixel_count,
        },
        artifacts: ArtifactManifest {
            png: path_text(&png_path),
            manifest: path_text(&manifest_path),
        },
        timing: TimingManifest {
            gpu_submission_and_readback_ms: context.gpu_readback_ms,
            row_copy_ms: pixels.row_copy_ms,
            hash_ms,
            encode_ms,
            png_write_ms,
            pre_manifest_write_ms: elapsed_ms(total_start),
        },
        last_error: context.last_error,
    };
    let json = serde_json::to_vec_pretty(&manifest).context("failed to encode frame manifest")?;
    fs::write(&manifest_path, [&json[..], b"\n"].concat())
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    serde_json::to_value(manifest).context("failed to encode capture response")
}

fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .context("failed to write PNG header")?;
        writer
            .write_image_data(rgba)
            .context("failed to encode PNG pixels")?;
        writer.finish().context("failed to finish PNG")?;
    }
    Ok(output)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1_000.0
}

fn git_revision() -> String {
    Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
