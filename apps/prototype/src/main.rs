use anyhow::{Context, Result};
use engine_runtime::{FrameRequest, Runtime};
use reference_host::{HostInput, bootstrap, window};
use serde_json::json;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const ESCAPE: u8 = 0x1B;
const CLEAR_COLOR: [f32; 4] = [0.035, 0.105, 0.14, 1.0];
const WINDOW_CONFIG: window::Config = window::Config {
    class_name: "WulinEnginePrototypeWindow",
    title: "Wulin Engine Prototype",
    width: WIDTH,
    height: HEIGHT,
};

fn main() {
    if let Err(error) = unsafe { run() } {
        eprintln!("prototype failed: {error:#}");
        std::process::exit(1);
    }
}

unsafe fn run() -> Result<()> {
    let arguments = bootstrap::Arguments::parse()?;
    let plan = arguments
        .bootstrap
        .context("prototype requires --bootstrap=<path>")?;
    let hwnd = window::create(WINDOW_CONFIG)?;
    let mut runtime = unsafe { Runtime::new(hwnd, WIDTH, HEIGHT)? };
    let mut input = HostInput::new();
    let ready = unsafe { bootstrap::drive(&mut runtime, &mut input, &plan, CLEAR_COLOR)? };
    unsafe { window::show(hwnd) };

    println!(
        "{}",
        json!({
            "role": "prototype",
            "instance_id": std::process::id().to_string(),
            "startup": ready.status,
        })
    );

    'running: loop {
        if !window::pump_messages() {
            break 'running;
        }
        input.ingest(window::drain_input());
        if input.is_held(ESCAPE) {
            window::request_close(hwnd)?;
            continue;
        }
        unsafe {
            let _ = runtime.frame(FrameRequest {
                clear_color: CLEAR_COLOR,
                capture: false,
                capture_object_ids: false,
                probe: false,
            })?;
        }
    }

    unsafe { runtime.wait_idle()? };
    window::teardown()?;
    Ok(())
}
