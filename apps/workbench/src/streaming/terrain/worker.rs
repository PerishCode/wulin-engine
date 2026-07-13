use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result, bail};
use terrain_format::TerrainPack;

use super::{TerrainAssignment, TerrainIoMetrics, TerrainUpload};

#[derive(Clone, Default)]
pub(super) struct IoGate {
    state: Arc<(Mutex<u64>, Condvar)>,
}

pub(super) struct PackWorker {
    requests: Option<SyncSender<ReadRequest>>,
    completions: Receiver<ReadCompletion>,
    thread: Option<JoinHandle<()>>,
}

pub(super) struct ReadRequest {
    pub transaction_id: u64,
    pub assignments: Vec<TerrainAssignment>,
    pub gate_fence: Option<u64>,
}

pub(super) enum ReadCompletion {
    Ready {
        transaction_id: u64,
        uploads: Vec<TerrainUpload>,
        metrics: TerrainIoMetrics,
    },
    Failed {
        transaction_id: u64,
        message: String,
        metrics: TerrainIoMetrics,
    },
}

impl IoGate {
    pub fn completed(&self) -> u64 {
        *self.state.0.lock().expect("terrain I/O gate poisoned")
    }

    pub fn signal(&self, value: u64) {
        let (lock, wake) = &*self.state;
        let mut completed = lock.lock().expect("terrain I/O gate poisoned");
        *completed = (*completed).max(value);
        wake.notify_all();
    }

    fn wait(&self, value: u64) {
        let (lock, wake) = &*self.state;
        let mut completed = lock.lock().expect("terrain I/O gate poisoned");
        while *completed < value {
            completed = wake.wait(completed).expect("terrain I/O gate poisoned");
        }
    }
}

impl PackWorker {
    pub fn start(pack: TerrainPack, gate: IoGate) -> Result<Self> {
        let (request_tx, request_rx) = sync_channel::<ReadRequest>(1);
        let (completion_tx, completion_rx) = sync_channel::<ReadCompletion>(1);
        let thread = thread::Builder::new()
            .name("terrain-pack-io".into())
            .spawn(move || worker_loop(pack, gate, request_rx, completion_tx))
            .context("failed to start terrain pack worker")?;
        Ok(Self {
            requests: Some(request_tx),
            completions: completion_rx,
            thread: Some(thread),
        })
    }

    pub fn send(&self, request: ReadRequest) -> Result<()> {
        self.requests
            .as_ref()
            .context("terrain pack worker is stopped")?
            .try_send(request)
            .map_err(|error| anyhow::anyhow!("terrain pack request queue is unavailable: {error}"))
    }

    pub fn try_recv(&self) -> Option<ReadCompletion> {
        self.completions.try_recv().ok()
    }
}

impl Drop for PackWorker {
    fn drop(&mut self) {
        self.requests.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn worker_loop(
    mut pack: TerrainPack,
    gate: IoGate,
    requests: Receiver<ReadRequest>,
    completions: SyncSender<ReadCompletion>,
) {
    while let Ok(request) = requests.recv() {
        if let Some(value) = request.gate_fence {
            gate.wait(value);
        }
        let transaction_id = request.transaction_id;
        let completion = match read_request(&mut pack, request) {
            Ok((uploads, metrics)) => ReadCompletion::Ready {
                transaction_id,
                uploads,
                metrics,
            },
            Err((message, metrics)) => ReadCompletion::Failed {
                transaction_id,
                message,
                metrics,
            },
        };
        if completions.send(completion).is_err() {
            break;
        }
    }
}

fn read_request(
    pack: &mut TerrainPack,
    request: ReadRequest,
) -> std::result::Result<(Vec<TerrainUpload>, TerrainIoMetrics), (String, TerrainIoMetrics)> {
    let start = std::time::Instant::now();
    let mut metrics = TerrainIoMetrics::default();
    let mut uploads = Vec::with_capacity(request.assignments.len());
    for assignment in request.assignments {
        let read = match pack.read_region(assignment.region_id) {
            Ok(read) => read,
            Err(error) => {
                metrics.total_ms = start.elapsed().as_secs_f64() * 1_000.0;
                return Err((format!("{error:#}"), metrics));
            }
        };
        metrics.payload_bytes += u64::from(read.payload_bytes);
        metrics.read_ms += read.read_ms;
        metrics.verify_ms += read.verify_ms;
        uploads.push(TerrainUpload {
            slot: assignment.slot,
            region_id: assignment.region_id,
            global_region: assignment.global_region,
            payload: read.payload,
            tile: read.tile,
            sha256: read.sha256,
        });
    }
    metrics.total_ms = start.elapsed().as_secs_f64() * 1_000.0;
    if uploads.len() > 25 {
        return Err((
            "terrain worker exceeded active upload capacity".into(),
            metrics,
        ));
    }
    Ok((uploads, metrics))
}

pub(super) fn ensure_gate_advance(completed: u64, value: u64) -> Result<()> {
    if value <= completed {
        bail!("terrain I/O gate fence must advance")
    }
    Ok(())
}
