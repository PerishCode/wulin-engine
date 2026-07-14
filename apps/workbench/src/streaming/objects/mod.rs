mod worker;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::async_resident::{
    AsyncPlanCounts, AsyncReservationReport, AsyncTransactionReport, ObjectSourceNamespace,
};
use crate::resident::RegionUpload;
use crate::world::RegionCoord;

use self::worker::{IoGate, PackWorker, ReadCompletion, ReadRequest};

pub const COOKED_OBJECT_REVISION: &str = "cooked-canonical-object-v1";

#[derive(Clone, Copy)]
pub struct ObjectPackSource {
    pub source_namespace: ObjectSourceNamespace,
    pub stable_seed_namespace: ObjectSourceNamespace,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectIoMetrics {
    pub chunk_count: usize,
    pub payload_bytes: u64,
    pub seek_count: usize,
    pub worker_queue_ms: f64,
    pub gate_ms: f64,
    pub read_ms: f64,
    pub verify_ms: f64,
    pub total_ms: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectScheduleReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub global_config: crate::address::GlobalRegionConfig,
    pub source_namespace: ObjectSourceNamespace,
    pub stable_seed_namespace: ObjectSourceNamespace,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub requested_global_regions: Vec<RegionCoord>,
    pub gate_fence: Option<u64>,
}

pub enum ObjectCompletion {
    Ready {
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        metrics: ObjectIoMetrics,
        active_page_checksums: Vec<[u8; 32]>,
    },
    Failed {
        transaction_id: u64,
        message: String,
        metrics: ObjectIoMetrics,
    },
}

#[derive(Default)]
pub struct CookedObjectStreamer {
    worker: Option<PackWorker>,
    pack: Option<PackState>,
    pending: Option<PendingObject>,
    last_completed: Option<Value>,
    last_failure: Option<Value>,
    gate: IoGate,
    armed_gate: Option<u64>,
    next_gate_fence: u64,
}

struct PackState {
    path: PathBuf,
    metadata: region_format::GlobalPackMetadata,
    regions: BTreeSet<RegionCoord>,
    checksums: std::collections::BTreeMap<RegionCoord, [u8; 32]>,
    source: ObjectPackSource,
}

struct PendingObject {
    transaction_id: u64,
    global_config: crate::address::GlobalRegionConfig,
    source: ObjectPackSource,
    counts: AsyncPlanCounts,
    requested_global_regions: Vec<RegionCoord>,
    active_page_checksums: Vec<[u8; 32]>,
    gate_fence: Option<u64>,
    stage: &'static str,
    io: Option<ObjectIoMetrics>,
}

impl CookedObjectStreamer {
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<Value> {
        ensure!(self.pending.is_none(), "object_stream_busy");
        ensure!(
            self.armed_gate.is_none(),
            "object I/O gate must be released before opening a pack"
        );
        let path = path.as_ref().to_path_buf();
        let pack = region_format::GlobalRegionPack::open(&path)?;
        let metadata = pack.metadata().clone();
        let source = ObjectPackSource {
            source_namespace: ObjectSourceNamespace::from_bytes(pack.source_namespace()),
            stable_seed_namespace: ObjectSourceNamespace::from_bytes(pack.stable_seed_namespace()),
        };
        let pack_regions = pack.regions().collect::<Vec<_>>();
        let regions = pack_regions
            .iter()
            .map(|region| RegionCoord::new(region.x, region.z))
            .collect();
        let checksums = pack_regions
            .into_iter()
            .map(|region| {
                let checksum = pack
                    .region_sha256(region)
                    .expect("pack region has no index checksum");
                (RegionCoord::new(region.x, region.z), checksum)
            })
            .collect();
        self.worker = Some(PackWorker::start(pack, self.gate.clone())?);
        self.pack = Some(PackState {
            path,
            metadata: metadata.clone(),
            regions,
            checksums,
            source,
        });
        self.last_failure = None;
        Ok(serde_json::to_value(metadata)?)
    }

    pub fn disable(&mut self) -> Result<bool> {
        ensure!(self.pending.is_none(), "object_stream_busy");
        ensure!(
            self.armed_gate.is_none(),
            "object I/O gate must be released before disabling the source"
        );
        let was_open = self.pack.take().is_some();
        self.worker = None;
        self.last_completed = None;
        self.last_failure = None;
        Ok(was_open)
    }

    pub fn source(&self) -> Option<ObjectPackSource> {
        self.pack.as_ref().map(|pack| pack.source)
    }

    pub fn schedule(
        &mut self,
        reservation: AsyncReservationReport,
    ) -> Result<ObjectScheduleReport> {
        ensure!(self.pending.is_none(), "object_stream_busy");
        let pack = self
            .pack
            .as_ref()
            .context("no cooked object pack is open")?;
        let global_config = reservation
            .global_config
            .context("cooked object source requires a global schedule")?;
        ensure!(
            reservation.object_source_namespace == Some(pack.source.source_namespace),
            "cooked object source namespace mismatch"
        );
        ensure!(
            reservation.object_stable_seed_namespace == Some(pack.source.stable_seed_namespace),
            "cooked object stable-seed namespace mismatch"
        );
        let requested_global_regions = reservation
            .assignments
            .iter()
            .map(|assignment| {
                let region = assignment
                    .global_region
                    .context("cooked object assignment has no signed region")?;
                ensure!(
                    pack.regions.contains(&region),
                    "signed object region ({},{}) is absent from the pack",
                    region.x,
                    region.z
                );
                Ok(region)
            })
            .collect::<Result<Vec<_>>>()?;
        let active_page_checksums = global_config
            .addressed_regions()?
            .into_iter()
            .map(|region| {
                pack.checksums
                    .get(&region.global_region)
                    .copied()
                    .with_context(|| {
                        format!(
                            "signed object region ({},{}) has no index checksum",
                            region.global_region.x, region.global_region.z
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let gate_fence = self.armed_gate;
        self.worker
            .as_ref()
            .context("cooked object worker is unavailable")?
            .send(ReadRequest {
                transaction_id: reservation.transaction_id,
                assignments: reservation.assignments,
                gate_fence,
                queued_at: std::time::Instant::now(),
            })?;
        let report = ObjectScheduleReport {
            revision: COOKED_OBJECT_REVISION,
            transaction_id: reservation.transaction_id,
            global_config,
            source_namespace: pack.source.source_namespace,
            stable_seed_namespace: pack.source.stable_seed_namespace,
            counts: reservation.counts,
            requested_global_regions: requested_global_regions.clone(),
            gate_fence,
        };
        self.pending = Some(PendingObject {
            transaction_id: reservation.transaction_id,
            global_config,
            source: pack.source,
            counts: reservation.counts,
            requested_global_regions,
            active_page_checksums,
            gate_fence,
            stage: "reading",
            io: None,
        });
        Ok(report)
    }

    pub fn poll_completion(&mut self) -> Option<ObjectCompletion> {
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
                    .expect("object completion has no request");
                assert_eq!(pending.transaction_id, transaction_id);
                pending.stage = "ready";
                pending.io = Some(metrics);
                Some(ObjectCompletion::Ready {
                    transaction_id,
                    uploads,
                    metrics,
                    active_page_checksums: pending.active_page_checksums.clone(),
                })
            }
            ReadCompletion::Failed {
                transaction_id,
                message,
                metrics,
            } => Some(ObjectCompletion::Failed {
                transaction_id,
                message,
                metrics,
            }),
        }
    }

    pub fn mark_submitted(&mut self, report: &AsyncTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .as_mut()
            .context("object submission has no request")?;
        ensure!(
            pending.transaction_id == report.transaction_id,
            "object submission transaction mismatch"
        );
        pending.stage = "copying";
        Ok(())
    }

    pub fn mark_completed(&mut self, report: &AsyncTransactionReport) -> Result<()> {
        let pending = self
            .pending
            .take()
            .context("object completion has no request")?;
        ensure!(
            pending.transaction_id == report.transaction_id,
            "object completion transaction mismatch"
        );
        self.last_completed = Some(json!({
            "revision": COOKED_OBJECT_REVISION,
            "transactionId": report.transaction_id,
            "globalConfig": pending.global_config,
            "sourceNamespace": pending.source.source_namespace,
            "stableSeedNamespace": pending.source.stable_seed_namespace,
            "counts": pending.counts,
            "requestedGlobalRegions": pending.requested_global_regions,
            "gateFence": pending.gate_fence,
            "io": pending.io,
            "gpu": report,
        }));
        self.last_failure = None;
        Ok(())
    }

    pub fn mark_failed(&mut self, transaction_id: u64, message: String, metrics: ObjectIoMetrics) {
        let pending = self.pending.take();
        self.last_failure = Some(json!({
            "transactionId": transaction_id,
            "message": message,
            "io": metrics,
            "requestedGlobalRegions": pending
                .as_ref()
                .map(|pending| &pending.requested_global_regions)
                .cloned()
                .unwrap_or_default(),
        }));
    }

    pub fn arm_gate(&mut self) -> Result<u64> {
        ensure!(
            self.pending.is_none() && self.armed_gate.is_none(),
            "object I/O gate or request is already active"
        );
        ensure!(self.worker.is_some(), "no cooked object pack is open");
        let value = self.next_gate_fence.max(self.gate.completed() + 1);
        self.next_gate_fence = value + 1;
        self.armed_gate = Some(value);
        Ok(value)
    }

    pub fn release_gate(&mut self) -> Result<u64> {
        let value = self.armed_gate.context("object I/O gate is not armed")?;
        self.gate.signal(value);
        self.armed_gate = None;
        Ok(value)
    }

    pub fn status_json(&self) -> Option<Value> {
        let pack = self.pack.as_ref()?;
        let pending = self.pending.as_ref().map(|pending| {
            json!({
                "transactionId": pending.transaction_id,
                "globalConfig": pending.global_config,
                "sourceNamespace": pending.source.source_namespace,
                "stableSeedNamespace": pending.source.stable_seed_namespace,
                "counts": pending.counts,
                "requestedGlobalRegions": pending.requested_global_regions,
                "stage": pending.stage,
                "gateFence": pending.gate_fence,
                "io": pending.io,
            })
        });
        Some(json!({
            "revision": COOKED_OBJECT_REVISION,
            "pack": {
                "path": pack.path,
                "metadata": pack.metadata,
                "indexReadBytes": u64::from(region_format::GLOBAL_HEADER_BYTES)
                    + pack.metadata.index_bytes,
                "payloadBytesReadAtOpen": 0,
            },
            "pending": pending,
            "lastCompleted": self.last_completed,
            "lastFailure": self.last_failure,
            "ioGate": { "armed": self.armed_gate, "completed": self.gate.completed() },
            "workerCount": 1,
            "requestCapacity": 1,
            "completionCapacity": 1,
            "inFlightCapacity": 1,
        }))
    }

    pub fn owns(&self, transaction_id: u64) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|pending| pending.transaction_id == transaction_id)
    }
}
