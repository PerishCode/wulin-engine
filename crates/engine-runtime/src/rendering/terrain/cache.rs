use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::load::LoadConfig;
use crate::terrain::TerrainSourceNamespace;
use crate::terrain::{AddressedRegion, GlobalTerrainConfig, TerrainAssignment, TerrainPlanCounts};

pub(super) const TERRAIN_CACHE_CAPACITY: usize = 50;
pub(super) const TERRAIN_ACTIVE_CAPACITY: usize = 25;

#[derive(Clone)]
pub(super) struct TerrainCache {
    slots: [Option<CacheEntry>; TERRAIN_CACHE_CAPACITY],
    clock: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CacheKey {
    source_namespace: TerrainSourceNamespace,
    global_region: crate::region::RegionCoord,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    key: CacheKey,
    last_used: u64,
}

#[derive(Clone, Copy)]
struct DesiredRegion {
    key: CacheKey,
    assignment: TerrainAssignment,
}

pub(super) struct LayoutPlan {
    pub config: LoadConfig,
    pub global_config: GlobalTerrainConfig,
    pub source_namespace: TerrainSourceNamespace,
    pub next_cache: TerrainCache,
    pub assignments: Vec<TerrainAssignment>,
    pub active: Vec<TerrainAssignment>,
    pub reused_slots: Vec<u32>,
    pub counts: TerrainPlanCounts,
}

impl Default for TerrainCache {
    fn default() -> Self {
        Self {
            slots: [None; TERRAIN_CACHE_CAPACITY],
            clock: 0,
        }
    }
}

impl TerrainCache {
    pub fn plan_canonical_global(
        &self,
        config: GlobalTerrainConfig,
        source_namespace: TerrainSourceNamespace,
        protected: &BTreeSet<u32>,
    ) -> Result<LayoutPlan> {
        let local = config.local_config()?;
        let desired = config
            .addressed_regions()?
            .into_iter()
            .map(|region| canonical_global_region(region, source_namespace))
            .collect::<Vec<_>>();
        self.plan_regions(local, config, source_namespace, desired, protected)
    }

    fn plan_regions(
        &self,
        config: LoadConfig,
        global_config: GlobalTerrainConfig,
        source_namespace: TerrainSourceNamespace,
        desired: Vec<DesiredRegion>,
        protected: &BTreeSet<u32>,
    ) -> Result<LayoutPlan> {
        ensure!(
            desired.len() <= TERRAIN_ACTIVE_CAPACITY,
            "terrain active capacity exceeded"
        );
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
            let slot = if let Some(slot) = next.slots.iter().position(Option::is_none) {
                slot
            } else {
                let slot = next
                    .slots
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, entry)| entry.map(|entry| (slot, entry)))
                    .filter(|(slot, entry)| {
                        !protected.contains(&(*slot as u32)) && !desired_set.contains(&entry.key)
                    })
                    .min_by_key(|(slot, entry)| (entry.last_used, *slot))
                    .map(|(slot, _)| slot)
                    .context("terrain cache has no unprotected eviction candidate")?;
                reused_slots.push(slot as u32);
                evicted += 1;
                slot
            };
            next.slots[slot] = Some(CacheEntry {
                key: region.key,
                last_used: next.clock,
            });
            assignments.push(TerrainAssignment {
                slot: slot as u32,
                ..region.assignment
            });
        }
        let active = desired
            .iter()
            .map(|region| {
                next.slot_for(region.key)
                    .context("active terrain region is not resident")
                    .map(|slot| TerrainAssignment {
                        slot: slot as u32,
                        ..region.assignment
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let resident = next.slots.iter().flatten().count();
        Ok(LayoutPlan {
            config,
            global_config,
            source_namespace,
            next_cache: next,
            assignments: assignments.clone(),
            active,
            reused_slots,
            counts: TerrainPlanCounts {
                retained_region_count: retained,
                uploaded_region_count: assignments.len(),
                evicted_region_count: evicted,
                protected_region_count: protected.len(),
                resident_region_count: resident,
                free_region_count: TERRAIN_CACHE_CAPACITY - resident,
                payload_bytes: assignments.len() * terrain_format::PAYLOAD_BYTES as usize,
            },
        })
    }

    fn slot_for(&self, key: CacheKey) -> Option<usize> {
        self.slots
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.key == key))
    }
}

fn canonical_global_region(
    region: AddressedRegion,
    source_namespace: TerrainSourceNamespace,
) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey {
            source_namespace,
            global_region: region.global_region,
        },
        assignment: TerrainAssignment {
            slot: 0,
            region_id: region.local_region_id,
            global_region: region.global_region,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_slots(plan: &LayoutPlan) -> BTreeSet<u32> {
        plan.active.iter().map(|entry| entry.slot).collect()
    }

    #[test]
    fn canonical_rebind_retains_slots() {
        let source = TerrainSourceNamespace([7; 32]);
        let far = 1_i64 << 40;
        let first_config = GlobalTerrainConfig::new(far, -far, far, -far, 2).unwrap();
        let first = TerrainCache::default()
            .plan_canonical_global(first_config, source, &BTreeSet::new())
            .unwrap();
        let shifted_config = GlobalTerrainConfig::new(far - 32, -far, far, -far, 2).unwrap();
        let shifted = first
            .next_cache
            .plan_canonical_global(shifted_config, source, &active_slots(&first))
            .unwrap();
        assert_eq!(shifted.counts.retained_region_count, 25);
        assert_eq!(shifted.counts.uploaded_region_count, 0);
        assert_eq!(shifted.counts.resident_region_count, 25);
        assert_ne!(
            first
                .active
                .iter()
                .map(|entry| entry.region_id)
                .collect::<Vec<_>>(),
            shifted
                .active
                .iter()
                .map(|entry| entry.region_id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            first
                .active
                .iter()
                .map(|entry| (entry.global_region, entry.slot))
                .collect::<Vec<_>>(),
            shifted
                .active
                .iter()
                .map(|entry| (entry.global_region, entry.slot))
                .collect::<Vec<_>>()
        );

        let changed_source = shifted
            .next_cache
            .plan_canonical_global(
                first_config,
                TerrainSourceNamespace([8; 32]),
                &BTreeSet::new(),
            )
            .unwrap();
        assert_eq!(changed_source.counts.retained_region_count, 0);
        assert_eq!(changed_source.counts.uploaded_region_count, 25);
        assert_eq!(changed_source.counts.resident_region_count, 50);
    }
}
