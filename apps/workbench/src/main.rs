mod capture;
mod inspect;
mod load;
mod perception;
mod rendering;
mod scene;
mod window;

use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use inspect::{ControlCommand, ControlKind, ProtocolError};
use rendering::Renderer;
use serde_json::json;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;

const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.035, 0.105, 0.14, 1.0];

fn main() {
    if let Err(error) = unsafe { run() } {
        eprintln!("workbench failed: {error:#}");
        std::process::exit(1);
    }
}

unsafe fn run() -> Result<()> {
    let hwnd = unsafe { window::create()? };
    let mut renderer = unsafe { Renderer::new(hwnd, window::WIDTH, window::HEIGHT)? };
    let (inspect, commands) = inspect::InspectServer::start()?;
    let launched_by_sidecar = std::env::args().any(|arg| arg.starts_with("--sidecar-stamp="));
    let mut state = WorkbenchState::new(launched_by_sidecar);
    let mut scene = scene::SceneState::new();

    unsafe {
        window::show(hwnd);
        let _ = renderer.render(state.clear_color, false, false, false, &scene)?;
    }
    state.record_frame();

    println!(
        "{}",
        json!({
            "role": "workbench",
            "endpoint": inspect.endpoint(),
            "instance_id": std::process::id().to_string()
        })
    );

    let mut message = MSG::default();
    let mut pending_capture = None;
    let mut pending_probe = None;
    'running: loop {
        while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
            if message.message == WM_QUIT {
                break 'running;
            }
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }

        handle_commands(
            hwnd,
            &mut renderer,
            &mut state,
            &mut scene,
            &commands,
            &mut pending_capture,
            &mut pending_probe,
        );
        let capture_requested = pending_capture.is_some();
        let probe_requested = pending_probe.is_some();
        let perception_requested = pending_capture
            .as_ref()
            .is_some_and(|request| request.perception.is_some());
        if state.paused && !capture_requested && !probe_requested {
            thread::sleep(Duration::from_millis(8));
            continue;
        }

        let frame_start = Instant::now();
        match unsafe {
            renderer.render(
                state.clear_color,
                capture_requested,
                perception_requested,
                probe_requested,
                &scene,
            )
        } {
            Ok(outcome) => complete_frame(
                &renderer,
                &mut state,
                &scene,
                &mut pending_capture,
                &mut pending_probe,
                outcome,
                frame_start.elapsed(),
            ),
            Err(error) => fail_frame(&mut state, &mut pending_capture, &mut pending_probe, error),
        }
    }

    unsafe { renderer.wait_idle()? };
    unsafe { window::teardown()? };
    Ok(())
}

struct PendingCapture {
    id: String,
    collection: String,
    perception: Option<perception::Request>,
    response: SyncSender<inspect::ControlResult>,
}

struct WorkbenchState {
    started_at: Instant,
    frame_index: u64,
    last_frame_ms: f64,
    paused: bool,
    clear_color: [f32; 4],
    last_error: Option<String>,
    launched_by_sidecar: bool,
}

impl WorkbenchState {
    fn new(launched_by_sidecar: bool) -> Self {
        Self {
            started_at: Instant::now(),
            frame_index: 0,
            last_frame_ms: 0.0,
            paused: false,
            clear_color: DEFAULT_CLEAR_COLOR,
            last_error: None,
            launched_by_sidecar,
        }
    }

    fn record_frame(&mut self) {
        self.frame_index += 1;
    }

    fn record_frame_with_duration(&mut self, duration: Duration) {
        self.record_frame();
        self.last_frame_ms = duration.as_secs_f64() * 1_000.0;
    }
}

fn complete_frame(
    renderer: &Renderer,
    state: &mut WorkbenchState,
    scene: &scene::SceneState,
    pending_capture: &mut Option<PendingCapture>,
    pending_probe: &mut Option<SyncSender<inspect::ControlResult>>,
    outcome: rendering::RenderOutcome,
    frame_duration: Duration,
) {
    state.record_frame_with_duration(frame_duration);
    if let Some(request) = pending_capture.take() {
        let result = outcome
            .capture
            .context("capture request completed without pixels")
            .and_then(|frame| {
                capture::write(
                    frame.color,
                    frame.object_ids,
                    capture::FrameContext {
                        capture_id: &request.id,
                        collection: &request.collection,
                        frame_index: state.frame_index,
                        clear_color: state.clear_color,
                        paused: state.paused,
                        launched_by_sidecar: state.launched_by_sidecar,
                        adapter: renderer.adapter_name(),
                        debug_layer: renderer.debug_layer(),
                        device_removed_reason: unsafe { renderer.device_removed_reason() },
                        last_error: state.last_error.as_deref(),
                        gpu_readback_ms: frame_duration.as_secs_f64() * 1_000.0,
                        spatial: scene.spatial_json(),
                        workload: load_status(renderer),
                        perception: request.perception.as_ref(),
                    },
                )
            })
            .map_err(|error| capture_error(state, error));
        let _ = request.response.send(result);
    }
    if let Some(response) = pending_probe.take() {
        let result = outcome
            .load_probe
            .context("load probe completed without GPU evidence")
            .and_then(|probe| serde_json::to_value(probe).context("load probe encoding failed"))
            .map_err(|error| capture_error(state, error));
        let _ = response.send(result);
    }
}

fn fail_frame(
    state: &mut WorkbenchState,
    pending_capture: &mut Option<PendingCapture>,
    pending_probe: &mut Option<SyncSender<inspect::ControlResult>>,
    error: anyhow::Error,
) {
    let message = format!("{error:#}");
    state.last_error = Some(message.clone());
    state.paused = true;
    if let Some(request) = pending_capture.take() {
        let _ = request.response.send(Err(ProtocolError {
            code: "render_failed",
            message: message.clone(),
        }));
    }
    if let Some(response) = pending_probe.take() {
        let _ = response.send(Err(ProtocolError {
            code: "render_failed",
            message,
        }));
    }
}

fn handle_commands(
    hwnd: HWND,
    renderer: &mut Renderer,
    state: &mut WorkbenchState,
    scene: &mut scene::SceneState,
    commands: &Receiver<ControlCommand>,
    pending_capture: &mut Option<PendingCapture>,
    pending_probe: &mut Option<SyncSender<inspect::ControlResult>>,
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
            ControlKind::LoadStatus => Ok(load_status(renderer)),
            ControlKind::LoadDisable => {
                renderer.disable_load();
                Ok(load_status(renderer))
            }
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
            .map(|config| {
                renderer.configure_load(config);
                load_status(renderer)
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
                } else if pending_capture.is_none() && pending_probe.is_none() {
                    *pending_probe = Some(response);
                    continue;
                } else {
                    Err(ProtocolError {
                        code: "capture_busy",
                        message: "a capture or probe request is already pending".into(),
                    })
                }
            }
            ControlKind::Capture { id, collection } => {
                if pending_capture.is_none() && pending_probe.is_none() {
                    *pending_capture = Some(PendingCapture {
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
                Ok(perception) if pending_capture.is_none() && pending_probe.is_none() => {
                    *pending_capture = Some(PendingCapture {
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

fn capture_error(state: &mut WorkbenchState, error: anyhow::Error) -> ProtocolError {
    let message = format!("{error:#}");
    state.last_error = Some(message.clone());
    ProtocolError {
        code: "capture_failed",
        message,
    }
}

fn load_status(renderer: &Renderer) -> serde_json::Value {
    renderer.load_config().map_or_else(
        || json!({"mode": "calibration", "load": null}),
        |config| json!({"mode": "region-load", "load": config.json()}),
    )
}

fn status(
    hwnd: HWND,
    renderer: &Renderer,
    state: &WorkbenchState,
    scene: &scene::SceneState,
) -> inspect::ControlResult {
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
