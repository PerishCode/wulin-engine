use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::load::LoadConfig;
use crate::resident::active_region_ids;
use crate::terrain::{TerrainAssignment, TerrainPlanCounts};

pub(super) const TERRAIN_CACHE_CAPACITY: usize = 50;
pub(super) const TERRAIN_ACTIVE_CAPACITY: usize = 25;

#[derive(Clone)]
pub(super) struct TerrainCache {
    slots: [Option<CacheEntry>; TERRAIN_CACHE_CAPACITY],
    clock: u64,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    region_id: u32,
    last_used: u64,
}

pub(super) struct LayoutPlan {
    pub config: LoadConfig,
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
        let desired = active_region_ids(config)?;
        ensure!(
            desired.len() <= TERRAIN_ACTIVE_CAPACITY,
            "terrain active capacity exceeded"
        );
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
        let mut assignments = Vec::new();
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
                        !protected.contains(&(*slot as u32))
                            && !desired_set.contains(&entry.region_id)
                    })
                    .min_by_key(|(slot, entry)| (entry.last_used, *slot))
                    .map(|(slot, _)| slot)
                    .context("terrain cache has no unprotected eviction candidate")?;
                reused_slots.push(slot as u32);
                evicted += 1;
                slot
            };
            next.slots[slot] = Some(CacheEntry {
                region_id,
                last_used: next.clock,
            });
            assignments.push(TerrainAssignment {
                slot: slot as u32,
                region_id,
            });
        }
        let active = desired
            .iter()
            .map(|region_id| {
                next.slot_for(*region_id)
                    .context("active terrain region is not resident")
                    .map(|slot| TerrainAssignment {
                        slot: slot as u32,
                        region_id: *region_id,
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let resident = next.slots.iter().flatten().count();
        Ok(LayoutPlan {
            config,
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

    fn slot_for(&self, region_id: u32) -> Option<usize> {
        self.slots
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.region_id == region_id))
    }
}
