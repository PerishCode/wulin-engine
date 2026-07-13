mod worker;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use terrain_format::{PackMetadata, TerrainPack, TerrainTile};

use crate::load::LoadConfig;

use self::worker::{IoGate, PackWorker, ReadCompletion, ReadRequest, ensure_gate_advance};

pub const TERRAIN_STREAM_REVISION: &str = "terrain-stream-v1";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainAssignment {
    pub slot: u32,
    pub region_id: u32,
}

pub struct TerrainUpload {
    pub slot: u32,
    pub region_id: u32,
    pub payload: [u8; terrain_format::PAYLOAD_BYTES as usize],
    pub tile: TerrainTile,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainIoMetrics {
    pub payload_bytes: u64,
    pub read_ms: f64,
    pub verify_ms: f64,
    pub total_ms: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainPlanCounts {
    pub retained_region_count: usize,
    pub uploaded_region_count: usize,
    pub evicted_region_count: usize,
    pub protected_region_count: usize,
    pub resident_region_count: usize,
    pub free_region_count: usize,
    pub payload_bytes: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainReservationReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(flatten)]
    pub counts: TerrainPlanCounts,
    pub assignments: Vec<TerrainAssignment>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainScheduleReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    pub requested_region_ids: Vec<u32>,
    pub gate_fence: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainTransactionReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(flatten)]
    pub counts: TerrainPlanCounts,
    pub uploaded_sha256: String,
    pub direct_release_fence: u64,
    pub copy_fence: u64,
    pub copy_gate_fence: Option<u64>,
    pub io: TerrainIoMetrics,
    pub schedule_ms: f64,
    pub copy_gpu_ms: f64,
    pub copy_to_publication_ms: f64,
    pub pending_ms: f64,
}

pub enum TerrainCompletion {
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

#[derive(Clone)]
struct PackState {
    path: PathBuf,
    metadata: PackMetadata,
    region_ids: BTreeSet<u32>,
}

struct PendingTerrain {
    transaction_id: u64,
    requested_region_ids: Vec<u32>,
    io_gate_fence: Option<u64>,
    stage: &'static str,
    io: Option<TerrainIoMetrics>,
}

#[derive(Default)]
pub struct TerrainStreamer {
    worker: Option<PackWorker>,
    pack: Option<PackState>,
    pending: Option<PendingTerrain>,
    last_completed: Option<Value>,
    last_failure: Option<Value>,
    gate: IoGate,
    armed_gate: Option<u64>,
    next_gate_fence: u64,
}

impl TerrainStreamer {
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<PackMetadata> {
        ensure!(self.pending.is_none(), "terrain_stream_busy");
        let path = path.as_ref().to_path_buf();
        let pack = TerrainPack::open(&path)?;
        let metadata = pack.metadata().clone();
        let region_ids = pack.region_ids().collect();
        self.worker = Some(PackWorker::start(pack, self.gate.clone())?);
        self.pack = Some(PackState {
            path,
            metadata: metadata.clone(),
            region_ids,
        });
        self.last_failure = None;
        Ok(metadata)
    }

    pub fn schedule(
        &mut self,
        reservation: TerrainReservationReport,
    ) -> Result<TerrainScheduleReport> {
        ensure!(self.pending.is_none(), "terrain_stream_busy");
        let pack = self.pack.as_ref().context("no terrain pack is open")?;
        let requested_region_ids = reservation
            .assignments
            .iter()
            .map(|value| value.region_id)
            .collect::<Vec<_>>();
        for region_id in &requested_region_ids {
            ensure!(
                pack.region_ids.contains(region_id),
                "terrain region {region_id} is absent from the pack"
            );
        }
        let io_gate_fence = self.armed_gate;
        self.worker
            .as_ref()
            .context("terrain pack worker is unavailable")?
            .send(ReadRequest {
                transaction_id: reservation.transaction_id,
                assignments: reservation.assignments,
                gate_fence: io_gate_fence,
            })?;
        self.pending = Some(PendingTerrain {
            transaction_id: reservation.transaction_id,
            requested_region_ids: requested_region_ids.clone(),
            io_gate_fence,
            stage: "reading",
            io: None,
        });
        Ok(TerrainScheduleReport {
            revision: TERRAIN_STREAM_REVISION,
            transaction_id: reservation.transaction_id,
            config: reservation.config,
            requested_region_ids,
            gate_fence: io_gate_fence,
        })
    }

    pub fn poll_completion(&mut self) -> Option<TerrainCompletion> {
        let completion = self.worker.as_ref()?.try_recv()?;
        match completion {
            ReadCompletion::Ready {
                transaction_id,
                uploads,
                metrics,
            } => {
                let pending = self
                    .pending
                    .as_mut()
                    .expect("terrain completion has no request");
                assert_eq!(pending.transaction_id, transaction_id);
                pending.stage = "ready";
                pending.io = Some(metrics);
                Some(TerrainCompletion::Ready {
                    transaction_id,
                    uploads,
                    metrics,
                })
            }
            ReadCompletion::Failed {
                transaction_id,
                message,
                metrics,
            } => Some(TerrainCompletion::Failed {
                transaction_id,
                message,
                metrics,
            }),
        }
    }

    pub fn mark_submitted(&mut self, report: &mut TerrainTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .as_mut()
            .context("terrain submission has no request")?;
        ensure!(
            pending.transaction_id == report.transaction_id,
            "terrain submission transaction mismatch"
        );
        pending.stage = "copying";
        report.io = pending
            .io
            .context("terrain submission has no I/O metrics")?;
        Ok(())
    }

    pub fn mark_published(&mut self, report: &TerrainTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .take()
            .context("terrain publication has no request")?;
        ensure!(
            pending.transaction_id == report.transaction_id,
            "terrain publication transaction mismatch"
        );
        self.last_completed = Some(serde_json::to_value(report)?);
        self.last_failure = None;
        Ok(())
    }

    pub fn mark_failed(&mut self, transaction_id: u64, message: String, metrics: TerrainIoMetrics) {
        let pending = self.pending.take();
        self.last_failure = Some(json!({
            "transactionId": transaction_id,
            "message": message,
            "io": metrics,
            "requestedRegionIds": pending.map(|value| value.requested_region_ids).unwrap_or_default(),
        }));
    }

    pub fn arm_gate(&mut self) -> Result<u64> {
        ensure!(
            self.pending.is_none() && self.armed_gate.is_none(),
            "terrain I/O gate or request is already active"
        );
        let value = self.next_gate_fence.max(self.gate.completed() + 1);
        ensure_gate_advance(self.gate.completed(), value)?;
        self.next_gate_fence = value + 1;
        self.armed_gate = Some(value);
        Ok(value)
    }

    pub fn release_gate(&mut self) -> Result<u64> {
        let value = self.armed_gate.context("terrain I/O gate is not armed")?;
        self.gate.signal(value);
        self.armed_gate = None;
        Ok(value)
    }

    pub fn status_json(&self) -> Value {
        let pack = self.pack.as_ref().map(|pack| {
            json!({
                "path": pack.path,
                "metadata": pack.metadata,
                "indexReadBytes": u64::from(terrain_format::HEADER_BYTES) + pack.metadata.index_bytes,
            })
        });
        let pending = self.pending.as_ref().map(|pending| {
            json!({
                "transactionId": pending.transaction_id,
                "requestedRegionIds": pending.requested_region_ids,
                "stage": pending.stage,
                "ioGateFence": pending.io_gate_fence,
                "io": pending.io,
            })
        });
        json!({
            "revision": TERRAIN_STREAM_REVISION,
            "pack": pack,
            "pending": pending,
            "ioGate": { "armed": self.armed_gate, "completed": self.gate.completed() },
            "lastCompleted": self.last_completed,
            "lastFailure": self.last_failure,
            "queueCapacity": 1,
            "workerCount": usize::from(self.worker.is_some()),
        })
    }
}
