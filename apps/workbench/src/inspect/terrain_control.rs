use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::load::LoadConfig;
use crate::rendering::Renderer;
use crate::terrain::GlobalTerrainConfig;

use super::protocol::{ControlKind, ControlResult, ProtocolError};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GlobalPayload {
    origin_x: i64,
    origin_z: i64,
    center_x: i64,
    center_z: i64,
    active_radius: u32,
}

pub fn parse_global(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload: GlobalPayload = serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })?;
    Ok(ControlKind::TerrainGlobalSchedule {
        origin_x: payload.origin_x,
        origin_z: payload.origin_z,
        center_x: payload.center_x,
        center_z: payload.center_z,
        active_radius: payload.active_radius,
    })
}

pub fn dispatch(renderer: &mut Renderer, kind: ControlKind) -> ControlResult {
    match kind {
        ControlKind::TerrainStatus => Ok(renderer.terrain_status()),
        ControlKind::TerrainOpen { path } => validate_path(&path)
            .and_then(|path| renderer.open_terrain_pack(path))
            .and_then(|metadata| serde_json::to_value(metadata).map_err(Into::into))
            .map_err(|error| ProtocolError {
                code: "pack_open_failed",
                message: error.to_string(),
            }),
        ControlKind::TerrainEnable => renderer
            .enable_terrain()
            .map(|()| renderer.terrain_status())
            .map_err(|error| ProtocolError {
                code: "terrain_unavailable",
                message: error.to_string(),
            }),
        ControlKind::TerrainDisable => {
            renderer.disable_terrain();
            Ok(renderer.terrain_status())
        }
        ControlKind::TerrainIoGateArm => renderer
            .arm_terrain_io_gate()
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        ControlKind::TerrainIoGateRelease => renderer
            .release_terrain_io_gate()
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        ControlKind::TerrainCopyGateArm => renderer
            .arm_terrain_copy_gate()
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        ControlKind::TerrainCopyGateRelease => unsafe { renderer.release_terrain_copy_gate() }
            .map(|fence| json!({"gateFence": fence}))
            .map_err(gate_error),
        ControlKind::TerrainLodStatus => Ok(renderer.terrain_lod_status()),
        ControlKind::TerrainLodConfigure {
            near_patch_radius,
            middle_patch_radius,
            forced_lod,
        } => renderer
            .configure_terrain_lod(near_patch_radius, middle_patch_radius, forced_lod)
            .map(|()| renderer.terrain_lod_status())
            .map_err(|error| ProtocolError {
                code: "invalid_terrain_lod_config",
                message: error.to_string(),
            }),
        ControlKind::TerrainLodEnable => {
            renderer.enable_terrain_lod();
            Ok(renderer.terrain_lod_status())
        }
        ControlKind::TerrainLodDisable => {
            renderer.disable_terrain_lod();
            Ok(renderer.terrain_lod_status())
        }
        ControlKind::TerrainSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        } => schedule(
            renderer,
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        ),
        ControlKind::TerrainGlobalSchedule {
            origin_x,
            origin_z,
            center_x,
            center_z,
            active_radius,
        } => schedule_global(
            renderer,
            origin_x,
            origin_z,
            center_x,
            center_z,
            active_radius,
        ),
        _ => unreachable!("non-terrain command reached terrain dispatcher"),
    }
}

fn schedule_global(
    renderer: &mut Renderer,
    origin_x: i64,
    origin_z: i64,
    center_x: i64,
    center_z: i64,
    active_radius: u32,
) -> ControlResult {
    let config = GlobalTerrainConfig::new(origin_x, origin_z, center_x, center_z, active_radius)
        .map_err(|error| ProtocolError {
            code: "invalid_global_terrain_config",
            message: error.to_string(),
        })?;
    let report = renderer
        .schedule_global_terrain(config)
        .map_err(stream_error)?;
    serde_json::to_value(report).map_err(|error| ProtocolError {
        code: "stream_failed",
        message: error.to_string(),
    })
}

fn schedule(
    renderer: &mut Renderer,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> ControlResult {
    let config = LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    )
    .map_err(|error| ProtocolError {
        code: "invalid_load_config",
        message: error.to_string(),
    })?;
    let report = renderer.schedule_terrain(config).map_err(stream_error)?;
    serde_json::to_value(report).map_err(|error| ProtocolError {
        code: "stream_failed",
        message: error.to_string(),
    })
}

fn stream_error(error: anyhow::Error) -> ProtocolError {
    let message = error.to_string();
    ProtocolError {
        code: if message.contains("stream_busy") {
            "stream_busy"
        } else {
            "stream_failed"
        },
        message,
    }
}

fn validate_path(value: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(value);
    anyhow::ensure!(
        !path.is_absolute(),
        "terrain pack path must be repository-relative"
    );
    anyhow::ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "terrain pack path contains an invalid component"
    );
    let components = path.components().collect::<Vec<_>>();
    anyhow::ensure!(
        components.len() >= 3
            && components[0].as_os_str() == "out"
            && components[1].as_os_str() == "terrain",
        "terrain pack path must be under out/terrain"
    );
    anyhow::ensure!(
        path.extension().is_some_and(|extension| extension == "wlt"),
        "terrain pack must use the .wlt extension"
    );
    Ok(path.to_path_buf())
}

fn gate_error(error: anyhow::Error) -> ProtocolError {
    ProtocolError {
        code: "gate_failed",
        message: error.to_string(),
    }
}
