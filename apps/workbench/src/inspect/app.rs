use std::sync::mpsc::{Receiver, SyncSender};

use serde_json::json;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, IsWindowVisible,
};

use crate::rendering::Renderer;
use crate::{PendingCapture, PendingOperations, WorkbenchState, load, perception, scene, window};

use super::server::{ControlCommand, ControlKind, ControlResult, ProtocolError};

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
            ControlKind::Status => status(hwnd, renderer, state, scene),
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
                .set_camera(position, target, vertical_fov_degrees)
                .map(|_| scene.camera_json())
                .map_err(|error| ProtocolError {
                    code: "invalid_camera",
                    message: error.to_string(),
                }),
            ControlKind::SceneListObjects => Ok(scene.objects_json()),
            ControlKind::LoadStatus | ControlKind::ResidentStatus => Ok(load_status(renderer)),
            ControlKind::AsyncResidentStatus => Ok(renderer.async_resident_status()),
            ControlKind::AsyncCopyGateArm => renderer
                .arm_async_copy_gate()
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            ControlKind::AsyncCopyGateRelease => unsafe { renderer.release_async_copy_gate() }
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            ControlKind::LoadDisable => renderer
                .disable_load()
                .map(|()| load_status(renderer))
                .map_err(|error| ProtocolError {
                    code: "stream_busy",
                    message: error.to_string(),
                }),
            ControlKind::LoadConfigure {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => load::LoadConfig::new(
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            )
            .and_then(|config| {
                renderer
                    .configure_load(config)
                    .map(|()| load_status(renderer))
            })
            .map_err(|error| ProtocolError {
                code: "invalid_load_config",
                message: error.to_string(),
            }),
            ControlKind::LoadProbe => {
                if renderer.load_config().is_none() {
                    Err(ProtocolError {
                        code: "load_disabled",
                        message: "load mode must be configured before probing".into(),
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
            ControlKind::ResidentStream {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => match begin_resident_stream(
                renderer,
                pending,
                &response,
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            ) {
                Some(result) => result,
                None => continue,
            },
            ControlKind::AsyncResidentSchedule {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => begin_async_stream(
                renderer,
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            ),
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
                    message: "a capture request is already pending".into(),
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
                    message: "a capture request is already pending".into(),
                }),
                Err(error) => Err(ProtocolError {
                    code: "invalid_region",
                    message: error.to_string(),
                }),
            },
        };
        let _ = response.send(result);
    }
}

fn begin_resident_stream(
    renderer: &mut Renderer,
    pending: &mut PendingOperations,
    response: &SyncSender<ControlResult>,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> Option<ControlResult> {
    let config = match load::LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    ) {
        Ok(config) => config,
        Err(error) => {
            return Some(Err(ProtocolError {
                code: "invalid_load_config",
                message: error.to_string(),
            }));
        }
    };
    if !pending.is_idle() {
        return Some(Err(ProtocolError {
            code: "capture_busy",
            message: "a capture, probe, or stream request is already pending".into(),
        }));
    }
    match unsafe { renderer.stream_resident(config) } {
        Ok(()) => {
            pending.stream = Some(response.clone());
            None
        }
        Err(error) => Some(Err(ProtocolError {
            code: "stream_failed",
            message: error.to_string(),
        })),
    }
}

fn begin_async_stream(
    renderer: &mut Renderer,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> ControlResult {
    let config = load::LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    )
    .map_err(|error| ProtocolError {
        code: "invalid_load_config",
        message: error.to_string(),
    })?;
    let report = unsafe { renderer.stream_async_resident(config) }.map_err(|error| {
        let message = error.to_string();
        ProtocolError {
            code: if message.contains("stream_busy") {
                "stream_busy"
            } else {
                "stream_failed"
            },
            message,
        }
    })?;
    serde_json::to_value(report).map_err(|error| ProtocolError {
        code: "stream_failed",
        message: error.to_string(),
    })
}

pub(crate) fn load_status(renderer: &Renderer) -> serde_json::Value {
    if renderer.async_resident_enabled() {
        json!({
            "mode": "async-resident-load",
            "load": renderer.async_resident_config().map(|config| config.json()),
            "async": renderer.async_resident_status(),
        })
    } else if let Some(config) = renderer.resident_config() {
        json!({"mode": "resident-load", "load": config.json()})
    } else if let Some(config) = renderer.load_config() {
        json!({"mode": "region-load", "load": config.json()})
    } else {
        json!({"mode": "calibration", "load": null})
    }
}

fn status(
    hwnd: HWND,
    renderer: &Renderer,
    state: &WorkbenchState,
    scene: &scene::SceneState,
) -> ControlResult {
    let mut client = RECT::default();
    unsafe { GetClientRect(hwnd, &mut client) }.map_err(internal_error)?;
    let device_removed_reason = unsafe { renderer.device_removed_reason() };
    Ok(json!({
        "schemaVersion": 1,
        "processId": std::process::id(),
        "launchedBySidecar": state.launched_by_sidecar,
        "uptimeMs": state.started_at.elapsed().as_millis(),
        "state": if state.paused { "paused" } else { "running" },
        "frameIndex": state.frame_index,
        "lastFrameMs": state.last_frame_ms,
        "clearColor": state.clear_color,
        "spatial": scene.spatial_json(),
        "workload": load_status(renderer),
        "window": {
            "handle": format!("0x{:X}", hwnd.0 as usize),
            "width": client.right - client.left,
            "height": client.bottom - client.top,
            "visible": unsafe { IsWindowVisible(hwnd) }.as_bool(),
            "foreground": unsafe { GetForegroundWindow() } == hwnd
        },
        "renderer": {
            "api": "D3D12",
            "adapter": renderer.adapter_name(),
            "featureLevel": "12_1",
            "swapChainBuffers": 2,
            "format": "R8G8B8A8_UNORM",
            "vsync": true,
            "debugLayer": renderer.debug_layer(),
            "deviceRemovedReason": device_removed_reason
        },
        "lastError": state.last_error
    }))
}

fn internal_error(error: windows::core::Error) -> ProtocolError {
    ProtocolError {
        code: "internal_error",
        message: error.to_string(),
    }
}
