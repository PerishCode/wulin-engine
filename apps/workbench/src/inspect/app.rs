use std::sync::mpsc::Receiver;

use engine_runtime::{GlobalRegionConfig, Runtime};
use serde_json::json;
use windows::Win32::Foundation::HWND;

use crate::{PendingCapture, PendingOperations, WorkbenchState, perception, window};

use super::protocol::{ControlKind, ControlResult, ProtocolError};
use super::server::ControlCommand;

pub(crate) fn handle_commands(
    hwnd: HWND,
    runtime: &mut Runtime,
    state: &mut WorkbenchState,
    commands: &Receiver<ControlCommand>,
    pending: &mut PendingOperations,
) {
    while let Ok(command) = commands.try_recv() {
        let ControlCommand { kind, response } = command;
        let result = match kind {
            ControlKind::Status => super::status::workbench(hwnd, runtime, state),
            ControlKind::SetClearColor(color) => {
                state.clear_color = color;
                Ok(json!({"clearColor": color}))
            }
            ControlKind::Pause => {
                state.paused = true;
                Ok(json!({"paused": true}))
            }
            ControlKind::Resume => {
                state.paused = false;
                Ok(json!({"paused": false}))
            }
            ControlKind::InputStatus => Ok(state.input.status_json()),
            ControlKind::InputRecordStart => state
                .input
                .start_recording()
                .map_err(|error| protocol_error("input_record_failed", error)),
            ControlKind::InputRecordStop => state
                .input
                .stop_recording()
                .map_err(|error| protocol_error("input_record_failed", error)),
            ControlKind::InputReplay => state
                .input
                .replay()
                .map_err(|error| protocol_error("input_replay_failed", error)),
            ControlKind::InputPost { messages } => window::post_input(hwnd, &messages)
                .map(|()| json!({"postedMessageCount": messages.len()}))
                .map_err(|error| protocol_error("native_input_failed", error)),
            ControlKind::CameraStatus => Ok(runtime.camera_json()),
            ControlKind::CameraReset => Ok(runtime.reset_camera()),
            ControlKind::CameraSetPose {
                position,
                target,
                vertical_fov_degrees,
            } => runtime
                .set_camera(position, target, vertical_fov_degrees)
                .map_err(|error| protocol_error("invalid_camera", error)),
            ControlKind::SceneListObjects => Ok(runtime.objects_json()),
            world @ (ControlKind::WorldStatus
            | ControlKind::WorldRelocate { .. }
            | ControlKind::WorldRebase { .. }
            | ControlKind::WorldReset
            | ControlKind::WorldProbe) => super::world_control::dispatch(runtime, world),
            ControlKind::TerrainSourceOpen { path } => {
                super::validate_pack_path(&path, super::PackKind::Terrain)
                    .and_then(|path| runtime.open_terrain_pack(path))
                    .map_err(|error| protocol_error("pack_open_failed", error))
            }
            ControlKind::ObjectSourceOpen { path } => {
                super::validate_pack_path(&path, super::PackKind::Objects)
                    .and_then(|path| runtime.open_cooked_object_pack(path))
                    .map_err(|error| protocol_error("pack_open_failed", error))
            }
            ControlKind::CanonicalStatus => Ok(runtime.composition_status()),
            ControlKind::CanonicalTimeStatus => Ok(runtime.presentation_time_status()),
            ControlKind::CanonicalTimePause => Ok(runtime.pause_presentation_time()),
            ControlKind::CanonicalTimeResume => Ok(runtime.resume_presentation_time()),
            ControlKind::CanonicalTimeSet { tick } => runtime
                .set_presentation_time(tick)
                .map_err(|error| protocol_error("invalid_presentation_time", error)),
            ControlKind::CanonicalTimeStep { ticks } => runtime
                .step_presentation_time(ticks)
                .map_err(|error| protocol_error("invalid_presentation_time", error)),
            ControlKind::CanonicalSchedule {
                origin_x,
                origin_z,
                center_x,
                center_z,
                active_radius,
            } => GlobalRegionConfig::new(origin_x, origin_z, center_x, center_z, active_radius)
                .map_err(|error| protocol_error("invalid_global_config", error))
                .and_then(|config| {
                    unsafe { runtime.schedule_global_composition(config) }.map_err(stream_error)
                }),
            ControlKind::CanonicalTraversalEnable => runtime
                .enable_composition_traversal()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalTraversalDisable => {
                runtime.disable_composition_traversal();
                Ok(runtime.composition_status())
            }
            ControlKind::CanonicalPrefetchEnable => runtime
                .enable_composition_prefetch()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalPrefetchDisable => runtime
                .disable_composition_prefetch()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalProbe => {
                if !runtime.composition_enabled() {
                    Err(ProtocolError {
                        code: "canonical_unavailable",
                        message: "canonical composition must be published before probing".into(),
                    })
                } else if pending.is_idle() {
                    pending.probe = Some(response);
                    continue;
                } else {
                    Err(ProtocolError {
                        code: "capture_busy",
                        message: "a capture or probe request is already pending".into(),
                    })
                }
            }
            ControlKind::ObjectIoGateArm => gate(runtime.arm_object_io_gate()),
            ControlKind::ObjectIoGateRelease => gate(runtime.release_object_io_gate()),
            ControlKind::ObjectCopyGateArm => gate(runtime.arm_object_copy_gate()),
            ControlKind::ObjectCopyGateRelease => {
                gate(unsafe { runtime.release_object_copy_gate() })
            }
            ControlKind::TerrainIoGateArm => gate(runtime.arm_terrain_io_gate()),
            ControlKind::TerrainIoGateRelease => gate(runtime.release_terrain_io_gate()),
            ControlKind::TerrainCopyGateArm => gate(runtime.arm_terrain_copy_gate()),
            ControlKind::TerrainCopyGateRelease => {
                gate(unsafe { runtime.release_terrain_copy_gate() })
            }
            ControlKind::Capture { id, collection } => {
                if pending.is_idle() {
                    pending.capture = Some(PendingCapture {
                        id,
                        collection,
                        perception: None,
                        response,
                    });
                    continue;
                }
                Err(ProtocolError {
                    code: "capture_busy",
                    message: "a capture or probe request is already pending".into(),
                })
            }
            ControlKind::PerceptionCapture {
                id,
                collection,
                region,
                samples,
            } => match perception::Request::new(region, samples, window::WIDTH, window::HEIGHT) {
                Ok(perception) if pending.is_idle() => {
                    pending.capture = Some(PendingCapture {
                        id,
                        collection,
                        perception: Some(perception),
                        response,
                    });
                    continue;
                }
                Ok(_) => Err(ProtocolError {
                    code: "capture_busy",
                    message: "a capture or probe request is already pending".into(),
                }),
                Err(error) => Err(protocol_error("invalid_region", error)),
            },
        };
        let _ = response.send(result);
    }
}

fn gate(result: anyhow::Result<u64>) -> ControlResult {
    result
        .map(|fence| json!({"gateFence": fence}))
        .map_err(|error| protocol_error("gate_failed", error))
}

fn stream_error(error: anyhow::Error) -> ProtocolError {
    let code = if error.to_string().contains("busy") {
        "stream_busy"
    } else {
        "stream_failed"
    };
    protocol_error(code, error)
}

fn protocol_error(code: &'static str, error: impl std::fmt::Display) -> ProtocolError {
    ProtocolError {
        code,
        message: error.to_string(),
    }
}
