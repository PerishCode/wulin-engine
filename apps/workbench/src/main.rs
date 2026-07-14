mod capture;
mod inspect;
mod load;
mod perception;
mod rendering;
mod resident;
mod scene;
mod streaming;
mod window;
mod world;

pub(crate) use streaming::{address, async_resident, cooked, objects, terrain};

use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use inspect::ProtocolError;
use rendering::Renderer;
use serde_json::json;
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
        let _ = renderer.render(state.clear_color, false, false, false, &mut scene)?;
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
    let mut pending = PendingOperations::default();
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

        inspect::handle_commands(
            hwnd,
            &mut renderer,
            &mut state,
            &mut scene,
            &commands,
            &mut pending,
        );
        let capture_requested = pending.capture.is_some();
        let probe_requested = pending.probe.is_some();
        let stream_requested = pending.stream.is_some();
        let perception_requested = pending
            .capture
            .as_ref()
            .is_some_and(|request| request.perception.is_some());
        if state.paused && !capture_requested && !probe_requested && !stream_requested {
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
                &mut scene,
            )
        } {
            Ok(outcome) => complete_frame(
                &renderer,
                &mut state,
                &scene,
                &mut pending,
                outcome,
                frame_start.elapsed(),
            ),
            Err(error) => fail_frame(&mut state, &mut pending, error),
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

#[derive(Default)]
struct PendingOperations {
    capture: Option<PendingCapture>,
    probe: Option<SyncSender<inspect::ControlResult>>,
    stream: Option<SyncSender<inspect::ControlResult>>,
}

impl PendingOperations {
    fn is_idle(&self) -> bool {
        self.capture.is_none() && self.probe.is_none() && self.stream.is_none()
    }
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
    pending: &mut PendingOperations,
    outcome: rendering::RenderOutcome,
    frame_duration: Duration,
) {
    state.record_frame_with_duration(frame_duration);
    if let Some(request) = pending.capture.take() {
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
                        workload: inspect::load_status(renderer),
                        perception: request.perception.as_ref(),
                    },
                )
            })
            .map_err(|error| capture_error(state, error));
        let _ = request.response.send(result);
    }
    if let Some(response) = pending.probe.take() {
        let result = outcome
            .composition_probe
            .map(|probe| serde_json::to_value(probe).context("composition probe encoding failed"))
            .or_else(|| {
                outcome.terrain_probe.map(|probe| {
                    serde_json::to_value(probe).context("terrain probe encoding failed")
                })
            })
            .or_else(|| {
                outcome.surface_probe.map(|probe| {
                    serde_json::to_value(probe).context("surface probe encoding failed")
                })
            })
            .or_else(|| {
                outcome.skeletal_probe.map(|probe| {
                    serde_json::to_value(probe).context("skeletal probe encoding failed")
                })
            })
            .or_else(|| {
                outcome.meshlet_probe.map(|probe| {
                    serde_json::to_value(probe).context("meshlet probe encoding failed")
                })
            })
            .or_else(|| {
                outcome
                    .load_probe
                    .map(|probe| serde_json::to_value(probe).context("load probe encoding failed"))
            })
            .context("load probe completed without GPU evidence")
            .and_then(|probe| probe)
            .map_err(|error| capture_error(state, error));
        let _ = response.send(result);
    }
    if let Some(response) = pending.stream.take() {
        let result = outcome
            .resident_stream
            .context("resident stream completed without transaction evidence")
            .and_then(|report| {
                serde_json::to_value(report).context("resident stream encoding failed")
            })
            .map_err(|error| capture_error(state, error));
        let _ = response.send(result);
    }
}

fn fail_frame(state: &mut WorkbenchState, pending: &mut PendingOperations, error: anyhow::Error) {
    let message = format!("{error:#}");
    state.last_error = Some(message.clone());
    state.paused = true;
    if let Some(request) = pending.capture.take() {
        let _ = request.response.send(Err(ProtocolError {
            code: "render_failed",
            message: message.clone(),
        }));
    }
    if let Some(response) = pending.probe.take() {
        let _ = response.send(Err(ProtocolError {
            code: "render_failed",
            message: message.clone(),
        }));
    }
    if let Some(response) = pending.stream.take() {
        let _ = response.send(Err(ProtocolError {
            code: "render_failed",
            message,
        }));
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
