use std::collections::BTreeSet;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::load::LoadConfig;
use crate::resident::{
    REGION_INSTANCE_BYTES, RegionUpload, active_region_ids, generate_region, hash_uploads,
};

pub const ASYNC_RESIDENT_REVISION: &str = "async-resident-v1";
pub const ASYNC_CACHE_CAPACITY: usize = 50;

#[derive(Clone)]
pub struct AsyncRegionCache {
    slots: [Option<CacheEntry>; ASYNC_CACHE_CAPACITY],
    clock: u64,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    region_id: u32,
    last_used: u64,
}

impl Default for AsyncRegionCache {
    fn default() -> Self {
        Self {
            slots: [None; ASYNC_CACHE_CAPACITY],
            clock: 0,
        }
    }
}

pub struct AsyncStreamPlan {
    pub next_cache: AsyncRegionCache,
    pub uploads: Vec<RegionUpload>,
    pub active_slots: Vec<u32>,
    pub reused_slots: Vec<u32>,
    pub counts: AsyncPlanCounts,
    pub uploaded_sha256: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncPlanCounts {
    pub retained_region_count: usize,
    pub uploaded_region_count: usize,
    pub evicted_region_count: usize,
    pub protected_region_count: usize,
    pub resident_region_count: usize,
    pub free_region_count: usize,
    pub instance_bytes: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncTransactionReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub uploaded_sha256: String,
    pub direct_release_fence: u64,
    pub copy_fence: u64,
    pub gate_fence: Option<u64>,
    pub generation_ms: f64,
    pub schedule_ms: f64,
    pub pending_ms: f64,
}

impl AsyncRegionCache {
    pub fn plan(
        &self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncStreamPlan> {
        let desired = active_region_ids(config)?;
        let desired_set = desired.iter().copied().collect::<BTreeSet<_>>();
        let mut next = self.clone();
        next.clock += 1;
        let mut retained = 0;
        for entry in next.slots.iter_mut().flatten() {
            if desired_set.contains(&entry.region_id) {
                entry.last_used = next.clock;
                retained += 1;
            }
        }

        let mut uploads = Vec::new();
        let mut reused_slots = Vec::new();
        let mut evicted = 0;
        for region_id in desired.iter().copied() {
            if next.slot_for(region_id).is_some() {
                continue;
            }
            let slot = if let Some(slot) = next.slots.iter().position(Option::is_none) {
                slot
            } else {
                let slot = next
                    .slots
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, entry)| entry.map(|entry| (slot, entry)))
                    .filter(|(slot, entry)| {
                        !protected_slots.contains(&(*slot as u32))
                            && !desired_set.contains(&entry.region_id)
                    })
                    .min_by_key(|(slot, entry)| (entry.last_used, *slot))
                    .map(|(slot, _)| slot)
                    .context("async resident cache has no unprotected eviction candidate")?;
                reused_slots.push(slot as u32);
                evicted += 1;
                slot
            };
            next.slots[slot] = Some(CacheEntry {
                region_id,
                last_used: next.clock,
            });
            uploads.push(RegionUpload {
                slot: slot as u32,
                records: generate_region(region_id),
            });
        }

        let active_slots = desired
            .iter()
            .map(|region_id| {
                next.slot_for(*region_id)
                    .context("async active region is not resident")
                    .map(|slot| slot as u32)
            })
            .collect::<Result<Vec<_>>>()?;
        let uploaded_sha256 = hash_uploads(&uploads);
        let resident = next.slots.iter().flatten().count();
        let counts = AsyncPlanCounts {
            retained_region_count: retained,
            uploaded_region_count: uploads.len(),
            evicted_region_count: evicted,
            protected_region_count: protected_slots.len(),
            resident_region_count: resident,
            free_region_count: ASYNC_CACHE_CAPACITY - resident,
            instance_bytes: uploads.len() * REGION_INSTANCE_BYTES,
        };
        Ok(AsyncStreamPlan {
            next_cache: next,
            uploads,
            active_slots,
            reused_slots,
            counts,
            uploaded_sha256,
        })
    }

    fn slot_for(&self, region_id: u32) -> Option<usize> {
        self.slots
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.region_id == region_id))
    }
}
