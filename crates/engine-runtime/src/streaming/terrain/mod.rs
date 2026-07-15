mod worker;

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use terrain_format::TerrainTile;

use crate::load::LoadConfig;

use self::worker::{
    IoGate, PackFile, PackWorker, ReadCompletion, ReadRequest, ensure_gate_advance,
};

pub(crate) use crate::address::AddressedRegion;
pub use crate::address::GlobalRegionConfig as GlobalTerrainConfig;

pub const TERRAIN_STREAM_REVISION: &str = "terrain-stream-v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TerrainSourceNamespace(pub(crate) [u8; 32]);

impl Serialize for TerrainSourceNamespace {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex(&self.0))
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainAssignment {
    pub slot: u32,
    pub region_id: u32,
    pub global_region: crate::region::RegionCoord,
}

pub struct TerrainUpload {
    pub slot: u32,
    pub region_id: u32,
    pub global_region: crate::region::RegionCoord,
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
    pub global_config: GlobalTerrainConfig,
    pub source_namespace: TerrainSourceNamespace,
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
    pub global_config: GlobalTerrainConfig,
    pub source_namespace: TerrainSourceNamespace,
    pub requested_region_ids: Vec<u32>,
    pub requested_global_regions: Vec<crate::region::RegionCoord>,
    pub gate_fence: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainTransactionReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    pub global_config: GlobalTerrainConfig,
    pub source_namespace: TerrainSourceNamespace,
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
    regions: BTreeSet<crate::region::RegionCoord>,
    source_namespace: TerrainSourceNamespace,
}

pub(super) struct PackDescriptor {
    pub metadata: Value,
    pub regions: BTreeSet<crate::region::RegionCoord>,
    pub source_namespace: TerrainSourceNamespace,
}

struct PendingTerrain {
    transaction_id: u64,
    requested_region_ids: Vec<u32>,
    requested_global_regions: Vec<crate::region::RegionCoord>,
    source_namespace: TerrainSourceNamespace,
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
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<Value> {
        ensure!(self.pending.is_none(), "terrain_stream_busy");
        let path = path.as_ref().to_path_buf();
        let (pack, descriptor) = PackFile::open(&path)?;
        let metadata = descriptor.metadata.clone();
        self.worker = Some(PackWorker::start(pack, self.gate.clone())?);
        self.pack = Some(PackState {
            regions: descriptor.regions,
            source_namespace: descriptor.source_namespace,
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
        let requested_global_regions = pack.preflight(&reservation)?;
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
            requested_global_regions: requested_global_regions.clone(),
            source_namespace: reservation.source_namespace,
            stage: "reading",
            io: None,
        });
        Ok(TerrainScheduleReport {
            revision: TERRAIN_STREAM_REVISION,
            transaction_id: reservation.transaction_id,
            config: reservation.config,
            global_config: reservation.global_config,
            source_namespace: reservation.source_namespace,
            requested_region_ids,
            requested_global_regions,
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

    pub fn mark_completed(&mut self, report: &TerrainTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .take()
            .context("terrain completion has no request")?;
        ensure!(
            pending.transaction_id == report.transaction_id,
            "terrain completion transaction mismatch"
        );
        self.last_completed = Some(serde_json::to_value(report)?);
        self.last_failure = None;
        Ok(())
    }

    pub fn mark_failed(&mut self, transaction_id: u64, message: String, metrics: TerrainIoMetrics) {
        let pending = self.pending.take();
        let mut failure = json!({
            "transactionId": transaction_id,
            "message": message,
            "io": metrics,
            "requestedRegionIds": pending.as_ref().map(|value| &value.requested_region_ids).cloned().unwrap_or_default(),
        });
        if let Some(pending) = pending {
            failure["requestedGlobalRegions"] = json!(pending.requested_global_regions);
            failure["sourceNamespace"] = json!(pending.source_namespace);
        }
        self.last_failure = Some(failure);
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

    pub fn source_namespace(&self) -> Result<TerrainSourceNamespace> {
        let pack = self.pack.as_ref().context("no terrain pack is open")?;
        Ok(pack.source_namespace)
    }
}

impl PackState {
    fn preflight(
        &self,
        reservation: &TerrainReservationReport,
    ) -> Result<Vec<crate::region::RegionCoord>> {
        ensure!(
            reservation.source_namespace == self.source_namespace,
            "signed terrain source namespace mismatch"
        );
        let config = reservation.global_config;
        for addressed in config.addressed_regions()? {
            ensure!(
                self.regions.contains(&addressed.global_region),
                "signed terrain region ({},{}) is absent from the pack",
                addressed.global_region.x,
                addressed.global_region.z
            );
        }
        Ok(reservation
            .assignments
            .iter()
            .map(|assignment| assignment.global_region)
            .collect())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
