use std::mem::size_of;

use anyhow::Result;
pub use region_format::{InstanceRecord, PresentationRecord};
use sha2::{Digest, Sha256};

use crate::load::{INSTANCES_PER_REGION, LoadConfig, MAX_REGION_SIDE};

pub const ACTIVE_REGION_CAPACITY: usize = 25;
pub const INSTANCE_RECORD_BYTES: usize = size_of::<InstanceRecord>();
pub const REGION_INSTANCE_BYTES: usize = INSTANCES_PER_REGION as usize * INSTANCE_RECORD_BYTES;
pub const REGION_IDENTITY_BYTES: usize = INSTANCES_PER_REGION as usize * size_of::<u32>();
pub const REGION_PRESENTATION_BYTES: usize =
    INSTANCES_PER_REGION as usize * size_of::<PresentationRecord>();

pub struct RegionUpload {
    pub slot: u32,
    pub records: Vec<InstanceRecord>,
    pub local_ids: Vec<u32>,
    pub presentations: Vec<PresentationRecord>,
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
    if regions.len() > ACTIVE_REGION_CAPACITY {
        anyhow::bail!("resident mapping supports at most {ACTIVE_REGION_CAPACITY} active regions");
    }
    Ok(regions)
}

pub(crate) fn canonical_stable_key(region_seed: u32, local_index: u32) -> u32 {
    region_seed ^ local_index.wrapping_mul(747_796_405)
}

pub(crate) fn hash_uploads(uploads: &[RegionUpload]) -> String {
    let mut hash = Sha256::new();
    for upload in uploads {
        hash.update(upload.slot.to_le_bytes());
        hash.update(as_bytes(&upload.records));
        hash.update(as_bytes(&upload.local_ids));
        hash.update(as_bytes(&upload.presentations));
    }
    format!("{:x}", hash.finalize())
}

pub fn as_bytes<T>(values: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}
