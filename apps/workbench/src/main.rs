mod capture;
mod inspect;
mod perception;

use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use engine_runtime::{FrameRequest, Runtime};
use inspect::ProtocolError;
use reference_host::{HostInput, bootstrap, window};
use serde_json::json;

const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.035, 0.105, 0.14, 1.0];
pub(crate) const WINDOW_WIDTH: u32 = 1280;
pub(crate) const WINDOW_HEIGHT: u32 = 720;
const WINDOW_CONFIG: window::Config = window::Config {
    class_name: "WulinEngineWorkbenchWindow",
    title: "Wulin Engine Workbench",
    width: WINDOW_WIDTH,
    height: WINDOW_HEIGHT,
};

fn main() {
    if let Err(error) = unsafe { run() } {
        eprintln!("workbench failed: {error:#}");
        std::process::exit(1);
    }
}

unsafe fn run() -> Result<()> {
    let arguments = bootstrap::Arguments::parse()?;
    let hwnd = window::create(WINDOW_CONFIG)?;
    let mut runtime = unsafe { Runtime::new(hwnd, WINDOW_WIDTH, WINDOW_HEIGHT)? };
    let (inspect, commands) = inspect::InspectServer::start()?;
    let startup = arguments
        .bootstrap
        .as_ref()
        .map_or_else(bootstrap::idle_json, bootstrap::Plan::pending_json);
    let mut state = WorkbenchState::new(arguments.launched_by_sidecar, startup);

    if let Some(plan) = arguments.bootstrap {
        let ready =
            unsafe { bootstrap::drive(&mut runtime, &mut state.input, &plan, state.clear_color)? };
        state.frame_index = ready.frame_count;
        state.last_frame_ms = ready.last_frame_duration.as_secs_f64() * 1_000.0;
        state.startup = ready.status;
        unsafe { window::show(hwnd) };
    } else {
        unsafe {
            window::show(hwnd);
            let _ = runtime.frame(FrameRequest {
                clear_color: state.clear_color,
                capture: false,
                capture_object_ids: false,
                probe: false,
            })?;
        }
        state.record_frame();
    }

    println!(
        "{}",
        json!({
            "role": "workbench",
            "endpoint": inspect.endpoint(),
            "instance_id": std::process::id().to_string(),
            "startup": state.startup["mode"]
        })
    );

    let mut pending = PendingOperations::default();
    'running: loop {
        if !window::pump_messages() {
            break 'running;
        }

        state.input.ingest(window::drain_input());
        inspect::handle_commands(hwnd, &mut runtime, &mut state, &commands, &mut pending);
        let capture_requested = pending.capture.is_some();
        let probe_requested = pending.probe.is_some();
        let perception_requested = pending
            .capture
            .as_ref()
            .is_some_and(|request| request.perception.is_some());
        if state.paused && !capture_requested && !probe_requested {
            thread::sleep(Duration::from_millis(8));
            continue;
        }

        let frame_start = Instant::now();
        match unsafe {
            runtime.frame(FrameRequest {
                clear_color: state.clear_color,
                capture: capture_requested,
                capture_object_ids: perception_requested,
                probe: probe_requested,
            })
        } {
            Ok(outcome) => complete_frame(
                &runtime,
                &mut state,
                &mut pending,
                outcome,
                frame_start.elapsed(),
            ),
            Err(error) => fail_frame(&mut state, &mut pending, error),
        }
    }

    unsafe { runtime.wait_idle()? };
    window::teardown()?;
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
}

impl PendingOperations {
    fn is_idle(&self) -> bool {
        self.capture.is_none() && self.probe.is_none()
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
    input: HostInput,
    startup: serde_json::Value,
}

impl WorkbenchState {
    fn new(launched_by_sidecar: bool, startup: serde_json::Value) -> Self {
        Self {
            started_at: Instant::now(),
            frame_index: 0,
            last_frame_ms: 0.0,
            paused: false,
            clear_color: DEFAULT_CLEAR_COLOR,
            last_error: None,
            launched_by_sidecar,
            input: HostInput::new(),
            startup,
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
    runtime: &Runtime,
    state: &mut WorkbenchState,
    pending: &mut PendingOperations,
    outcome: engine_runtime::RenderOutcome,
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
                        adapter: runtime.adapter_name(),
                        debug_layer: runtime.debug_layer(),
                        device_removed_reason: unsafe { runtime.device_removed_reason() },
                        last_error: state.last_error.as_deref(),
                        gpu_readback_ms: frame_duration.as_secs_f64() * 1_000.0,
                        spatial: runtime.spatial_json(),
                        workload: inspect::workload(runtime),
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
            .context("canonical probe completed without GPU evidence")
            .and_then(|probe| probe)
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
}

fn capture_error(state: &mut WorkbenchState, error: anyhow::Error) -> ProtocolError {
    let message = format!("{error:#}");
    state.last_error = Some(message.clone());
    ProtocolError {
        code: "capture_failed",
        message,
    }
}
