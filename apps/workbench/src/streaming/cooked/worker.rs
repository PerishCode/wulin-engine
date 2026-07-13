use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use region_format::RegionPack;

use crate::async_resident::RegionAssignment;
use crate::resident::RegionUpload;

use super::IoMetrics;

#[derive(Clone, Default)]
pub(super) struct IoGate {
    state: Arc<(Mutex<GateState>, Condvar)>,
}

pub(super) struct PackWorker {
    request: Option<SyncSender<ReadRequest>>,
    pub completion: Receiver<ReadCompletion>,
    shutdown: Arc<AtomicBool>,
    gate: IoGate,
    thread: Option<JoinHandle<()>>,
}

pub(super) struct ReadRequest {
    pub transaction_id: u64,
    pub assignments: Vec<RegionAssignment>,
    pub gate_fence: Option<u64>,
    pub queued_at: Instant,
}

pub(super) struct ReadCompletion {
    pub transaction_id: u64,
    pub result: std::result::Result<(Vec<RegionUpload>, IoMetrics), ReadFailure>,
}

pub(super) struct ReadFailure {
    pub message: String,
    pub metrics: IoMetrics,
}

#[derive(Default)]
struct GateState {
    completed: u64,
}

impl IoGate {
    pub fn completed(&self) -> u64 {
        self.state.0.lock().expect("I/O gate poisoned").completed
    }

    pub fn signal(&self, value: u64) {
        self.state.0.lock().expect("I/O gate poisoned").completed = value;
        self.state.1.notify_all();
    }
}

impl PackWorker {
    pub fn start(pack: RegionPack, gate: IoGate) -> Result<Self> {
        let (request_tx, request_rx) = mpsc::sync_channel(1);
        let (completion_tx, completion_rx) = mpsc::sync_channel(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_gate = gate.clone();
        let thread = thread::Builder::new()
            .name("cooked-region-io".into())
            .spawn(move || {
                worker_loop(
                    pack,
                    request_rx,
                    completion_tx,
                    worker_gate,
                    worker_shutdown,
                )
            })
            .context("failed to start cooked region worker")?;
        Ok(Self {
            request: Some(request_tx),
            completion: completion_rx,
            shutdown,
            gate,
            thread: Some(thread),
        })
    }

    pub fn send(&self, request: ReadRequest) -> Result<()> {
        match self
            .request
            .as_ref()
            .context("cooked region worker is stopped")?
            .try_send(request)
        {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => bail!("stream_busy"),
            Err(TrySendError::Disconnected(_)) => bail!("cooked region worker disconnected"),
        }
    }
}

impl Drop for PackWorker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.gate.state.1.notify_all();
        self.request.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn worker_loop(
    mut pack: RegionPack,
    requests: Receiver<ReadRequest>,
    completions: SyncSender<ReadCompletion>,
    gate: IoGate,
    shutdown: Arc<AtomicBool>,
) {
    while let Ok(request) = requests.recv() {
        if shutdown.load(Ordering::Acquire) {
            return;
        }
        let transaction_id = request.transaction_id;
        let result = read_request(&mut pack, request, &gate, &shutdown);
        if completions
            .send(ReadCompletion {
                transaction_id,
                result,
            })
            .is_err()
        {
            return;
        }
    }
}

fn read_request(
    pack: &mut RegionPack,
    request: ReadRequest,
    gate: &IoGate,
    shutdown: &AtomicBool,
) -> std::result::Result<(Vec<RegionUpload>, IoMetrics), ReadFailure> {
    let worker_queue_ms = request.queued_at.elapsed().as_secs_f64() * 1_000.0;
    let gate_start = Instant::now();
    if let Some(fence) = request.gate_fence {
        let (state, wake) = &*gate.state;
        let mut state = state.lock().expect("I/O gate poisoned");
        while state.completed < fence && !shutdown.load(Ordering::Acquire) {
            state = wake.wait(state).expect("I/O gate poisoned");
        }
        if shutdown.load(Ordering::Acquire) {
            return Err(ReadFailure {
                message: "cooked region worker stopped".into(),
                metrics: IoMetrics::default(),
            });
        }
    }
    let gate_ms = gate_start.elapsed().as_secs_f64() * 1_000.0;

    let mut uploads = Vec::with_capacity(request.assignments.len());
    let mut metrics = IoMetrics {
        worker_queue_ms,
        gate_ms,
        ..IoMetrics::default()
    };
    for assignment in request.assignments {
        metrics.chunk_count += 1;
        metrics.seek_count += 1;
        metrics.payload_bytes += u64::from(region_format::REGION_BYTES);
        let attempt_start = Instant::now();
        let region = pack
            .read_region(assignment.region_id)
            .map_err(|error| ReadFailure {
                message: format!("{error:#}"),
                metrics: IoMetrics {
                    verify_ms: metrics.verify_ms + attempt_start.elapsed().as_secs_f64() * 1_000.0,
                    ..metrics.clone()
                },
            })?;
        metrics.read_ms += region.read_ms;
        metrics.verify_ms += region.verify_ms;
        uploads.push(RegionUpload {
            slot: assignment.slot,
            records: region.records,
        });
    }
    Ok((uploads, metrics))
}
