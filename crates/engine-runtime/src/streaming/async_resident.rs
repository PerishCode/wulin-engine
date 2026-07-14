use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::address::GlobalRegionConfig;
use crate::load::LoadConfig;
use crate::resident::{REGION_INSTANCE_BYTES, RegionUpload, hash_uploads};
use crate::world::RegionCoord;

pub const ASYNC_RESIDENT_REVISION: &str = "async-resident-v1";
pub const ASYNC_CACHE_CAPACITY: usize = 50;

mod canonical;

pub use canonical::{ObjectSourceNamespace, canonical_stable_seed};

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
    source_namespace: ObjectSourceNamespace,
    global_region: RegionCoord,
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
    pub global_config: GlobalRegionConfig,
    pub object_source_namespace: ObjectSourceNamespace,
    pub object_stable_seed_namespace: ObjectSourceNamespace,
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
    pub global_region: RegionCoord,
    pub stable_seed: u32,
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
    pub global_config: GlobalRegionConfig,
    pub object_source_namespace: ObjectSourceNamespace,
    #[serde(skip)]
    pub object_stable_seed_namespace: ObjectSourceNamespace,
    #[serde(skip)]
    pub object_page_checksums: Vec<[u8; 32]>,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub uploaded_sha256: String,
    pub identity_copy_count: usize,
    pub identity_copy_bytes: usize,
    pub presentation_copy_count: usize,
    pub presentation_copy_bytes: usize,
    pub direct_release_fence: u64,
    pub copy_fence: u64,
    pub gate_fence: Option<u64>,
    pub payload_preparation_ms: f64,
    pub schedule_ms: f64,
    pub pending_ms: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncReservationReport {
    pub revision: &'static str,
    pub transaction_id: u64,
    pub config: LoadConfig,
    pub global_config: GlobalRegionConfig,
    pub object_source_namespace: ObjectSourceNamespace,
    #[serde(skip)]
    pub object_stable_seed_namespace: ObjectSourceNamespace,
    #[serde(flatten)]
    pub counts: AsyncPlanCounts,
    pub assignments: Vec<RegionAssignment>,
}

impl AsyncRegionCache {
    pub fn plan_canonical_layout(
        &self,
        config: GlobalRegionConfig,
        source_namespace: ObjectSourceNamespace,
        stable_seed_namespace: ObjectSourceNamespace,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncLayoutPlan> {
        let local = config.local_config()?;
        let desired = config
            .addressed_regions()?
            .into_iter()
            .map(|region| canonical::desired(region, source_namespace, stable_seed_namespace))
            .collect::<Vec<_>>();
        let unique_seeds = desired
            .iter()
            .map(|region| region.assignment.stable_seed)
            .collect::<BTreeSet<_>>();
        ensure!(
            unique_seeds.len() == desired.len(),
            "canonical object stable seeds collide inside the active window"
        );
        self.plan_layout_ordered(
            local,
            config,
            source_namespace,
            stable_seed_namespace,
            desired,
            protected_slots,
        )
    }

    fn plan_layout_ordered(
        &self,
        config: LoadConfig,
        global_config: GlobalRegionConfig,
        object_source_namespace: ObjectSourceNamespace,
        object_stable_seed_namespace: ObjectSourceNamespace,
        desired: Vec<DesiredRegion>,
        protected_slots: &BTreeSet<u32>,
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
            let free_slot = next.slots.iter().rposition(Option::is_none);
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
            object_source_namespace,
            object_stable_seed_namespace,
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
                    .all(|record| { record.region_id == assignment.stable_seed }),
                "async payload region does not match the cache reservation"
            );
            let local_ids = &upload.local_ids;
            ensure!(
                local_ids.len() == crate::load::INSTANCES_PER_REGION as usize,
                "async identity payload has an invalid local-ID count"
            );
            let unique = local_ids.iter().copied().collect::<BTreeSet<_>>();
            ensure!(
                unique.len() == local_ids.len()
                    && unique.first() == Some(&0)
                    && unique.last() == Some(&(crate::load::INSTANCES_PER_REGION - 1)),
                "async identity payload is not a canonical local-ID permutation"
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

    fn namespace(revision: &str) -> ObjectSourceNamespace {
        use sha2::Digest as _;

        ObjectSourceNamespace::from_bytes(sha2::Sha256::digest(revision.as_bytes()).into())
    }

    fn protected(plan: &AsyncLayoutPlan) -> BTreeSet<u32> {
        plan.active_slots.iter().copied().collect()
    }

    #[test]
    fn canonical_alias_rebind_hits() {
        let far = 1_i64 << 40;
        let source = namespace("canonical-object-test-v1");
        let base = GlobalRegionConfig::new(far, -far, far, -far, 2).unwrap();
        let first = AsyncRegionCache::default()
            .plan_canonical_layout(base, source, source, &BTreeSet::new())
            .unwrap();
        let alias = GlobalRegionConfig::new(far - 32, -far, far, -far, 2).unwrap();
        let rebound = first
            .next_cache
            .plan_canonical_layout(alias, source, source, &protected(&first))
            .unwrap();
        assert_eq!(rebound.counts.retained_region_count, 25);
        assert_eq!(rebound.counts.uploaded_region_count, 0);
        assert!(rebound.assignments.is_empty());
    }

    #[test]
    fn canonical_source_change_misses() {
        let config = GlobalRegionConfig::new(0, 0, 0, 0, 2).unwrap();
        let first_source = namespace("canonical-object-test-v1");
        let second_source = namespace("canonical-object-test-v2");
        let first = AsyncRegionCache::default()
            .plan_canonical_layout(config, first_source, first_source, &BTreeSet::new())
            .unwrap();
        let second = first
            .next_cache
            .plan_canonical_layout(config, second_source, second_source, &protected(&first))
            .unwrap();
        assert_eq!(second.counts.retained_region_count, 0);
        assert_eq!(second.counts.uploaded_region_count, 25);
        assert_eq!(second.counts.resident_region_count, 50);
    }
}
