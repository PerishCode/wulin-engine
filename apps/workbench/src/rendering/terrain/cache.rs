use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::load::LoadConfig;
use crate::resident::active_region_ids;
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
    global_region: Option<crate::world::RegionCoord>,
    local_region_id: u32,
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
    pub global_config: Option<GlobalTerrainConfig>,
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
    pub fn plan(&self, config: LoadConfig, protected: &BTreeSet<u32>) -> Result<LayoutPlan> {
        let desired = active_region_ids(config)?
            .into_iter()
            .map(legacy_region)
            .collect::<Vec<_>>();
        self.plan_regions(config, None, desired, protected)
    }

    pub fn plan_global(
        &self,
        config: GlobalTerrainConfig,
        protected: &BTreeSet<u32>,
    ) -> Result<LayoutPlan> {
        let local = config.local_config()?;
        let desired = config
            .addressed_regions()?
            .into_iter()
            .map(global_region)
            .collect::<Vec<_>>();
        self.plan_regions(local, Some(config), desired, protected)
    }

    fn plan_regions(
        &self,
        config: LoadConfig,
        global_config: Option<GlobalTerrainConfig>,
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

fn legacy_region(local_region_id: u32) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey {
            global_region: None,
            local_region_id,
        },
        assignment: TerrainAssignment {
            slot: 0,
            region_id: local_region_id,
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
        assignment: TerrainAssignment {
            slot: 0,
            region_id: region.local_region_id,
            global_region: Some(region.global_region),
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
    fn alias_miss_is_global() {
        let zero = GlobalTerrainConfig::new(0, 0, 0, 0, 2).unwrap();
        let first = TerrainCache::default()
            .plan_global(zero, &BTreeSet::new())
            .unwrap();
        assert_eq!(first.counts.uploaded_region_count, 25);

        let far = 1_i64 << 40;
        let shifted = GlobalTerrainConfig::new(far, -far, far, -far, 2).unwrap();
        let second = first
            .next_cache
            .plan_global(shifted, &active_slots(&first))
            .unwrap();
        assert_eq!(second.counts.retained_region_count, 0);
        assert_eq!(second.counts.uploaded_region_count, 25);
        assert_eq!(second.counts.resident_region_count, 50);
        assert_eq!(
            first
                .active
                .iter()
                .map(|entry| entry.region_id)
                .collect::<Vec<_>>(),
            second
                .active
                .iter()
                .map(|entry| entry.region_id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn adjacent_revisit_hits() {
        let origin = GlobalTerrainConfig::new(0, 0, 0, 0, 2).unwrap();
        let first = TerrainCache::default()
            .plan_global(origin, &BTreeSet::new())
            .unwrap();
        let adjacent_config = GlobalTerrainConfig::new(0, 0, 1, 0, 2).unwrap();
        let adjacent = first
            .next_cache
            .plan_global(adjacent_config, &active_slots(&first))
            .unwrap();
        assert_eq!(adjacent.counts.retained_region_count, 20);
        assert_eq!(adjacent.counts.uploaded_region_count, 5);
        assert_eq!(adjacent.counts.resident_region_count, 30);

        let revisit = adjacent
            .next_cache
            .plan_global(origin, &active_slots(&adjacent))
            .unwrap();
        assert_eq!(revisit.counts.retained_region_count, 25);
        assert_eq!(revisit.counts.uploaded_region_count, 0);
        assert_eq!(revisit.counts.payload_bytes, 0);
    }
}
