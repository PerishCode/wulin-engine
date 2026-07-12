use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};

const DEFAULT_ENDPOINT: &str = "tcp://127.0.0.1:47631";
const MAX_FRAME_BYTES: u64 = 64 * 1024;

pub struct InspectServer {
    endpoint: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

pub struct ControlCommand {
    pub kind: ControlKind,
    pub response: SyncSender<ControlResult>,
}

pub enum ControlKind {
    Status,
    SetClearColor([f32; 4]),
    Pause,
    Resume,
    Capture(String),
}

pub type ControlResult = std::result::Result<Value, ProtocolError>;

#[derive(Debug)]
pub struct ProtocolError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventFrame {
    kind: String,
    id: String,
    verb: String,
    payload: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetClearColorPayload {
    rgba: [f32; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturePayload {
    id: String,
}

impl InspectServer {
    pub fn start() -> Result<(Self, Receiver<ControlCommand>)> {
        let endpoint =
            env::var("SIDECAR_INSPECT_SOCKET").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        let address = endpoint
            .strip_prefix("tcp://")
            .context("workbench inspect requires a tcp:// endpoint on Windows")?;
        if !(address.starts_with("127.0.0.1:") || address.starts_with("localhost:")) {
            bail!("workbench inspect endpoint must bind to loopback");
        }

        let listener = TcpListener::bind(address)
            .with_context(|| format!("failed to bind inspect endpoint {endpoint}"))?;
        listener
            .set_nonblocking(true)
            .context("failed to configure inspect listener")?;
        let (commands_tx, commands_rx) = mpsc::sync_channel(16);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name("workbench-inspect".into())
            .spawn(move || serve(listener, commands_tx, thread_stop))
            .context("failed to start inspect thread")?;

        Ok((
            Self {
                endpoint,
                stop,
                thread: Some(thread),
            },
            commands_rx,
        ))
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl Drop for InspectServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve(listener: TcpListener, commands: SyncSender<ControlCommand>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => handle_connection(stream, &commands),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                eprintln!("inspect accept failed: {error}");
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, commands: &SyncSender<ControlCommand>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let mut line = String::new();
    let read = stream
        .try_clone()
        .map(BufReader::new)
        .and_then(|reader| reader.take(MAX_FRAME_BYTES).read_line(&mut line));
    if let Err(error) = read {
        let _ = write_error(&mut stream, "", "invalid_frame", error.to_string());
        return;
    }

    let frame = match serde_json::from_str::<EventFrame>(line.trim()) {
        Ok(frame) => frame,
        Err(error) => {
            let _ = write_error(&mut stream, "", "invalid_frame", error.to_string());
            return;
        }
    };
    if frame.kind != "event" || frame.id.is_empty() {
        let _ = write_error(
            &mut stream,
            &frame.id,
            "invalid_frame",
            "expected a SidecarRuntime event frame".into(),
        );
        return;
    }

    let kind = match parse_control(&frame.verb, frame.payload) {
        Ok(kind) => kind,
        Err(error) => {
            let _ = write_error(&mut stream, &frame.id, error.code, error.message);
            return;
        }
    };
    let (response_tx, response_rx) = mpsc::sync_channel(1);
    if commands
        .send(ControlCommand {
            kind,
            response: response_tx,
        })
        .is_err()
    {
        let _ = write_error(
            &mut stream,
            &frame.id,
            "unavailable",
            "workbench control loop is unavailable".into(),
        );
        return;
    }

    match response_rx.recv_timeout(Duration::from_secs(4)) {
        Ok(Ok(payload)) => {
            let _ = write_frame(
                &mut stream,
                json!({"kind": "event_response", "id": frame.id, "payload": payload}),
            );
        }
        Ok(Err(error)) => {
            let _ = write_error(&mut stream, &frame.id, error.code, error.message);
        }
        Err(_) => {
            let _ = write_error(
                &mut stream,
                &frame.id,
                "timeout",
                "workbench control loop did not respond".into(),
            );
        }
    }
}

fn parse_control(verb: &str, payload: Value) -> ControlResultKind {
    match verb {
        "workbench.status" => Ok(ControlKind::Status),
        "workbench.pause" => Ok(ControlKind::Pause),
        "workbench.resume" => Ok(ControlKind::Resume),
        "workbench.capture" => {
            let payload: CapturePayload =
                serde_json::from_value(payload).map_err(|error| ProtocolError {
                    code: "invalid_payload",
                    message: error.to_string(),
                })?;
            if payload.id.is_empty()
                || payload.id.len() > 64
                || !payload
                    .id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            {
                return Err(ProtocolError {
                    code: "invalid_payload",
                    message: "capture id must contain 1..=64 ASCII letters, digits, '-' or '_'"
                        .into(),
                });
            }
            Ok(ControlKind::Capture(payload.id))
        }
        "workbench.set_clear_color" => {
            let payload: SetClearColorPayload =
                serde_json::from_value(payload).map_err(|error| ProtocolError {
                    code: "invalid_payload",
                    message: error.to_string(),
                })?;
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
        _ => Err(ProtocolError {
            code: "unknown_event",
            message: format!("unsupported event {verb:?}"),
        }),
    }
}

type ControlResultKind = std::result::Result<ControlKind, ProtocolError>;

fn write_error(
    stream: &mut TcpStream,
    id: &str,
    code: &'static str,
    message: String,
) -> std::io::Result<()> {
    write_frame(
        stream,
        json!({
            "kind": "event_error",
            "id": id,
            "error": {"code": code, "message": message}
        }),
    )
}

fn write_frame(stream: &mut TcpStream, frame: Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *stream, &frame)?;
    stream.write_all(b"\n")?;
    stream.flush()
}
