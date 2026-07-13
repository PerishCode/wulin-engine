use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::address::{AddressedRegion, GlobalRegionConfig};
use crate::load::LoadConfig;
use crate::resident::{REGION_INSTANCE_BYTES, RegionUpload, active_region_ids, hash_uploads};
use crate::world::RegionCoord;

pub const ASYNC_RESIDENT_REVISION: &str = "async-resident-v1";
pub const ASYNC_CACHE_CAPACITY: usize = 50;

#[derive(Clone)]
pub struct AsyncRegionCache {
    slots: [Option<CacheEntry>; ASYNC_CACHE_CAPACITY],
    clock: u64,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    key: CacheKey,
    last_used: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CacheKey {
    global_region: Option<RegionCoord>,
    local_region_id: u32,
}

#[derive(Clone, Copy)]
struct DesiredRegion {
    key: CacheKey,
    assignment: RegionAssignment,
}

impl Default for AsyncRegionCache {
    fn default() -> Self {
        Self {
            slots: [None; ASYNC_CACHE_CAPACITY],
            clock: 0,
        }
    }
}

#[derive(Clone)]
pub struct AsyncLayoutPlan {
    pub config: LoadConfig,
    pub global_config: Option<GlobalRegionConfig>,
    pub next_cache: AsyncRegionCache,
    pub assignments: Vec<RegionAssignment>,
    pub active_slots: Vec<u32>,
    pub reused_slots: Vec<u32>,
    pub counts: AsyncPlanCounts,
}

pub struct AsyncStreamPlan {
    pub layout: AsyncLayoutPlan,
    pub uploads: Vec<RegionUpload>,
    pub uploaded_sha256: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionAssignment {
    pub slot: u32,
    pub region_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_region: Option<RegionCoord>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_config: Option<GlobalRegionConfig>,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub uploaded_sha256: String,
    pub direct_release_fence: u64,
    pub copy_fence: u64,
    pub gate_fence: Option<u64>,
    pub payload_source: &'static str,
    pub payload_preparation_ms: f64,
    pub generation_ms: f64,
    pub schedule_ms: f64,
    pub pending_ms: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncReservationReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_config: Option<GlobalRegionConfig>,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub assignments: Vec<RegionAssignment>,
}

#[derive(Clone, Copy)]
pub struct PayloadPreparation {
    pub source: &'static str,
    pub total_ms: f64,
    pub generation_ms: f64,
}

impl PayloadPreparation {
    pub fn generated(generation_ms: f64) -> Self {
        Self {
            source: "generated",
            total_ms: generation_ms,
            generation_ms,
        }
    }

    pub fn cooked(total_ms: f64) -> Self {
        Self {
            source: "cooked-pack",
            total_ms,
            generation_ms: 0.0,
        }
    }
}

impl AsyncRegionCache {
    pub fn plan_layout(
        &self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncLayoutPlan> {
        let desired = active_region_ids(config)?
            .into_iter()
            .map(local_region)
            .collect();
        self.plan_layout_ordered(config, None, desired, protected_slots, false)
    }

    pub fn plan_composition_layout(
        &self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncLayoutPlan> {
        let desired = active_region_ids(config)?
            .into_iter()
            .map(local_region)
            .collect();
        self.plan_layout_ordered(config, None, desired, protected_slots, true)
    }

    pub fn plan_global_composition_layout(
        &self,
        config: GlobalRegionConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncLayoutPlan> {
        let local = config.local_config()?;
        let desired = config
            .addressed_regions()?
            .into_iter()
            .map(global_region)
            .collect();
        self.plan_layout_ordered(local, Some(config), desired, protected_slots, true)
    }

    fn plan_layout_ordered(
        &self,
        config: LoadConfig,
        global_config: Option<GlobalRegionConfig>,
        desired: Vec<DesiredRegion>,
        protected_slots: &BTreeSet<u32>,
        high_slots_first: bool,
    ) -> Result<AsyncLayoutPlan> {
        let desired_set = desired
            .iter()
            .map(|region| region.key)
            .collect::<BTreeSet<_>>();
        let mut next = self.clone();
        next.clock += 1;
        let mut retained = 0;
        for entry in next.slots.iter_mut().flatten() {
            if desired_set.contains(&entry.key) {
                entry.last_used = next.clock;
                retained += 1;
            }
        }

        let mut assignments = Vec::new();
        let mut reused_slots = Vec::new();
        let mut evicted = 0;
        for region in desired.iter().copied() {
            if next.slot_for(region.key).is_some() {
                continue;
            }
            let free_slot = if high_slots_first {
                next.slots.iter().rposition(Option::is_none)
            } else {
                next.slots.iter().position(Option::is_none)
            };
            let slot = if let Some(slot) = free_slot {
                slot
            } else {
                let slot = next
                    .slots
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, entry)| entry.map(|entry| (slot, entry)))
                    .filter(|(slot, entry)| {
                        !protected_slots.contains(&(*slot as u32))
                            && !desired_set.contains(&entry.key)
                    })
                    .min_by_key(|(slot, entry)| (entry.last_used, *slot))
                    .map(|(slot, _)| slot)
                    .context("async resident cache has no unprotected eviction candidate")?;
                reused_slots.push(slot as u32);
                evicted += 1;
                slot
            };
            next.slots[slot] = Some(CacheEntry {
                key: region.key,
                last_used: next.clock,
            });
            assignments.push(RegionAssignment {
                slot: slot as u32,
                ..region.assignment
            });
        }

        let active_slots = desired
            .iter()
            .map(|region| {
                next.slot_for(region.key)
                    .context("async active region is not resident")
                    .map(|slot| slot as u32)
            })
            .collect::<Result<Vec<_>>>()?;
        let resident = next.slots.iter().flatten().count();
        let counts = AsyncPlanCounts {
            retained_region_count: retained,
            uploaded_region_count: assignments.len(),
            evicted_region_count: evicted,
            protected_region_count: protected_slots.len(),
            resident_region_count: resident,
            free_region_count: ASYNC_CACHE_CAPACITY - resident,
            instance_bytes: assignments.len() * REGION_INSTANCE_BYTES,
        };
        Ok(AsyncLayoutPlan {
            config,
            global_config,
            next_cache: next,
            assignments,
            active_slots,
            reused_slots,
            counts,
        })
    }

    fn slot_for(&self, key: CacheKey) -> Option<usize> {
        self.slots
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.key == key))
    }
}

fn local_region(region_id: u32) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey {
            global_region: None,
            local_region_id: region_id,
        },
        assignment: RegionAssignment {
            slot: 0,
            region_id,
            global_region: None,
        },
    }
}

fn global_region(region: AddressedRegion) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey {
            global_region: Some(region.global_region),
            local_region_id: region.local_region_id,
        },
        assignment: RegionAssignment {
            slot: 0,
            region_id: region.local_region_id,
            global_region: Some(region.global_region),
        },
    }
}

impl AsyncLayoutPlan {
    pub fn materialize(self, uploads: Vec<RegionUpload>) -> Result<AsyncStreamPlan> {
        ensure!(
            uploads.len() == self.assignments.len(),
            "async payload count does not match the cache reservation"
        );
        for (assignment, upload) in self.assignments.iter().zip(&uploads) {
            ensure!(
                upload.slot == assignment.slot,
                "async payload slot does not match the cache reservation"
            );
            ensure!(
                upload.records.len() * std::mem::size_of::<crate::resident::InstanceRecord>()
                    == REGION_INSTANCE_BYTES,
                "async region payload has an invalid record count"
            );
            ensure!(
                upload
                    .records
                    .iter()
                    .all(|record| record.region_id == assignment.region_id),
                "async payload region does not match the cache reservation"
            );
        }
        let uploaded_sha256 = hash_uploads(&uploads);
        Ok(AsyncStreamPlan {
            layout: self,
            uploads,
            uploaded_sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protected(plan: &AsyncLayoutPlan) -> BTreeSet<u32> {
        plan.active_slots.iter().copied().collect()
    }

    #[test]
    fn global_alias_misses() {
        let zero = GlobalRegionConfig::new(0, 0, 0, 0, 2).unwrap();
        let first = AsyncRegionCache::default()
            .plan_global_composition_layout(zero, &BTreeSet::new())
            .unwrap();
        let far = 1_i64 << 40;
        let shifted = GlobalRegionConfig::new(far, -far, far, -far, 2).unwrap();
        let second = first
            .next_cache
            .plan_global_composition_layout(shifted, &protected(&first))
            .unwrap();
        assert_eq!(second.counts.retained_region_count, 0);
        assert_eq!(second.counts.uploaded_region_count, 25);
        assert_eq!(second.counts.resident_region_count, 50);
        assert!(
            second
                .assignments
                .iter()
                .all(|assignment| assignment.global_region.is_some())
        );
    }

    #[test]
    fn global_revisit_hits() {
        let base = GlobalRegionConfig::new(0, 0, 0, 0, 2).unwrap();
        let first = AsyncRegionCache::default()
            .plan_global_composition_layout(base, &BTreeSet::new())
            .unwrap();
        let adjacent_config = GlobalRegionConfig::new(0, 0, 1, 0, 2).unwrap();
        let adjacent = first
            .next_cache
            .plan_global_composition_layout(adjacent_config, &protected(&first))
            .unwrap();
        assert_eq!(adjacent.counts.retained_region_count, 20);
        assert_eq!(adjacent.counts.uploaded_region_count, 5);

        let revisit = adjacent
            .next_cache
            .plan_global_composition_layout(base, &protected(&adjacent))
            .unwrap();
        assert_eq!(revisit.counts.retained_region_count, 25);
        assert_eq!(revisit.counts.uploaded_region_count, 0);
        assert_eq!(revisit.counts.instance_bytes, 0);
    }
}
