use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use engine_runtime::GlobalRegionConfig;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::inspect::{PackKind, validate_pack_path};

pub(crate) const REVISION: &str = "declarative-runtime-bootstrap-v1";
pub(crate) const TIMEOUT: Duration = Duration::from_secs(120);
const MAX_CONFIG_BYTES: usize = 64 * 1024;

pub(crate) struct Arguments {
    pub(crate) launched_by_sidecar: bool,
    pub(crate) bootstrap: Option<Plan>,
}

impl Arguments {
    pub(crate) fn parse() -> Result<Self> {
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
            .map_err(|_| anyhow::anyhow!("workbench arguments must be valid Unicode"))?;
        if argument.starts_with("--sidecar-stamp=") {
            ensure!(!launched_by_sidecar, "duplicate --sidecar-stamp argument");
            launched_by_sidecar = true;
        } else if let Some(value) = argument.strip_prefix("--bootstrap=") {
            ensure!(bootstrap.is_none(), "duplicate --bootstrap argument");
            ensure!(!value.is_empty(), "--bootstrap requires a path");
            bootstrap = Some(PathBuf::from(value));
        } else {
            bail!("unsupported workbench argument {argument:?}");
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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Coordinate {
    x: i64,
    z: i64,
}

pub(crate) struct Plan {
    config_path: PathBuf,
    config_bytes: usize,
    config_sha256: String,
    terrain_path: PathBuf,
    object_path: PathBuf,
    global_config: GlobalRegionConfig,
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
            document.schema_version == 1,
            "bootstrap schemaVersion must equal 1"
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
        let config_sha256 = format!("{:x}", Sha256::digest(bytes));
        Ok(Self {
            config_path,
            config_bytes: bytes.len(),
            config_sha256,
            terrain_path,
            object_path,
            global_config,
        })
    }

    pub(crate) fn terrain_path(&self) -> &Path {
        &self.terrain_path
    }

    pub(crate) fn object_path(&self) -> &Path {
        &self.object_path
    }

    pub(crate) fn global_config(&self) -> GlobalRegionConfig {
        self.global_config
    }

    pub(crate) fn pending_json(&self) -> Value {
        json!({
            "revision": REVISION,
            "mode": "canonical-bootstrap-pending",
            "configPath": self.config_path,
            "configBytes": self.config_bytes,
            "configSha256": self.config_sha256,
            "terrainPath": self.terrain_path,
            "objectPath": self.object_path,
            "globalConfig": self.global_config,
        })
    }

    pub(crate) fn ready_json(
        &self,
        schedule: Value,
        ready_frame_index: u64,
        elapsed: Duration,
    ) -> Value {
        let mut value = self.pending_json();
        value["mode"] = json!("canonical-bootstrap");
        value["schedule"] = schedule;
        value["readyFrameIndex"] = json!(ready_frame_index);
        value["elapsedMs"] = json!(elapsed.as_secs_f64() * 1_000.0);
        value
    }
}

pub(crate) fn idle_json() -> Value {
    json!({
        "revision": REVISION,
        "mode": "idle-shell",
    })
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
