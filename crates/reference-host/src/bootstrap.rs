use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use engine_runtime::{FrameRequest, GlobalRegionConfig, RegionCoord, Runtime};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{HostInput, window};

pub const REVISION: &str = "declarative-runtime-bootstrap-v2";
pub const TIMEOUT: Duration = Duration::from_secs(120);
const MAX_CONFIG_BYTES: usize = 64 * 1024;

pub struct Arguments {
    pub launched_by_sidecar: bool,
    pub bootstrap: Option<Plan>,
}

impl Arguments {
    pub fn parse() -> Result<Self> {
        let parsed = parse_arguments(std::env::args_os().skip(1))?;
        let bootstrap = parsed.bootstrap.map(Plan::load).transpose()?;
        Ok(Self {
            launched_by_sidecar: parsed.launched_by_sidecar,
            bootstrap,
        })
    }
}

#[derive(Debug)]
struct ParsedArguments {
    launched_by_sidecar: bool,
    bootstrap: Option<PathBuf>,
}

fn parse_arguments(arguments: impl IntoIterator<Item = OsString>) -> Result<ParsedArguments> {
    let mut launched_by_sidecar = false;
    let mut bootstrap = None;
    for argument in arguments {
        let argument = argument
            .into_string()
            .map_err(|_| anyhow::anyhow!("host arguments must be valid Unicode"))?;
        if argument.starts_with("--sidecar-stamp=") {
            ensure!(!launched_by_sidecar, "duplicate --sidecar-stamp argument");
            launched_by_sidecar = true;
        } else if let Some(value) = argument.strip_prefix("--bootstrap=") {
            ensure!(bootstrap.is_none(), "duplicate --bootstrap argument");
            ensure!(!value.is_empty(), "--bootstrap requires a path");
            bootstrap = Some(PathBuf::from(value));
        } else {
            bail!("unsupported host argument {argument:?}");
        }
    }
    Ok(ParsedArguments {
        launched_by_sidecar,
        bootstrap,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Document {
    schema_version: u32,
    terrain: String,
    objects: String,
    global_origin: Coordinate,
    global_center: Coordinate,
    active_radius: u32,
    playable_region_bounds: RegionBoundsDocument,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Coordinate {
    x: i64,
    z: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionBoundsDocument {
    minimum: Coordinate,
    maximum: Coordinate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayableRegionBounds {
    minimum: RegionCoord,
    maximum: RegionCoord,
}

impl PlayableRegionBounds {
    pub fn new(minimum: RegionCoord, maximum: RegionCoord) -> Result<Self> {
        ensure!(
            minimum.x <= maximum.x,
            "playable region minimum X must not exceed maximum X"
        );
        ensure!(
            minimum.z <= maximum.z,
            "playable region minimum Z must not exceed maximum Z"
        );
        Ok(Self { minimum, maximum })
    }

    pub const fn minimum(self) -> RegionCoord {
        self.minimum
    }

    pub const fn maximum(self) -> RegionCoord {
        self.maximum
    }

    pub const fn contains(self, region: RegionCoord) -> bool {
        region.x >= self.minimum.x
            && region.x <= self.maximum.x
            && region.z >= self.minimum.z
            && region.z <= self.maximum.z
    }
}

pub struct Plan {
    config_path: PathBuf,
    config_bytes: usize,
    config_sha256: String,
    terrain_path: PathBuf,
    object_path: PathBuf,
    global_config: GlobalRegionConfig,
    playable_region_bounds: PlayableRegionBounds,
}

impl Plan {
    fn load(path: PathBuf) -> Result<Self> {
        validate_config_path(&path)?;
        let metadata = fs::metadata(&path)
            .with_context(|| format!("reading bootstrap metadata {}", path.display()))?;
        ensure!(metadata.is_file(), "bootstrap path must identify a file");
        let length = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
        ensure!(
            (1..=MAX_CONFIG_BYTES).contains(&length),
            "bootstrap JSON must contain 1..={MAX_CONFIG_BYTES} bytes"
        );
        let bytes = fs::read(&path)
            .with_context(|| format!("reading bootstrap configuration {}", path.display()))?;
        Self::decode(path, &bytes)
    }

    fn decode(config_path: PathBuf, bytes: &[u8]) -> Result<Self> {
        ensure!(
            (1..=MAX_CONFIG_BYTES).contains(&bytes.len()),
            "bootstrap JSON must contain 1..={MAX_CONFIG_BYTES} bytes"
        );
        let document: Document = serde_json::from_slice(bytes).context("invalid bootstrap JSON")?;
        ensure!(
            document.schema_version == 2,
            "bootstrap schemaVersion must equal 2"
        );
        let terrain_path = validate_pack_path(&document.terrain, PackKind::Terrain)?;
        let object_path = validate_pack_path(&document.objects, PackKind::Objects)?;
        let global_config = GlobalRegionConfig::new(
            document.global_origin.x,
            document.global_origin.z,
            document.global_center.x,
            document.global_center.z,
            document.active_radius,
        )?;
        let playable_region_bounds = PlayableRegionBounds::new(
            RegionCoord::new(
                document.playable_region_bounds.minimum.x,
                document.playable_region_bounds.minimum.z,
            ),
            RegionCoord::new(
                document.playable_region_bounds.maximum.x,
                document.playable_region_bounds.maximum.z,
            ),
        )?;
        ensure!(
            playable_region_bounds.contains(global_config.global_center),
            "playable region bounds must contain globalCenter"
        );
        let config_sha256 = format!("{:x}", Sha256::digest(bytes));
        Ok(Self {
            config_path,
            config_bytes: bytes.len(),
            config_sha256,
            terrain_path,
            object_path,
            global_config,
            playable_region_bounds,
        })
    }

    pub fn terrain_path(&self) -> &Path {
        &self.terrain_path
    }

    pub fn object_path(&self) -> &Path {
        &self.object_path
    }

    pub fn global_config(&self) -> GlobalRegionConfig {
        self.global_config
    }

    pub fn playable_region_bounds(&self) -> PlayableRegionBounds {
        self.playable_region_bounds
    }

    pub fn pending_json(&self) -> Value {
        json!({
            "revision": REVISION,
            "mode": "canonical-bootstrap-pending",
            "configPath": self.config_path,
            "configBytes": self.config_bytes,
            "configSha256": self.config_sha256,
            "terrainPath": self.terrain_path,
            "objectPath": self.object_path,
            "globalConfig": self.global_config,
            "playableRegionBounds": self.playable_region_bounds,
        })
    }

    fn ready_json(&self, schedule: Value, ready_frame_index: u64, elapsed: Duration) -> Value {
        let mut value = self.pending_json();
        value["mode"] = json!("canonical-bootstrap");
        value["schedule"] = schedule;
        value["readyFrameIndex"] = json!(ready_frame_index);
        value["elapsedMs"] = json!(elapsed.as_secs_f64() * 1_000.0);
        value
    }
}

pub fn idle_json() -> Value {
    json!({
        "revision": REVISION,
        "mode": "idle-shell",
    })
}

#[derive(Clone, Copy)]
pub enum PackKind {
    Terrain,
    Objects,
}

pub fn validate_pack_path(value: &str, kind: PackKind) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(!path.is_absolute(), "pack path must be repository-relative");
    ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "pack path contains an invalid component"
    );
    let components = path.components().collect::<Vec<_>>();
    ensure!(
        components.len() >= 3
            && components[0].as_os_str() == "out"
            && components[1].as_os_str() == "cooked",
        "pack path must be under out/cooked"
    );
    let expected = match kind {
        PackKind::Terrain => "wlt",
        PackKind::Objects => "wlr",
    };
    ensure!(
        path.extension()
            .is_some_and(|extension| extension == expected),
        "pack must use the .{expected} extension"
    );
    Ok(path.to_path_buf())
}

pub struct Ready {
    pub status: Value,
    pub frame_count: u64,
    pub last_frame_duration: Duration,
}

/// Opens the exact configured sources and advances hidden canonical frames until publication.
///
/// # Safety
///
/// `runtime` must own a live window on the calling thread and retain exclusive access to its
/// native GPU objects for the duration of this call.
pub unsafe fn drive(
    runtime: &mut Runtime,
    input: &mut HostInput,
    plan: &Plan,
    clear_color: [f32; 4],
) -> Result<Ready> {
    runtime
        .open_terrain_pack(plan.terrain_path().to_path_buf())
        .context("bootstrap terrain source failed")?;
    runtime
        .open_cooked_object_pack(plan.object_path().to_path_buf())
        .context("bootstrap object source failed")?;
    let schedule = unsafe { runtime.schedule_global_composition(plan.global_config()) }
        .context("bootstrap canonical schedule failed")?;
    let started = std::time::Instant::now();
    let mut frame_count = 0_u64;
    loop {
        if !window::pump_messages() {
            bail!("host closed during canonical bootstrap");
        }
        input.ingest(window::drain_input());
        let frame_started = std::time::Instant::now();
        unsafe {
            let _ = runtime.frame(FrameRequest {
                clear_color,
                capture: false,
                capture_object_ids: false,
                probe: false,
                object_target: None,
            })?;
        }
        let last_frame_duration = frame_started.elapsed();
        frame_count += 1;
        if runtime.composition_enabled() {
            return Ok(Ready {
                status: plan.ready_json(schedule, frame_count, started.elapsed()),
                frame_count,
                last_frame_duration,
            });
        }
        let status = runtime.composition_status();
        if !status["lastFailure"].is_null() {
            bail!("canonical bootstrap pair failed: {}", status["lastFailure"]);
        }
        if started.elapsed() >= TIMEOUT {
            bail!(
                "canonical bootstrap did not publish within {} seconds",
                TIMEOUT.as_secs()
            );
        }
    }
}

fn validate_config_path(path: &Path) -> Result<()> {
    ensure!(
        !path.is_absolute(),
        "bootstrap path must be repository-relative"
    );
    ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "bootstrap path contains an invalid component"
    );
    let components = path.components().collect::<Vec<_>>();
    ensure!(
        components.len() >= 3
            && components[0].as_os_str() == "out"
            && components[1].as_os_str() == "cooked",
        "bootstrap path must be under out/cooked"
    );
    ensure!(
        path.extension()
            .is_some_and(|extension| extension == "json"),
        "bootstrap path must use the .json extension"
    );
    Ok(())
}

#[cfg(test)]
#[path = "../tests/private/bootstrap.rs"]
mod tests;
