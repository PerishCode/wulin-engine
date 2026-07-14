use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use anyhow::{Context, Result, bail, ensure};

use crate::async_resident::RegionAssignment;
use crate::resident::{REGION_INSTANCE_BYTES, RegionUpload};

use super::ObjectIoMetrics;

#[derive(Clone, Default)]
pub(super) struct IoGate {
    state: Arc<(Mutex<u64>, Condvar)>,
}

pub(super) struct PackWorker {
    requests: Option<SyncSender<ReadRequest>>,
    completions: Receiver<ReadCompletion>,
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

pub(super) enum ReadCompletion {
    Ready {
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        metrics: ObjectIoMetrics,
    },
    Failed {
        transaction_id: u64,
        message: String,
        metrics: ObjectIoMetrics,
    },
}

impl IoGate {
    pub fn completed(&self) -> u64 {
        *self.state.0.lock().expect("object I/O gate poisoned")
    }

    pub fn signal(&self, value: u64) {
        let (lock, wake) = &*self.state;
        let mut completed = lock.lock().expect("object I/O gate poisoned");
        *completed = (*completed).max(value);
        wake.notify_all();
    }

    fn wait(&self, value: u64, shutdown: &AtomicBool) -> Result<()> {
        let (lock, wake) = &*self.state;
        let mut completed = lock.lock().expect("object I/O gate poisoned");
        while *completed < value && !shutdown.load(Ordering::Acquire) {
            completed = wake.wait(completed).expect("object I/O gate poisoned");
        }
        ensure!(
            !shutdown.load(Ordering::Acquire),
            "cooked object worker stopped"
        );
        Ok(())
    }
}

impl PackWorker {
    pub fn start(pack: region_format::GlobalRegionPack, gate: IoGate) -> Result<Self> {
        let (request_tx, request_rx) = sync_channel::<ReadRequest>(1);
        let (completion_tx, completion_rx) = sync_channel::<ReadCompletion>(1);
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker_gate = gate.clone();
        let thread = thread::Builder::new()
            .name("canonical-object-pack-io".into())
            .spawn(move || {
                worker_loop(
                    pack,
                    worker_gate,
                    worker_shutdown,
                    request_rx,
                    completion_tx,
                )
            })
            .context("failed to start cooked object worker")?;
        Ok(Self {
            requests: Some(request_tx),
            completions: completion_rx,
            shutdown,
            gate,
            thread: Some(thread),
        })
    }

    pub fn send(&self, request: ReadRequest) -> Result<()> {
        match self
            .requests
            .as_ref()
            .context("cooked object worker is stopped")?
            .try_send(request)
        {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => bail!("object_stream_busy"),
            Err(TrySendError::Disconnected(_)) => bail!("cooked object worker disconnected"),
        }
    }

    pub fn try_recv(&self) -> Option<ReadCompletion> {
        self.completions.try_recv().ok()
    }
}

impl Drop for PackWorker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.gate.state.1.notify_all();
        self.requests.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn worker_loop(
    mut pack: region_format::GlobalRegionPack,
    gate: IoGate,
    shutdown: Arc<AtomicBool>,
    requests: Receiver<ReadRequest>,
    completions: SyncSender<ReadCompletion>,
) {
    while let Ok(request) = requests.recv() {
        if shutdown.load(Ordering::Acquire) {
            return;
        }
        let transaction_id = request.transaction_id;
        let completion = match read_request(&mut pack, request, &gate, &shutdown) {
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
            return;
        }
    }
}

fn read_request(
    pack: &mut region_format::GlobalRegionPack,
    request: ReadRequest,
    gate: &IoGate,
    shutdown: &AtomicBool,
) -> std::result::Result<(Vec<RegionUpload>, ObjectIoMetrics), (String, ObjectIoMetrics)> {
    let explicit_local_ids =
        pack.metadata().payload_schema == region_format::GLOBAL_IDENTITY_PAYLOAD_SCHEMA;
    let started = Instant::now();
    let mut metrics = ObjectIoMetrics {
        worker_queue_ms: request.queued_at.elapsed().as_secs_f64() * 1_000.0,
        ..ObjectIoMetrics::default()
    };
    let gate_started = Instant::now();
    if let Some(fence) = request.gate_fence
        && let Err(error) = gate.wait(fence, shutdown)
    {
        metrics.gate_ms = gate_started.elapsed().as_secs_f64() * 1_000.0;
        metrics.total_ms = started.elapsed().as_secs_f64() * 1_000.0;
        return Err((error.to_string(), metrics));
    }
    metrics.gate_ms = gate_started.elapsed().as_secs_f64() * 1_000.0;

    let mut uploads = Vec::with_capacity(request.assignments.len());
    for assignment in request.assignments {
        let Some(global) = assignment.global_region else {
            metrics.total_ms = started.elapsed().as_secs_f64() * 1_000.0;
            return Err((
                "cooked object assignment has no signed region".into(),
                metrics,
            ));
        };
        metrics.chunk_count += 1;
        metrics.seek_count += 1;
        let read = match pack.read_region(region_format::GlobalRegion::new(global.x, global.z)) {
            Ok(read) => read,
            Err(error) => {
                metrics.total_ms = started.elapsed().as_secs_f64() * 1_000.0;
                return Err((format!("{error:#}"), metrics));
            }
        };
        metrics.payload_bytes += u64::from(read.payload_bytes);
        metrics.record_bytes += u64::from(region_format::REGION_BYTES);
        if explicit_local_ids {
            metrics.identity_bytes += u64::from(region_format::IDENTITY_PLANE_BYTES);
        }
        metrics.read_ms += read.read_ms;
        metrics.verify_ms += read.verify_ms;
        if assignment.stable_seed != Some(read.stable_seed) {
            metrics.total_ms = started.elapsed().as_secs_f64() * 1_000.0;
            return Err((
                format!(
                    "signed object region ({},{}) stable seed does not match its reservation",
                    global.x, global.z
                ),
                metrics,
            ));
        }
        uploads.push(RegionUpload {
            slot: assignment.slot,
            records: read.records,
            local_ids: explicit_local_ids.then_some(read.local_ids),
        });
    }
    metrics.total_ms = started.elapsed().as_secs_f64() * 1_000.0;
    if uploads.len() > 25 || uploads.len() * REGION_INSTANCE_BYTES > 25 * REGION_INSTANCE_BYTES {
        return Err((
            "cooked object worker exceeded active upload capacity".into(),
            metrics,
        ));
    }
    Ok((uploads, metrics))
}
