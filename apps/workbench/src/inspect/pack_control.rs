use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::rendering::Renderer;

use super::protocol::{ControlKind, ControlResult, ProtocolError};

pub enum PackTarget {
    Cooked,
    Object,
    Terrain,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackPayload {
    path: String,
}

pub fn parse(value: Value, target: PackTarget) -> Result<ControlKind, ProtocolError> {
    let payload: PackPayload = serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })?;
    Ok(match target {
        PackTarget::Cooked => ControlKind::CookedOpen { path: payload.path },
        PackTarget::Object => ControlKind::CookedObjectOpen { path: payload.path },
        PackTarget::Terrain => ControlKind::TerrainOpen { path: payload.path },
    })
}

pub fn validate_cooked(value: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(value);
    anyhow::ensure!(
        !path.is_absolute(),
        "cooked pack path must be repository-relative"
    );
    anyhow::ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "cooked pack path contains an invalid component"
    );
    let components = path.components().collect::<Vec<_>>();
    anyhow::ensure!(
        components.len() >= 3
            && components[0].as_os_str() == "out"
            && components[1].as_os_str() == "cooked",
        "cooked pack path must be under out/cooked"
    );
    anyhow::ensure!(
        path.extension().is_some_and(|extension| extension == "wlr"),
        "cooked pack must use the .wlr extension"
    );
    Ok(path.to_path_buf())
}

pub fn dispatch_object(renderer: &mut Renderer, kind: ControlKind) -> ControlResult {
    match kind {
        ControlKind::CookedObjectStatus => {
            Ok(renderer.cooked_object_status().unwrap_or(Value::Null))
        }
        ControlKind::CookedObjectOpen { path } => validate_cooked(&path)
            .and_then(|path| renderer.open_cooked_object_pack(path))
            .map_err(|error| ProtocolError {
                code: "pack_open_failed",
                message: error.to_string(),
            }),
        ControlKind::CookedObjectDisable => renderer
            .disable_cooked_object_source()
            .map(|disabled| json!({"disabled": disabled}))
            .map_err(|error| ProtocolError {
                code: "source_disable_failed",
                message: error.to_string(),
            }),
        ControlKind::CookedObjectIoGateArm => renderer
            .arm_object_io_gate()
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        ControlKind::CookedObjectIoGateRelease => renderer
            .release_object_io_gate()
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        _ => unreachable!("non-object command reached object dispatcher"),
    }
}

fn gate_error(error: anyhow::Error) -> ProtocolError {
    ProtocolError {
        code: "gate_failed",
        message: error.to_string(),
    }
}
