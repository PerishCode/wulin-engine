use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result, bail, ensure};
use terrain_format::{GlobalRegion, GlobalTerrainPack, TerrainPack, TerrainTile};

use super::{
    PackAddressing, PackDescriptor, TerrainAssignment, TerrainIoMetrics, TerrainSourceNamespace,
    TerrainUpload,
};

#[derive(Clone, Default)]
pub(super) struct IoGate {
    state: Arc<(Mutex<u64>, Condvar)>,
}

pub(super) struct PackWorker {
    requests: Option<SyncSender<ReadRequest>>,
    completions: Receiver<ReadCompletion>,
    thread: Option<JoinHandle<()>>,
}

pub(super) enum PackFile {
    Local(TerrainPack),
    Signed(GlobalTerrainPack),
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
    pub fn start(pack: PackFile, gate: IoGate) -> Result<Self> {
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

impl PackFile {
    pub fn open(path: impl AsRef<Path>) -> Result<(Self, PackDescriptor)> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .with_context(|| format!("failed to open terrain pack {}", path.display()))?;
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .context("terrain pack magic is truncated")?;
        if magic == terrain_format::MAGIC {
            let pack = TerrainPack::open(path)?;
            let descriptor = PackDescriptor {
                metadata: serde_json::to_value(pack.metadata())?,
                addressing: PackAddressing::LocalAlias {
                    region_ids: pack.region_ids().collect(),
                },
            };
            Ok((Self::Local(pack), descriptor))
        } else if magic == terrain_format::GLOBAL_MAGIC {
            let pack = GlobalTerrainPack::open(path)?;
            let descriptor = PackDescriptor {
                metadata: serde_json::to_value(pack.metadata())?,
                addressing: PackAddressing::SignedGlobal {
                    regions: pack
                        .regions()
                        .map(|region| crate::world::RegionCoord::new(region.x, region.z))
                        .collect(),
                    source_namespace: TerrainSourceNamespace(pack.source_namespace()),
                },
            };
            Ok((Self::Signed(pack), descriptor))
        } else {
            bail!("terrain pack magic is invalid")
        }
    }

    fn read(&mut self, assignment: TerrainAssignment) -> Result<(TerrainUpload, u32, f64, f64)> {
        match self {
            Self::Local(pack) => {
                let read = pack.read_region(assignment.region_id)?;
                Ok((
                    TerrainUpload {
                        slot: assignment.slot,
                        region_id: assignment.region_id,
                        global_region: assignment.global_region,
                        payload: read.payload,
                        tile: read.tile,
                        sha256: read.sha256,
                    },
                    read.payload_bytes,
                    read.read_ms,
                    read.verify_ms,
                ))
            }
            Self::Signed(pack) => {
                let global = assignment
                    .global_region
                    .context("signed terrain read has no global region")?;
                let read = pack.read_region(GlobalRegion::new(global.x, global.z))?;
                ensure!(
                    read.tile.region == GlobalRegion::new(global.x, global.z),
                    "signed terrain read identity mismatch"
                );
                Ok((
                    TerrainUpload {
                        slot: assignment.slot,
                        region_id: assignment.region_id,
                        global_region: Some(global),
                        payload: read.payload,
                        tile: TerrainTile {
                            region_id: assignment.region_id,
                            heights: read.tile.heights,
                            materials: read.tile.materials,
                        },
                        sha256: read.sha256,
                    },
                    read.payload_bytes,
                    read.read_ms,
                    read.verify_ms,
                ))
            }
        }
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
    mut pack: PackFile,
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
    pack: &mut PackFile,
    request: ReadRequest,
) -> std::result::Result<(Vec<TerrainUpload>, TerrainIoMetrics), (String, TerrainIoMetrics)> {
    let start = std::time::Instant::now();
    let mut metrics = TerrainIoMetrics::default();
    let mut uploads = Vec::with_capacity(request.assignments.len());
    for assignment in request.assignments {
        let (upload, payload_bytes, read_ms, verify_ms) = match pack.read(assignment) {
            Ok(read) => read,
            Err(error) => {
                metrics.total_ms = start.elapsed().as_secs_f64() * 1_000.0;
                return Err((format!("{error:#}"), metrics));
            }
        };
        metrics.payload_bytes += u64::from(payload_bytes);
        metrics.read_ms += read_ms;
        metrics.verify_ms += verify_ms;
        uploads.push(upload);
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
