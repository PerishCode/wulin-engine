use std::collections::BTreeSet;
use std::mem::size_of;

use anyhow::{Context, Result};
pub use region_format::InstanceRecord;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::load::{INSTANCES_PER_REGION, LoadConfig, MAX_REGION_SIDE};

pub const RESIDENT_REVISION: &str = "resident-region-v1";
pub const CACHE_REGION_CAPACITY: usize = 49;
pub const ACTIVE_REGION_CAPACITY: usize = 25;
pub const INSTANCE_RECORD_BYTES: usize = size_of::<InstanceRecord>();
pub const REGION_INSTANCE_BYTES: usize = INSTANCES_PER_REGION as usize * INSTANCE_RECORD_BYTES;
pub const ACTIVE_MAPPING_BYTES: usize = ACTIVE_REGION_CAPACITY * size_of::<ActiveRegion>();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ActiveRegion {
    pub slot: u32,
    pub region_id: u32,
}

#[derive(Clone)]
pub struct RegionCache {
    slots: [Option<CacheEntry>; CACHE_REGION_CAPACITY],
    clock: u64,
}

#[derive(Clone, Copy)]
struct CacheEntry {
    region_id: u32,
    last_used: u64,
}

impl Default for RegionCache {
    fn default() -> Self {
        Self {
            slots: [None; CACHE_REGION_CAPACITY],
            clock: 0,
        }
    }
}

pub struct StreamPlan {
    pub next_cache: RegionCache,
    pub uploads: Vec<RegionUpload>,
    pub active_regions: Vec<ActiveRegion>,
    pub report: StreamReport,
}

pub struct RegionUpload {
    pub slot: u32,
    pub records: Vec<InstanceRecord>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamReport {
    pub revision: &'static str,
    pub config: LoadConfig,
    pub retained_region_count: usize,
    pub uploaded_region_count: usize,
    pub evicted_region_count: usize,
    pub resident_region_count: usize,
    pub free_region_count: usize,
    pub instance_bytes: usize,
    pub mapping_bytes: usize,
    pub total_bytes: usize,
    pub uploaded_sha256: String,
    pub generation_ms: f64,
    pub transaction_ms: f64,
}

impl RegionCache {
    pub fn plan(&self, config: LoadConfig) -> Result<StreamPlan> {
        let generation_start = std::time::Instant::now();
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
                    .filter(|(_, entry)| !desired_set.contains(&entry.region_id))
                    .min_by_key(|(slot, entry)| (entry.last_used, *slot))
                    .map(|(slot, _)| slot)
                    .context("resident cache has no evictable region")?;
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

        let active_regions = desired
            .iter()
            .map(|region_id| {
                Ok(ActiveRegion {
                    slot: next
                        .slot_for(*region_id)
                        .context("active region is not resident")? as u32,
                    region_id: *region_id,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let uploaded_sha256 = hash_uploads(&uploads);
        let resident = next.slots.iter().flatten().count();
        let instance_bytes = uploads.len() * REGION_INSTANCE_BYTES;
        Ok(StreamPlan {
            next_cache: next,
            report: StreamReport {
                revision: RESIDENT_REVISION,
                config,
                retained_region_count: retained,
                uploaded_region_count: uploads.len(),
                evicted_region_count: evicted,
                resident_region_count: resident,
                free_region_count: CACHE_REGION_CAPACITY - resident,
                instance_bytes,
                mapping_bytes: ACTIVE_MAPPING_BYTES,
                total_bytes: instance_bytes + ACTIVE_MAPPING_BYTES,
                uploaded_sha256,
                generation_ms: generation_start.elapsed().as_secs_f64() * 1_000.0,
                transaction_ms: 0.0,
            },
            uploads,
            active_regions,
        })
    }

    fn slot_for(&self, region_id: u32) -> Option<usize> {
        self.slots
            .iter()
            .position(|entry| entry.is_some_and(|entry| entry.region_id == region_id))
    }
}

pub(crate) fn active_region_ids(config: LoadConfig) -> Result<Vec<u32>> {
    let diameter = config.active_radius * 2 + 1;
    let mut regions = Vec::with_capacity((diameter * diameter) as usize);
    for offset_z in 0..diameter {
        for offset_x in 0..diameter {
            let x = config.active_center_x + offset_x - config.active_radius;
            let z = config.active_center_z + offset_z - config.active_radius;
            regions.push(z * MAX_REGION_SIDE + x);
        }
    }
    if regions.len() != ACTIVE_REGION_CAPACITY {
        anyhow::bail!("resident mode requires exactly {ACTIVE_REGION_CAPACITY} active regions");
    }
    Ok(regions)
}

pub(crate) fn generate_region(region_id: u32) -> Vec<InstanceRecord> {
    let region_x = region_id % MAX_REGION_SIDE;
    let region_z = region_id / MAX_REGION_SIDE;
    (0..INSTANCES_PER_REGION)
        .map(|local_index| {
            let local_x = local_index % 32;
            let local_z = local_index / 32;
            let position_x =
                (region_x as i32 - 64) as f32 * 16.0 + ((local_x as f32 + 0.5) / 32.0 - 0.5) * 16.0;
            let position_z =
                (region_z as i32 - 64) as f32 * 16.0 + ((local_z as f32 + 0.5) / 32.0 - 0.5) * 16.0;
            let reference = region_id * INSTANCES_PER_REGION + local_index;
            InstanceRecord {
                position: [position_x, 0.0, position_z],
                height: instance_height(reference),
                region_id,
            }
        })
        .collect()
}

fn instance_height(reference: u32) -> f32 {
    let mut value = reference
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    value = (value >> 22) ^ value;
    0.7 + (value & 1023) as f32 / 1023.0 * 2.3
}

pub(crate) fn hash_uploads(uploads: &[RegionUpload]) -> String {
    let mut hash = Sha256::new();
    for upload in uploads {
        hash.update(upload.slot.to_le_bytes());
        hash.update(as_bytes(&upload.records));
    }
    format!("{:x}", hash.finalize())
}

pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}
