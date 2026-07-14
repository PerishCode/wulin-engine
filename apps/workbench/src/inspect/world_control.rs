use engine_runtime::{RegionCoord, Runtime};
use serde::Deserialize;
use serde_json::Value;

use super::protocol::{ControlKind, ControlResult, ProtocolError};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionPayload {
    region_x: i64,
    region_z: i64,
}

pub(super) fn parse_relocate(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload = decode(value)?;
    Ok(ControlKind::WorldRelocate {
        region_x: payload.region_x,
        region_z: payload.region_z,
    })
}

pub(super) fn parse_rebase(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload = decode(value)?;
    Ok(ControlKind::WorldRebase {
        region_x: payload.region_x,
        region_z: payload.region_z,
    })
}

pub(super) fn dispatch(runtime: &mut Runtime, kind: ControlKind) -> ControlResult {
    match kind {
        ControlKind::WorldStatus => runtime.world_json().map_err(world_error),
        ControlKind::WorldRelocate { region_x, region_z } => {
            require_calibration(runtime)?;
            runtime
                .relocate_world(RegionCoord::new(region_x, region_z))
                .map_err(world_error)
        }
        ControlKind::WorldRebase { region_x, region_z } => {
            require_calibration(runtime)?;
            runtime
                .rebase_world(RegionCoord::new(region_x, region_z))
                .map_err(world_error)
        }
        ControlKind::WorldReset => {
            require_calibration(runtime)?;
            runtime.reset_world().map_err(world_error)
        }
        ControlKind::WorldProbe => {
            require_calibration(runtime)?;
            runtime.world_probe_json().map_err(world_error)
        }
        _ => Err(ProtocolError {
            code: "internal_error",
            message: "non-world control reached world dispatcher".into(),
        }),
    }
}

fn decode(value: Value) -> Result<RegionPayload, ProtocolError> {
    serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })
}

fn require_calibration(runtime: &Runtime) -> Result<(), ProtocolError> {
    if runtime.calibration_mode_active() {
        Ok(())
    } else {
        Err(ProtocolError {
            code: "world_mode_required",
            message: "world control requires calibration mode".into(),
        })
    }
}

fn world_error(error: anyhow::Error) -> ProtocolError {
    ProtocolError {
        code: "invalid_world_space",
        message: error.to_string(),
    }
}
