use std::sync::mpsc::Receiver;

use serde_json::json;
use windows::Win32::Foundation::HWND;

use crate::address::GlobalRegionConfig;
use crate::rendering::Renderer;
use crate::{PendingCapture, PendingOperations, WorkbenchState, perception, scene, window};

use super::protocol::{ControlKind, ControlResult, ProtocolError};
use super::server::ControlCommand;

pub(crate) fn handle_commands(
    hwnd: HWND,
    renderer: &mut Renderer,
    state: &mut WorkbenchState,
    scene: &mut scene::SceneState,
    commands: &Receiver<ControlCommand>,
    pending: &mut PendingOperations,
) {
    while let Ok(command) = commands.try_recv() {
        let ControlCommand { kind, response } = command;
        let result = match kind {
            ControlKind::Status => super::status::workbench(hwnd, renderer, state, scene),
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
            ControlKind::CameraStatus => Ok(scene.camera_json()),
            ControlKind::CameraReset => {
                scene.reset_camera();
                Ok(scene.camera_json())
            }
            ControlKind::CameraSetPose {
                position,
                target,
                vertical_fov_degrees,
            } => scene
                .set_camera(
                    position,
                    target,
                    vertical_fov_degrees,
                    renderer.calibration_mode_active(),
                )
                .map(|_| scene.camera_json())
                .map_err(|error| protocol_error("invalid_camera", error)),
            ControlKind::SceneListObjects => Ok(scene.objects_json()),
            world @ (ControlKind::WorldStatus
            | ControlKind::WorldRelocate { .. }
            | ControlKind::WorldRebase { .. }
            | ControlKind::WorldReset
            | ControlKind::WorldProbe) => super::world_control::dispatch(renderer, scene, world),
            ControlKind::TerrainSourceOpen { path } => {
                super::pack_control::validate(&path, super::pack_control::PackKind::Terrain)
                    .and_then(|path| renderer.open_terrain_pack(path))
                    .map_err(|error| protocol_error("pack_open_failed", error))
            }
            ControlKind::ObjectSourceOpen { path } => {
                super::pack_control::validate(&path, super::pack_control::PackKind::Objects)
                    .and_then(|path| renderer.open_cooked_object_pack(path))
                    .map_err(|error| protocol_error("pack_open_failed", error))
            }
            ControlKind::CanonicalStatus => Ok(renderer.composition_status()),
            ControlKind::CanonicalTimeStatus => Ok(renderer.presentation_time_status()),
            ControlKind::CanonicalTimePause => Ok(renderer.pause_presentation_time()),
            ControlKind::CanonicalTimeResume => Ok(renderer.resume_presentation_time()),
            ControlKind::CanonicalTimeSet { tick } => renderer
                .set_presentation_time(tick)
                .map_err(|error| protocol_error("invalid_presentation_time", error)),
            ControlKind::CanonicalTimeStep { ticks } => renderer
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
                    unsafe { renderer.schedule_global_composition(config) }.map_err(stream_error)
                }),
            ControlKind::CanonicalTraversalEnable => renderer
                .enable_composition_traversal()
                .map(|()| renderer.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalTraversalDisable => {
                renderer.disable_composition_traversal();
                Ok(renderer.composition_status())
            }
            ControlKind::CanonicalPrefetchEnable => renderer
                .enable_composition_prefetch()
                .map(|()| renderer.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalPrefetchDisable => renderer
                .disable_composition_prefetch()
                .map(|()| renderer.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalProbe => {
                if !renderer.composition_enabled() {
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
            ControlKind::ObjectIoGateArm => gate(renderer.arm_object_io_gate()),
            ControlKind::ObjectIoGateRelease => gate(renderer.release_object_io_gate()),
            ControlKind::ObjectCopyGateArm => gate(renderer.arm_async_copy_gate()),
            ControlKind::ObjectCopyGateRelease => {
                gate(unsafe { renderer.release_async_copy_gate() })
            }
            ControlKind::TerrainIoGateArm => gate(renderer.arm_terrain_io_gate()),
            ControlKind::TerrainIoGateRelease => gate(renderer.release_terrain_io_gate()),
            ControlKind::TerrainCopyGateArm => gate(renderer.arm_terrain_copy_gate()),
            ControlKind::TerrainCopyGateRelease => {
                gate(unsafe { renderer.release_terrain_copy_gate() })
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
