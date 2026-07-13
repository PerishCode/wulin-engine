use std::collections::BTreeSet;
use std::path::Path;
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use anyhow::{Context, Result, bail, ensure};
use region_format::{PackMetadata, RegionPack};
use serde::Serialize;
use serde_json::{Value, json};

use crate::async_resident::{AsyncPlanCounts, AsyncReservationReport, AsyncTransactionReport};
use crate::load::LoadConfig;
use crate::resident::RegionUpload;

use self::worker::{IoGate, PackWorker, ReadRequest};

mod worker;

pub const COOKED_REVISION: &str = "cooked-region-v1";

pub struct CookedStreamer {
    worker: Option<PackWorker>,
    pack: Option<PackState>,
    pending: Option<PendingCook>,
    last_completed: Option<Value>,
    last_error: Option<Value>,
    gate: IoGate,
    armed_gate: Option<u64>,
    next_gate: u64,
}

#[derive(Clone)]
struct PackState {
    path: String,
    metadata: PackMetadata,
    region_ids: BTreeSet<u32>,
}

struct PendingCook {
    reservation: AsyncReservationReport,
    requested_region_ids: Vec<u32>,
    gate_fence: Option<u64>,
    started_at: Instant,
    stage: CookStage,
    io: Option<IoMetrics>,
    gpu: Option<AsyncTransactionReport>,
}

#[derive(Clone, Copy)]
enum CookStage {
    AwaitingIo,
    Prepared,
    Copying,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CookScheduleReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub requested_region_ids: Vec<u32>,
    pub gate_fence: Option<u64>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IoMetrics {
    chunk_count: usize,
    payload_bytes: u64,
    seek_count: usize,
    worker_queue_ms: f64,
    gate_ms: f64,
    read_ms: f64,
    verify_ms: f64,
}

pub(crate) enum CookCompletion {
    Ready {
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        preparation_ms: f64,
    },
    Failed {
        transaction_id: u64,
        message: String,
        metrics: IoMetrics,
    },
}

impl Default for CookedStreamer {
    fn default() -> Self {
        Self {
            worker: None,
            pack: None,
            pending: None,
            last_completed: None,
            last_error: None,
            gate: IoGate::default(),
            armed_gate: None,
            next_gate: 1,
        }
    }
}

impl CookedStreamer {
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<PackMetadata> {
        ensure!(self.pending.is_none(), "stream_busy");
        let path = path.as_ref();
        let pack = RegionPack::open(path)?;
        let state = PackState {
            path: path.display().to_string(),
            metadata: pack.metadata().clone(),
            region_ids: pack.region_ids().collect(),
        };
        self.worker = Some(PackWorker::start(pack, self.gate.clone())?);
        self.pack = Some(state.clone());
        self.last_completed = None;
        self.last_error = None;
        Ok(state.metadata)
    }

    pub fn schedule(&mut self, reservation: AsyncReservationReport) -> Result<CookScheduleReport> {
        ensure!(self.pending.is_none(), "stream_busy");
        let pack = self
            .pack
            .as_ref()
            .context("no cooked region pack is open")?;
        let requested_region_ids = reservation
            .assignments
            .iter()
            .map(|assignment| assignment.region_id)
            .collect::<Vec<_>>();
        for region_id in &requested_region_ids {
            ensure!(
                pack.region_ids.contains(region_id),
                "region {region_id} is absent from the cooked pack"
            );
        }
        let gate_fence = self.armed_gate;
        let request = ReadRequest {
            transaction_id: reservation.transaction_id,
            assignments: reservation.assignments.clone(),
            gate_fence,
            queued_at: Instant::now(),
        };
        self.worker
            .as_ref()
            .context("cooked region worker is unavailable")?
            .send(request)?;
        let report = CookScheduleReport {
            revision: COOKED_REVISION,
            transaction_id: reservation.transaction_id,
            config: reservation.config,
            counts: reservation.counts,
            requested_region_ids: requested_region_ids.clone(),
            gate_fence,
        };
        self.pending = Some(PendingCook {
            reservation,
            requested_region_ids,
            gate_fence,
            started_at: Instant::now(),
            stage: CookStage::AwaitingIo,
            io: None,
            gpu: None,
        });
        Ok(report)
    }

    pub fn poll_completion(&mut self) -> Option<CookCompletion> {
        let pending = self.pending.as_mut()?;
        if !matches!(pending.stage, CookStage::AwaitingIo) {
            return None;
        }
        let completion = match self.worker.as_ref()?.completion.try_recv() {
            Ok(completion) => completion,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                return Some(CookCompletion::Failed {
                    transaction_id: pending.reservation.transaction_id,
                    message: "cooked region worker disconnected".into(),
                    metrics: IoMetrics::default(),
                });
            }
        };
        if completion.transaction_id != pending.reservation.transaction_id {
            return Some(CookCompletion::Failed {
                transaction_id: pending.reservation.transaction_id,
                message: format!(
                    "worker completed transaction {} while {} was pending",
                    completion.transaction_id, pending.reservation.transaction_id
                ),
                metrics: IoMetrics::default(),
            });
        }
        match completion.result {
            Ok((uploads, metrics)) => {
                let preparation_ms = metrics.read_ms + metrics.verify_ms;
                pending.io = Some(metrics);
                pending.stage = CookStage::Prepared;
                Some(CookCompletion::Ready {
                    transaction_id: completion.transaction_id,
                    uploads,
                    preparation_ms,
                })
            }
            Err(failure) => {
                pending.io = Some(failure.metrics.clone());
                Some(CookCompletion::Failed {
                    transaction_id: completion.transaction_id,
                    message: failure.message,
                    metrics: failure.metrics,
                })
            }
        }
    }

    pub fn mark_submitted(&mut self, report: AsyncTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .as_mut()
            .context("no cooked request is pending")?;
        ensure!(
            pending.reservation.transaction_id == report.transaction_id,
            "cooked GPU transaction does not match its I/O request"
        );
        pending.gpu = Some(report);
        pending.stage = CookStage::Copying;
        Ok(())
    }

    pub fn mark_published(&mut self, report: &AsyncTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .take()
            .context("no cooked request is pending")?;
        ensure!(
            pending.reservation.transaction_id == report.transaction_id,
            "published transaction does not match cooked request"
        );
        let io = pending
            .io
            .context("published cooked request has no I/O metrics")?;
        self.last_completed = Some(json!({
            "revision": COOKED_REVISION,
            "transactionId": report.transaction_id,
            "config": report.config,
            "counts": pending.reservation.counts,
            "requestedRegionIds": pending.requested_region_ids,
            "gateFence": pending.gate_fence,
            "io": io,
            "gpu": report,
            "publicationMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
        }));
        Ok(())
    }

    pub fn mark_failed(&mut self, transaction_id: u64, message: String, metrics: IoMetrics) {
        let pending = self.pending.take();
        self.last_error = Some(json!({
            "transactionId": transaction_id,
            "message": message,
            "io": pending
                .as_ref()
                .and_then(|pending| pending.io.as_ref())
                .unwrap_or(&metrics),
        }));
    }

    pub fn arm_gate(&mut self) -> Result<u64> {
        if self.pending.is_some() || self.armed_gate.is_some() {
            bail!("I/O gate or cooked transaction is already active");
        }
        ensure!(self.worker.is_some(), "no cooked region pack is open");
        let value = self.next_gate;
        self.next_gate += 1;
        self.armed_gate = Some(value);
        Ok(value)
    }

    pub fn release_gate(&mut self) -> Result<u64> {
        let value = self.armed_gate.context("I/O gate is not armed")?;
        self.gate.signal(value);
        self.armed_gate = None;
        Ok(value)
    }

    pub fn status_json(&self) -> Value {
        let completed_gate = self.gate.completed();
        let pending = self.pending.as_ref().map(|pending| {
            let stage = match pending.stage {
                CookStage::AwaitingIo
                    if pending
                        .gate_fence
                        .is_some_and(|fence| completed_gate < fence) =>
                {
                    "io-gated"
                }
                CookStage::AwaitingIo => "reading",
                CookStage::Prepared => "prepared",
                CookStage::Copying => "copying",
            };
            json!({
                "stage": stage,
                "transactionId": pending.reservation.transaction_id,
                "config": pending.reservation.config,
                "counts": pending.reservation.counts,
                "requestedRegionIds": pending.requested_region_ids,
                "gateFence": pending.gate_fence,
                "payloadBytesRead": pending.io.as_ref().map(|io| io.payload_bytes).unwrap_or(0),
                "chunkCount": pending.io.as_ref().map(|io| io.chunk_count).unwrap_or(0),
                "pendingMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
            })
        });
        let pack = self.pack.as_ref().map(|pack| {
            json!({
                "path": pack.path,
                "metadata": pack.metadata,
                "indexReadBytes": u64::from(region_format::HEADER_BYTES) + pack.metadata.index_bytes,
                "payloadBytesReadAtOpen": 0,
            })
        });
        json!({
            "revision": COOKED_REVISION,
            "pack": pack,
            "workerCapacity": 1,
            "requestCapacity": 1,
            "completionCapacity": 1,
            "inFlightCapacity": 1,
            "pending": pending,
            "lastCompleted": self.last_completed,
            "lastError": self.last_error,
            "gate": {
                "armedFence": self.armed_gate,
                "completedFence": completed_gate,
            },
        })
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
}
