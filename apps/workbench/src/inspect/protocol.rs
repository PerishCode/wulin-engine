use serde::Deserialize;
use serde_json::Value;

use reference_host::input::PostedMessage;

use crate::perception::{PixelPoint, PixelRegion};

pub enum ControlKind {
    Status,
    SetClearColor([f32; 4]),
    Pause,
    Resume,
    InputStatus,
    InputRecordStart,
    InputRecordStop,
    InputReplay,
    InputPost {
        messages: Vec<PostedMessage>,
    },
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
    TerrainSourceOpen {
        path: String,
    },
    ObjectSourceOpen {
        path: String,
    },
    CanonicalStatus,
    CanonicalTimeStatus,
    CanonicalTimePause,
    CanonicalTimeResume,
    CanonicalTimeSet {
        tick: u32,
    },
    CanonicalTimeStep {
        ticks: u32,
    },
    CanonicalSchedule {
        origin_x: i64,
        origin_z: i64,
        center_x: i64,
        center_z: i64,
        active_radius: u32,
    },
    CanonicalTraversalEnable,
    CanonicalTraversalDisable,
    CanonicalPrefetchEnable,
    CanonicalPrefetchDisable,
    CanonicalProbe,
    CanonicalTerrainHeight {
        region_x: i64,
        region_z: i64,
        local_x_q9: i32,
        local_z_q9: i32,
    },
    ObjectIoGateArm,
    ObjectIoGateRelease,
    ObjectCopyGateArm,
    ObjectCopyGateRelease,
    TerrainIoGateArm,
    TerrainIoGateRelease,
    TerrainCopyGateArm,
    TerrainCopyGateRelease,
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
struct PackPayload {
    path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalSchedulePayload {
    origin_x: i64,
    origin_z: i64,
    center_x: i64,
    center_z: i64,
    active_radius: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalTimeSetPayload {
    tick: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalTimeStepPayload {
    ticks: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalTerrainHeightPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputPostPayload {
    messages: Vec<InputPostMessage>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum InputPostMessage {
    Key {
        key: u16,
        down: bool,
        #[serde(default)]
        system: bool,
    },
    FocusLost,
}

pub fn parse_control(verb: &str, payload: Value) -> ParsedControl {
    match verb {
        "workbench.status" => Ok(ControlKind::Status),
        "workbench.pause" => Ok(ControlKind::Pause),
        "workbench.resume" => Ok(ControlKind::Resume),
        "input.status" => Ok(ControlKind::InputStatus),
        "input.record.start" => Ok(ControlKind::InputRecordStart),
        "input.record.stop" => Ok(ControlKind::InputRecordStop),
        "input.replay" => Ok(ControlKind::InputReplay),
        "input.native.post" => parse_input_post(payload),
        "workbench.capture" => parse_capture(payload),
        "workbench.set_clear_color" => parse_color(payload),
        "perception.capture" => parse_perception(payload),
        "camera.status" => Ok(ControlKind::CameraStatus),
        "camera.set_pose" => parse_camera(payload),
        "camera.reset" => Ok(ControlKind::CameraReset),
        "source.terrain.open" => parse_pack(payload, true),
        "source.objects.open" => parse_pack(payload, false),
        "canonical.status" => Ok(ControlKind::CanonicalStatus),
        "canonical.time.status" => Ok(ControlKind::CanonicalTimeStatus),
        "canonical.time.pause" => Ok(ControlKind::CanonicalTimePause),
        "canonical.time.resume" => Ok(ControlKind::CanonicalTimeResume),
        "canonical.time.set" => parse_canonical_time_set(payload),
        "canonical.time.step" => parse_canonical_time_step(payload),
        "canonical.schedule" => parse_canonical_schedule(payload),
        "canonical.traversal.enable" => Ok(ControlKind::CanonicalTraversalEnable),
        "canonical.traversal.disable" => Ok(ControlKind::CanonicalTraversalDisable),
        "canonical.prefetch.enable" => Ok(ControlKind::CanonicalPrefetchEnable),
        "canonical.prefetch.disable" => Ok(ControlKind::CanonicalPrefetchDisable),
        "canonical.probe" => Ok(ControlKind::CanonicalProbe),
        "canonical.terrain.height" => parse_canonical_terrain_height(payload),
        "canonical.objects.io_gate.arm" => Ok(ControlKind::ObjectIoGateArm),
        "canonical.objects.io_gate.release" => Ok(ControlKind::ObjectIoGateRelease),
        "canonical.objects.copy_gate.arm" => Ok(ControlKind::ObjectCopyGateArm),
        "canonical.objects.copy_gate.release" => Ok(ControlKind::ObjectCopyGateRelease),
        "canonical.terrain.io_gate.arm" => Ok(ControlKind::TerrainIoGateArm),
        "canonical.terrain.io_gate.release" => Ok(ControlKind::TerrainIoGateRelease),
        "canonical.terrain.copy_gate.arm" => Ok(ControlKind::TerrainCopyGateArm),
        "canonical.terrain.copy_gate.release" => Ok(ControlKind::TerrainCopyGateRelease),
        _ => Err(ProtocolError {
            code: "unknown_event",
            message: format!("unsupported event {verb:?}"),
        }),
    }
}

fn parse_input_post(value: Value) -> ParsedControl {
    let payload: InputPostPayload = decode(value)?;
    if payload.messages.is_empty() || payload.messages.len() > 64 {
        return Err(ProtocolError {
            code: "invalid_payload",
            message: "native input post requires 1..=64 messages".into(),
        });
    }
    let mut messages = Vec::with_capacity(payload.messages.len());
    for message in payload.messages {
        messages.push(match message {
            InputPostMessage::Key { key, down, system } => {
                let key = u8::try_from(key).map_err(|_| ProtocolError {
                    code: "invalid_payload",
                    message: "native input key must be in 1..=255".into(),
                })?;
                if key == 0 {
                    return Err(ProtocolError {
                        code: "invalid_payload",
                        message: "native input key must be in 1..=255".into(),
                    });
                }
                PostedMessage::Key { key, down, system }
            }
            InputPostMessage::FocusLost => PostedMessage::FocusLost,
        });
    }
    Ok(ControlKind::InputPost { messages })
}

fn parse_pack(value: Value, terrain: bool) -> ParsedControl {
    let payload: PackPayload = decode(value)?;
    Ok(if terrain {
        ControlKind::TerrainSourceOpen { path: payload.path }
    } else {
        ControlKind::ObjectSourceOpen { path: payload.path }
    })
}

fn parse_canonical_schedule(value: Value) -> ParsedControl {
    let payload: CanonicalSchedulePayload = decode(value)?;
    Ok(ControlKind::CanonicalSchedule {
        origin_x: payload.origin_x,
        origin_z: payload.origin_z,
        center_x: payload.center_x,
        center_z: payload.center_z,
        active_radius: payload.active_radius,
    })
}

fn parse_canonical_time_set(value: Value) -> ParsedControl {
    let payload: CanonicalTimeSetPayload = decode(value)?;
    Ok(ControlKind::CanonicalTimeSet { tick: payload.tick })
}

fn parse_canonical_time_step(value: Value) -> ParsedControl {
    let payload: CanonicalTimeStepPayload = decode(value)?;
    Ok(ControlKind::CanonicalTimeStep {
        ticks: payload.ticks,
    })
}

fn parse_canonical_terrain_height(value: Value) -> ParsedControl {
    let payload: CanonicalTerrainHeightPayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainHeight {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
    })
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
