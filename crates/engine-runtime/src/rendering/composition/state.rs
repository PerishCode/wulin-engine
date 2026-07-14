use anyhow::Result;

use crate::objects::{ObjectCompletion, ObjectIoMetrics};
use crate::resident::RegionUpload;

use super::super::renderer::Renderer;
use super::*;

impl Default for CompositionCoordinator {
    fn default() -> Self {
        Self {
            enabled: false,
            next_token: 1,
            publication_count: 0,
            pending: None,
            published: None,
            last_failure: None,
            traversal: traversal::CameraTraversal::default(),
        }
    }
}

impl CompositionCoordinator {
    pub(in crate::rendering) fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn begin(&mut self, input: PendingPairInput) -> u64 {
        let token = self.next_token;
        self.next_token += 1;
        self.pending = Some(PendingPair {
            token,
            config: input.config,
            global_config: input.global_config,
            terrain_source_namespace: input.terrain_source_namespace,
            object_source_namespace: input.object_source_namespace,
            object_stable_seed_namespace: input.object_stable_seed_namespace,
            terrain_transaction_id: input.terrain_transaction_id,
            instance_transaction_id: input.instance_transaction_id,
            terrain: HalfState::InFlight,
            instance: HalfState::InFlight,
            failure: None,
            purpose: input.purpose,
            started_at: Instant::now(),
        });
        token
    }

    pub(in crate::rendering) fn fail_half(
        &mut self,
        terrain: bool,
        transaction_id: u64,
        message: String,
    ) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        let expected = if terrain {
            pending.terrain_transaction_id
        } else {
            pending.instance_transaction_id
        };
        if expected != transaction_id {
            return;
        }
        if terrain {
            pending.terrain = HalfState::Failed;
        } else {
            pending.instance = HalfState::Failed;
        }
        pending.failure = Some(message);
    }

    pub(super) fn status_json(&self) -> Value {
        let pending = self.pending.as_ref().map(|value| {
            let mut pending = json!({
                "token": value.token,
                "config": value.config,
                "terrainTransactionId": value.terrain_transaction_id,
                "instanceTransactionId": value.instance_transaction_id,
                "terrainStage": value.terrain,
                "instanceStage": value.instance,
                "failure": value.failure,
                "cameraDriven": value.purpose.camera_driven(),
                "pendingMs": value.started_at.elapsed().as_secs_f64() * 1_000.0,
            });
            if value.purpose.prefetch() {
                pending["prefetch"] = json!(true);
            }
            pending["globalConfig"] = json!(value.global_config);
            pending["terrainSourceNamespace"] = json!(value.terrain_source_namespace);
            pending["objectSourceNamespace"] = json!(value.object_source_namespace);
            pending["objectStableSeedNamespace"] = json!(value.object_stable_seed_namespace);
            pending
        });
        json!({
            "revision": COMPOSITION_REVISION,
            "enabled": self.enabled,
            "nextToken": self.next_token,
            "pending": pending,
            "published": self.published,
            "lastFailure": self.last_failure,
            "traversal": self.traversal.status_json(),
        })
    }
}

pub(super) fn rollback_failed_pair(renderer: &mut Renderer) {
    let Some(pending) = renderer.composition.pending.as_mut() else {
        return;
    };
    if pending.instance == HalfState::Staged {
        if let Some(report) = renderer.async_resident_renderer.discard_staged()
            && renderer.cooked_object_streamer.owns(report.transaction_id)
        {
            let _ = renderer.cooked_object_streamer.mark_completed(&report);
        }
        pending.instance = HalfState::Discarded;
    }
    if pending.terrain == HalfState::Staged {
        if let Some(report) = renderer.terrain_renderer.discard_staged() {
            let _ = renderer.terrain_streamer.mark_completed(&report);
        }
        pending.terrain = HalfState::Discarded;
    }
    if pending.instance == HalfState::InFlight || pending.terrain == HalfState::InFlight {
        return;
    }
    let pending = renderer
        .composition
        .pending
        .take()
        .expect("failed composition pair disappeared");
    if pending.purpose == PairPurpose::Traversal {
        renderer.composition.traversal.mark_failed(
            pending.config,
            pending.global_config,
            pending
                .failure
                .clone()
                .unwrap_or_else(|| "composition pair failed".into()),
        );
    } else if pending.purpose.prefetch() {
        renderer.composition.traversal.mark_prefetch_failed(
            TraversalTarget {
                config: pending.config,
                global_config: pending.global_config,
            },
            pending
                .failure
                .clone()
                .unwrap_or_else(|| "composition prefetch failed".into()),
        );
    }
    renderer.composition.last_failure = Some(json!({
        "token": pending.token,
        "config": pending.config,
        "terrainTransactionId": pending.terrain_transaction_id,
        "instanceTransactionId": pending.instance_transaction_id,
        "terrainStage": pending.terrain,
        "instanceStage": pending.instance,
        "message": pending.failure,
        "rollbackMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
    }));
}

impl Renderer {
    pub fn open_cooked_object_pack(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<serde_json::Value> {
        anyhow::ensure!(!self.composition.has_pending(), "composition_pair_busy");
        self.cooked_object_streamer.open(path)
    }

    pub fn arm_object_io_gate(&mut self) -> Result<u64> {
        self.cooked_object_streamer.arm_gate()
    }

    pub fn release_object_io_gate(&mut self) -> Result<u64> {
        self.cooked_object_streamer.release_gate()
    }

    pub(in crate::rendering) unsafe fn poll_cooked_object_completion(&mut self) -> Result<()> {
        let Some(completion) = self.cooked_object_streamer.poll_completion() else {
            return Ok(());
        };
        match completion {
            ObjectCompletion::Ready {
                transaction_id,
                uploads,
                metrics,
                active_page_checksums,
            } => unsafe {
                self.submit_object_completion(
                    transaction_id,
                    uploads,
                    metrics,
                    active_page_checksums,
                )
            },
            ObjectCompletion::Failed {
                transaction_id,
                message,
                metrics,
            } => {
                let cancellation = self
                    .async_resident_renderer
                    .cancel_reservation(transaction_id);
                let message = match cancellation {
                    Ok(()) => message,
                    Err(error) => {
                        format!("{message}; object reservation cancellation failed: {error}")
                    }
                };
                self.cooked_object_streamer
                    .mark_failed(transaction_id, message.clone(), metrics);
                self.composition.fail_half(false, transaction_id, message);
                Ok(())
            }
        }
    }

    unsafe fn submit_object_completion(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        metrics: ObjectIoMetrics,
        active_page_checksums: Vec<[u8; 32]>,
    ) -> Result<()> {
        let release_fence = self.next_fence_value;
        self.next_fence_value += 1;
        match unsafe {
            self.async_resident_renderer.submit_canonical_cooked(
                transaction_id,
                uploads,
                metrics.total_ms,
                active_page_checksums,
                (&self.queue, &self.fence, release_fence),
            )
        } {
            Ok(report) => self.cooked_object_streamer.mark_submitted(&report),
            Err(error) => {
                let message = format!("object half failed to submit: {error:#}");
                self.cooked_object_streamer
                    .mark_failed(transaction_id, message.clone(), metrics);
                self.composition.fail_half(false, transaction_id, message);
                Ok(())
            }
        }
    }

    pub(in crate::rendering::composition) fn complete_cooked_object(
        &mut self,
        report: &crate::async_resident::AsyncTransactionReport,
    ) -> Result<()> {
        if self.cooked_object_streamer.owns(report.transaction_id) {
            self.cooked_object_streamer.mark_completed(report)?;
        }
        Ok(())
    }
}
