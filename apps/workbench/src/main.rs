mod bootstrap;
mod capture;
mod input;
mod inspect;
mod perception;
mod window;

use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use engine_runtime::{FrameRequest, Runtime};
use inspect::ProtocolError;
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
    let arguments = bootstrap::Arguments::parse()?;
    let hwnd = unsafe { window::create()? };
    let mut runtime = unsafe { Runtime::new(hwnd, window::WIDTH, window::HEIGHT)? };
    let (inspect, commands) = inspect::InspectServer::start()?;
    let startup = arguments
        .bootstrap
        .as_ref()
        .map_or_else(bootstrap::idle_json, bootstrap::Plan::pending_json);
    let mut state = WorkbenchState::new(arguments.launched_by_sidecar, startup);

    if let Some(plan) = arguments.bootstrap {
        state.startup = unsafe { bootstrap_runtime(hwnd, &mut runtime, &mut state, &plan)? };
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

    let mut message = MSG::default();
    let mut pending = PendingOperations::default();
    'running: loop {
        if !unsafe { pump_messages(&mut message) } {
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
    unsafe { window::teardown()? };
    Ok(())
}

unsafe fn bootstrap_runtime(
    hwnd: windows::Win32::Foundation::HWND,
    runtime: &mut Runtime,
    state: &mut WorkbenchState,
    plan: &bootstrap::Plan,
) -> Result<serde_json::Value> {
    runtime
        .open_terrain_pack(plan.terrain_path().to_path_buf())
        .context("bootstrap terrain source failed")?;
    runtime
        .open_cooked_object_pack(plan.object_path().to_path_buf())
        .context("bootstrap object source failed")?;
    let schedule = unsafe { runtime.schedule_global_composition(plan.global_config()) }
        .context("bootstrap canonical schedule failed")?;
    let started = Instant::now();
    let mut message = MSG::default();
    loop {
        if !unsafe { pump_messages(&mut message) } {
            bail!("workbench closed during canonical bootstrap");
        }
        state.input.ingest(window::drain_input());
        let frame_start = Instant::now();
        unsafe {
            let _ = runtime.frame(FrameRequest {
                clear_color: state.clear_color,
                capture: false,
                capture_object_ids: false,
                probe: false,
            })?;
        }
        state.record_frame_with_duration(frame_start.elapsed());
        if runtime.composition_enabled() {
            unsafe { window::show(hwnd) };
            return Ok(plan.ready_json(schedule, state.frame_index, started.elapsed()));
        }
        let status = runtime.composition_status();
        if !status["lastFailure"].is_null() {
            bail!("canonical bootstrap pair failed: {}", status["lastFailure"]);
        }
        if started.elapsed() >= bootstrap::TIMEOUT {
            bail!(
                "canonical bootstrap did not publish within {} seconds",
                bootstrap::TIMEOUT.as_secs()
            );
        }
    }
}

unsafe fn pump_messages(message: &mut MSG) -> bool {
    while unsafe { PeekMessageW(message, None, 0, 0, PM_REMOVE) }.as_bool() {
        if message.message == WM_QUIT {
            return false;
        }
        unsafe {
            let _ = TranslateMessage(message);
            DispatchMessageW(message);
        }
    }
    true
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
    input: input::HostInput,
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
            input: input::HostInput::new(),
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
