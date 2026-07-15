use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::address::{AddressedRegion, GlobalRegionConfig};
use crate::region::RegionCoord;

use super::{PairPurpose, Renderer};

impl Renderer {
    pub unsafe fn schedule_global_composition(
        &mut self,
        config: GlobalRegionConfig,
    ) -> Result<serde_json::Value> {
        anyhow::ensure!(
            !self.composition.traversal.is_enabled(),
            "camera traversal owns composition scheduling"
        );
        unsafe {
            self.schedule_composition_pair(config.local_config()?, config, PairPurpose::Manual)
        }
    }
}

pub(super) fn addressed(config: GlobalRegionConfig) -> Result<Vec<AddressedRegion>> {
    config.addressed_regions()
}

pub(super) fn local_ids(addressed: &[AddressedRegion]) -> Vec<u32> {
    addressed
        .iter()
        .map(|region| region.local_region_id)
        .collect()
}

pub(super) fn evidence(addressed: &[AddressedRegion]) -> (Vec<RegionCoord>, String) {
    let mut digest = Sha256::new();
    let mut globals = Vec::with_capacity(addressed.len());
    for region in addressed {
        globals.push(region.global_region);
        digest.update(region.global_region.x.to_le_bytes());
        digest.update(region.global_region.z.to_le_bytes());
        digest.update(region.local_region_id.to_le_bytes());
    }
    (globals, format!("{:x}", digest.finalize()))
}
