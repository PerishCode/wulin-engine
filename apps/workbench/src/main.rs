mod capture;
mod gpu_capture;
mod inspect;
mod renderer;

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use inspect::{ControlCommand, ControlKind, ProtocolError};
use renderer::Renderer;
use serde_json::json;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, w};

const CLIENT_WIDTH: u32 = 1280;
const CLIENT_HEIGHT: u32 = 720;
const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.035, 0.105, 0.14, 1.0];
static WINDOW_HANDLE: AtomicIsize = AtomicIsize::new(0);

fn main() {
    if let Err(error) = unsafe { run() } {
        eprintln!("workbench failed: {error:#}");
        std::process::exit(1);
    }
}

unsafe fn run() -> Result<()> {
    let hwnd = unsafe { create_window()? };
    WINDOW_HANDLE.store(hwnd.0 as isize, Ordering::Release);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), true) }
        .context("SetConsoleCtrlHandler failed")?;
    let mut renderer = unsafe { Renderer::new(hwnd, CLIENT_WIDTH, CLIENT_HEIGHT)? };
    let (inspect, commands) = inspect::InspectServer::start()?;
    let launched_by_sidecar = std::env::args().any(|arg| arg.starts_with("--sidecar-stamp="));
    let mut state = WorkbenchState::new(launched_by_sidecar);

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = renderer.render(state.clear_color, false)?;
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

        handle_commands(hwnd, &renderer, &mut state, &commands, &mut pending_capture);
        let capture_requested = pending_capture.is_some();
        if state.paused && !capture_requested {
            thread::sleep(Duration::from_millis(8));
            continue;
        }

        let frame_start = Instant::now();
        match unsafe { renderer.render(state.clear_color, capture_requested) } {
            Ok(captured) => complete_frame(
                &renderer,
                &mut state,
                &mut pending_capture,
                captured,
                frame_start.elapsed(),
            ),
            Err(error) => fail_frame(&mut state, &mut pending_capture, error),
        }
    }

    unsafe { renderer.wait_idle()? };
    WINDOW_HANDLE.store(0, Ordering::Release);
    unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), false) }
        .context("removing console control handler failed")?;
    Ok(())
}

struct PendingCapture {
    id: String,
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
    pending_capture: &mut Option<PendingCapture>,
    captured: Option<gpu_capture::CapturedPixels>,
    frame_duration: Duration,
) {
    state.record_frame_with_duration(frame_duration);
    let Some(request) = pending_capture.take() else {
        return;
    };
    let result = captured
        .context("capture request completed without pixels")
        .and_then(|pixels| {
            capture::write(
                pixels,
                capture::FrameContext {
                    capture_id: &request.id,
                    frame_index: state.frame_index,
                    clear_color: state.clear_color,
                    paused: state.paused,
                    launched_by_sidecar: state.launched_by_sidecar,
                    adapter: renderer.adapter_name(),
                    debug_layer: renderer.debug_layer(),
                    device_removed_reason: unsafe { renderer.device_removed_reason() },
                    last_error: state.last_error.as_deref(),
                    gpu_readback_ms: frame_duration.as_secs_f64() * 1_000.0,
                },
            )
        })
        .map_err(|error| capture_error(state, error));
    let _ = request.response.send(result);
}

fn fail_frame(
    state: &mut WorkbenchState,
    pending_capture: &mut Option<PendingCapture>,
    error: anyhow::Error,
) {
    let message = format!("{error:#}");
    state.last_error = Some(message.clone());
    state.paused = true;
    if let Some(request) = pending_capture.take() {
        let _ = request.response.send(Err(ProtocolError {
            code: "render_failed",
            message,
        }));
    }
}

fn handle_commands(
    hwnd: HWND,
    renderer: &Renderer,
    state: &mut WorkbenchState,
    commands: &Receiver<ControlCommand>,
    pending_capture: &mut Option<PendingCapture>,
) {
    while let Ok(command) = commands.try_recv() {
        let ControlCommand { kind, response } = command;
        let result = match kind {
            ControlKind::Status => status(hwnd, renderer, state),
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
            ControlKind::Capture(id) => {
                if pending_capture.is_none() {
                    *pending_capture = Some(PendingCapture { id, response });
                    continue;
                }
                Err(ProtocolError {
                    code: "capture_busy",
                    message: "a capture request is already pending".into(),
                })
            }
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

fn status(hwnd: HWND, renderer: &Renderer, state: &WorkbenchState) -> inspect::ControlResult {
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

unsafe fn create_window() -> Result<HWND> {
    let module = unsafe { GetModuleHandleW(None) }.context("GetModuleHandleW failed")?;
    let instance = HINSTANCE(module.0);
    let class_name = w!("WulinEngineWorkbenchWindow");
    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.context("LoadCursorW failed")?,
        lpszClassName: class_name,
        ..Default::default()
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(windows::core::Error::from_thread()).context("RegisterClassW failed");
    }

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: CLIENT_WIDTH as i32,
        bottom: CLIENT_HEIGHT as i32,
    };
    unsafe { AdjustWindowRect(&mut rect, style, false) }.context("AdjustWindowRect failed")?;
    unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Wulin Engine Workbench"),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            Some(instance),
            None,
        )
    }
    .context("CreateWindowExW failed")
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CLOSE => {
            let _ = unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe extern "system" fn console_ctrl_handler(control: u32) -> BOOL {
    if !matches!(control, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        return false.into();
    }
    let raw = WINDOW_HANDLE.load(Ordering::Acquire);
    if raw == 0 {
        return false.into();
    }
    let hwnd = HWND(raw as *mut _);
    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
        .is_ok()
        .into()
}
