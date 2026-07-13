use serde::Deserialize;
use serde_json::Value;

use crate::perception::{PixelPoint, PixelRegion};

pub enum ControlKind {
    Status,
    SetClearColor([f32; 4]),
    Pause,
    Resume,
    Capture {
        id: String,
        collection: String,
    },
    PerceptionCapture {
        id: String,
        collection: String,
        region: Option<PixelRegion>,
        samples: Vec<PixelPoint>,
    },
    CameraStatus,
    CameraSetPose {
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    },
    CameraReset,
    SceneListObjects,
    LoadConfigure {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    LoadDisable,
    LoadStatus,
    LoadProbe,
    ResidentStream {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    ResidentStatus,
    AsyncResidentSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    AsyncResidentStatus,
    AsyncCopyGateArm,
    AsyncCopyGateRelease,
    CookedOpen {
        path: String,
    },
    CookedSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    CookedStatus,
    CookedIoGateArm,
    CookedIoGateRelease,
    MeshletStatus,
    MeshletConfigure {
        archetype_mask: u32,
        forced_lod: Option<u32>,
    },
    MeshletEnable,
    MeshletDisable,
    SkeletalStatus,
    SkeletalConfigure {
        animated_percent: u32,
        bone_count: u32,
        phase_count: u32,
        time_tick: u32,
        unique_poses: bool,
        forced_lod: Option<u32>,
    },
    SkeletalEnable,
    SkeletalDisable,
}

pub type ControlResult = std::result::Result<Value, ProtocolError>;
type ParsedControl = std::result::Result<ControlKind, ProtocolError>;

#[derive(Debug)]
pub struct ProtocolError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColorPayload {
    rgba: [f32; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturePayload {
    id: String,
    collection: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PerceptionPayload {
    id: String,
    collection: Option<String>,
    region: Option<PixelRegion>,
    #[serde(default)]
    samples: Vec<PixelPoint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CameraPayload {
    position: [f32; 3],
    target: [f32; 3],
    vertical_fov_degrees: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadPayload {
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CookedPayload {
    path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MeshletPayload {
    archetype_mask: u32,
    forced_lod: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletalPayload {
    animated_percent: u32,
    bone_count: u32,
    phase_count: u32,
    time_tick: u32,
    unique_poses: bool,
    forced_lod: Option<u32>,
}

pub fn parse_control(verb: &str, payload: Value) -> ParsedControl {
    match verb {
        "workbench.status" => Ok(ControlKind::Status),
        "workbench.pause" => Ok(ControlKind::Pause),
        "workbench.resume" => Ok(ControlKind::Resume),
        "camera.status" => Ok(ControlKind::CameraStatus),
        "camera.reset" => Ok(ControlKind::CameraReset),
        "scene.list_objects" => Ok(ControlKind::SceneListObjects),
        "load.disable" => Ok(ControlKind::LoadDisable),
        "load.status" => Ok(ControlKind::LoadStatus),
        "load.probe" => Ok(ControlKind::LoadProbe),
        "resident.status" => Ok(ControlKind::ResidentStatus),
        "async.status" => Ok(ControlKind::AsyncResidentStatus),
        "async.gate.arm" => Ok(ControlKind::AsyncCopyGateArm),
        "async.gate.release" => Ok(ControlKind::AsyncCopyGateRelease),
        "cooked.status" => Ok(ControlKind::CookedStatus),
        "cooked.gate.arm" => Ok(ControlKind::CookedIoGateArm),
        "cooked.gate.release" => Ok(ControlKind::CookedIoGateRelease),
        "meshlet.status" => Ok(ControlKind::MeshletStatus),
        "meshlet.enable" => Ok(ControlKind::MeshletEnable),
        "meshlet.disable" => Ok(ControlKind::MeshletDisable),
        "meshlet.configure" => parse_meshlet(payload),
        "skeletal.status" => Ok(ControlKind::SkeletalStatus),
        "skeletal.enable" => Ok(ControlKind::SkeletalEnable),
        "skeletal.disable" => Ok(ControlKind::SkeletalDisable),
        "skeletal.configure" => parse_skeletal(payload),
        "cooked.open" => parse_cooked(payload),
        "load.configure" => parse_load(payload, LoadTarget::Procedural),
        "resident.stream" => parse_load(payload, LoadTarget::Resident),
        "async.schedule" => parse_load(payload, LoadTarget::Async),
        "cooked.schedule" => parse_load(payload, LoadTarget::Cooked),
        "camera.set_pose" => parse_camera(payload),
        "workbench.capture" => parse_capture(payload),
        "perception.capture" => parse_perception(payload),
        "workbench.set_clear_color" => parse_color(payload),
        _ => Err(ProtocolError {
            code: "unknown_event",
            message: format!("unsupported event {verb:?}"),
        }),
    }
}

enum LoadTarget {
    Procedural,
    Resident,
    Async,
    Cooked,
}

fn parse_load(value: Value, target: LoadTarget) -> ParsedControl {
    let payload: LoadPayload = decode(value)?;
    let LoadPayload {
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    } = payload;
    Ok(match target {
        LoadTarget::Procedural => ControlKind::LoadConfigure {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Resident => ControlKind::ResidentStream {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Async => ControlKind::AsyncResidentSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Cooked => ControlKind::CookedSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
    })
}

fn parse_meshlet(value: Value) -> ParsedControl {
    let payload: MeshletPayload = decode(value)?;
    Ok(ControlKind::MeshletConfigure {
        archetype_mask: payload.archetype_mask,
        forced_lod: payload.forced_lod,
    })
}

fn parse_skeletal(value: Value) -> ParsedControl {
    let payload: SkeletalPayload = decode(value)?;
    Ok(ControlKind::SkeletalConfigure {
        animated_percent: payload.animated_percent,
        bone_count: payload.bone_count,
        phase_count: payload.phase_count,
        time_tick: payload.time_tick,
        unique_poses: payload.unique_poses,
        forced_lod: payload.forced_lod,
    })
}

fn parse_cooked(value: Value) -> ParsedControl {
    let payload: CookedPayload = decode(value)?;
    Ok(ControlKind::CookedOpen { path: payload.path })
}

fn parse_camera(value: Value) -> ParsedControl {
    let payload: CameraPayload = decode(value)?;
    Ok(ControlKind::CameraSetPose {
        position: payload.position,
        target: payload.target,
        vertical_fov_degrees: payload.vertical_fov_degrees,
    })
}

fn parse_capture(value: Value) -> ParsedControl {
    let payload: CapturePayload = decode(value)?;
    validate_name("capture id", &payload.id)?;
    let collection = payload.collection.unwrap_or_else(|| "operator".into());
    validate_name("collection", &collection)?;
    Ok(ControlKind::Capture {
        id: payload.id,
        collection,
    })
}

fn parse_perception(value: Value) -> ParsedControl {
    let payload: PerceptionPayload = decode(value)?;
    validate_name("capture id", &payload.id)?;
    let collection = payload.collection.unwrap_or_else(|| "operator".into());
    validate_name("collection", &collection)?;
    Ok(ControlKind::PerceptionCapture {
        id: payload.id,
        collection,
        region: payload.region,
        samples: payload.samples,
    })
}

fn parse_color(value: Value) -> ParsedControl {
    let payload: ColorPayload = decode(value)?;
    if payload.rgba.iter().any(|value| !value.is_finite())
        || payload
            .rgba
            .iter()
            .any(|value| !(0.0..=1.0).contains(value))
    {
        return Err(ProtocolError {
            code: "invalid_payload",
            message: "rgba values must be finite numbers in the range 0..=1".into(),
        });
    }
    Ok(ControlKind::SetClearColor(payload.rgba))
}

fn decode<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, ProtocolError> {
    serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })
}

fn validate_name(label: &str, value: &str) -> Result<(), ProtocolError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(ProtocolError {
            code: "invalid_payload",
            message: format!("{label} must contain 1..=64 ASCII letters, digits, '-' or '_'"),
        })
    }
}
