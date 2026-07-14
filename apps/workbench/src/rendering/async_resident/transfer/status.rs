use serde_json::{Value, json};

use crate::async_resident::{ASYNC_CACHE_CAPACITY, ASYNC_RESIDENT_REVISION};
use crate::load::LoadConfig;
use crate::resident::{REGION_IDENTITY_BYTES, REGION_INSTANCE_BYTES};

use super::AsyncTransfer;

impl AsyncTransfer {
    pub fn status_json(&self, published: Option<LoadConfig>) -> Value {
        let completed_copy_fence = unsafe { self.copy_fence.GetCompletedValue() };
        let completed_gate_fence = unsafe { self.gate_fence.GetCompletedValue() };
        let pending = self.pending.as_ref().map(|pending| {
            let stage = if pending
                .report
                .gate_fence
                .is_some_and(|value| completed_gate_fence < value)
            {
                "gated"
            } else if completed_copy_fence < pending.report.copy_fence {
                "copying"
            } else {
                "ready"
            };
            json!({
                "stage": stage,
                "report": pending.report,
                "pendingMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
            })
        });
        let reservation = self.reservation.as_ref().map(|reservation| {
            let mut value = json!({
                "stage": "materializing",
                "transactionId": reservation.transaction_id,
                "config": reservation.layout.config,
                "counts": reservation.layout.counts,
                "assignments": reservation.layout.assignments,
                "pendingMs": reservation.started_at.elapsed().as_secs_f64() * 1_000.0,
            });
            if let Some(global) = reservation.layout.global_config {
                value["globalConfig"] = json!(global);
            }
            value
        });
        json!({
            "revision": ASYNC_RESIDENT_REVISION,
            "capacity": ASYNC_CACHE_CAPACITY,
            "descriptorCount": ASYNC_CACHE_CAPACITY,
            "identityDescriptorCount": ASYNC_CACHE_CAPACITY,
            "inFlightCapacity": 1,
            "regionPayloadBytes": ASYNC_CACHE_CAPACITY * REGION_INSTANCE_BYTES,
            "defaultHeapAllocationBytes": self.region_allocation_bytes,
            "uploadArenaBytes": ASYNC_CACHE_CAPACITY * REGION_INSTANCE_BYTES,
            "identityPayloadBytes": ASYNC_CACHE_CAPACITY * REGION_IDENTITY_BYTES,
            "identityDefaultHeapAllocationBytes": self.identity_allocation_bytes,
            "identityUploadArenaBytes": ASYNC_CACHE_CAPACITY * REGION_IDENTITY_BYTES,
            "identityInitializationCopyCount": ASYNC_CACHE_CAPACITY,
            "published": published,
            "reservation": reservation,
            "pending": pending,
            "lastCompleted": self.last_completed,
            "gate": {
                "armedFence": self.armed_gate,
                "completedFence": completed_gate_fence,
            },
            "copy": {
                "completedFence": completed_copy_fence,
                "nextFence": self.next_copy_fence,
            },
        })
    }
}
