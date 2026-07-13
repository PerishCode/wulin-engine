mod worker;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_region: Option<crate::world::RegionCoord>,
}

pub struct TerrainUpload {
    pub slot: u32,
    pub region_id: u32,
    pub global_region: Option<crate::world::RegionCoord>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_config: Option<GlobalTerrainConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_namespace: Option<TerrainSourceNamespace>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_config: Option<GlobalTerrainConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_namespace: Option<TerrainSourceNamespace>,
    pub requested_region_ids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_global_regions: Option<Vec<crate::world::RegionCoord>>,
    pub gate_fence: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainTransactionReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_config: Option<GlobalTerrainConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_namespace: Option<TerrainSourceNamespace>,
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
    metadata: Value,
    addressing: PackAddressing,
}

#[derive(Clone)]
pub(super) enum PackAddressing {
    LocalAlias {
        region_ids: BTreeSet<u32>,
    },
    SignedGlobal {
        regions: BTreeSet<crate::world::RegionCoord>,
        source_namespace: TerrainSourceNamespace,
    },
}

pub(super) struct PackDescriptor {
    pub metadata: Value,
    pub addressing: PackAddressing,
}

struct PendingTerrain {
    transaction_id: u64,
    requested_region_ids: Vec<u32>,
    requested_global_regions: Option<Vec<crate::world::RegionCoord>>,
    global_config: Option<GlobalTerrainConfig>,
    source_namespace: Option<TerrainSourceNamespace>,
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
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<Value> {
        ensure!(self.pending.is_none(), "terrain_stream_busy");
        let path = path.as_ref().to_path_buf();
        let (pack, descriptor) = PackFile::open(&path)?;
        let metadata = descriptor.metadata.clone();
        self.worker = Some(PackWorker::start(pack, self.gate.clone())?);
        self.pack = Some(PackState {
            path,
            metadata: metadata.clone(),
            addressing: descriptor.addressing,
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
            global_config: reservation.global_config,
            source_namespace: reservation.source_namespace,
            io_gate_fence,
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
            if let Some(regions) = pending.requested_global_regions {
                failure["requestedGlobalRegions"] = json!(regions);
            }
            if let Some(namespace) = pending.source_namespace {
                failure["sourceNamespace"] = json!(namespace);
            }
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

    pub fn status_json(&self) -> Value {
        let pack = self.pack.as_ref().map(|pack| {
            let index_bytes = pack
                .metadata
                .get("indexBytes")
                .and_then(Value::as_u64)
                .expect("terrain pack metadata omitted index bytes");
            json!({
                "path": pack.path,
                "metadata": pack.metadata,
                "indexReadBytes": u64::from(terrain_format::HEADER_BYTES) + index_bytes,
            })
        });
        let pending = self.pending.as_ref().map(|pending| {
            let mut value = json!({
                "transactionId": pending.transaction_id,
                "requestedRegionIds": pending.requested_region_ids,
                "globalConfig": pending.global_config,
                "stage": pending.stage,
                "ioGateFence": pending.io_gate_fence,
                "io": pending.io,
            });
            if let Some(regions) = &pending.requested_global_regions {
                value["requestedGlobalRegions"] = json!(regions);
            }
            if let Some(namespace) = pending.source_namespace {
                value["sourceNamespace"] = json!(namespace);
            }
            value
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

    pub fn source_namespace(&self) -> Result<Option<TerrainSourceNamespace>> {
        let pack = self.pack.as_ref().context("no terrain pack is open")?;
        Ok(match &pack.addressing {
            PackAddressing::LocalAlias { .. } => None,
            PackAddressing::SignedGlobal {
                source_namespace, ..
            } => Some(*source_namespace),
        })
    }

    pub fn ensure_local_source(&self) -> Result<()> {
        ensure!(
            self.source_namespace()?.is_none(),
            "signed terrain pack requires a global schedule"
        );
        Ok(())
    }
}

impl PackState {
    fn preflight(
        &self,
        reservation: &TerrainReservationReport,
    ) -> Result<Option<Vec<crate::world::RegionCoord>>> {
        match &self.addressing {
            PackAddressing::LocalAlias { region_ids } => {
                ensure!(
                    reservation.source_namespace.is_none(),
                    "local terrain pack received a canonical source namespace"
                );
                for assignment in &reservation.assignments {
                    ensure!(
                        region_ids.contains(&assignment.region_id),
                        "terrain region {} is absent from the pack",
                        assignment.region_id
                    );
                }
                Ok(None)
            }
            PackAddressing::SignedGlobal {
                regions,
                source_namespace,
            } => {
                ensure!(
                    reservation.source_namespace == Some(*source_namespace),
                    "signed terrain source namespace mismatch"
                );
                let config = reservation
                    .global_config
                    .context("signed terrain pack requires a global schedule")?;
                for addressed in config.addressed_regions()? {
                    ensure!(
                        regions.contains(&addressed.global_region),
                        "signed terrain region ({},{}) is absent from the pack",
                        addressed.global_region.x,
                        addressed.global_region.z
                    );
                }
                Ok(Some(
                    reservation
                        .assignments
                        .iter()
                        .map(|assignment| {
                            assignment
                                .global_region
                                .context("canonical terrain assignment has no global region")
                        })
                        .collect::<Result<Vec<_>>>()?,
                ))
            }
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
