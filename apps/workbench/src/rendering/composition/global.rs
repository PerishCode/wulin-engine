use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::address::{AddressedRegion, GlobalRegionConfig};
use crate::load::LoadConfig;
use crate::resident::active_region_ids;
use crate::world::RegionCoord;

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
            self.schedule_composition_pair(
                config.local_config()?,
                Some(config),
                PairPurpose::Manual,
            )
        }
    }
}

pub(super) fn addressed(
    config: Option<GlobalRegionConfig>,
) -> Result<Option<Vec<AddressedRegion>>> {
    config
        .map(GlobalRegionConfig::addressed_regions)
        .transpose()
}

pub(super) fn local_ids(
    addressed: Option<&[AddressedRegion]>,
    config: LoadConfig,
) -> Result<Vec<u32>> {
    match addressed {
        Some(regions) => Ok(regions
            .iter()
            .map(|region| region.local_region_id)
            .collect()),
        None => active_region_ids(config),
    }
}

pub(super) fn evidence(
    addressed: Option<&[AddressedRegion]>,
) -> (Option<Vec<RegionCoord>>, Option<String>) {
    let Some(regions) = addressed else {
        return (None, None);
    };
    let mut digest = Sha256::new();
    let mut globals = Vec::with_capacity(regions.len());
    for region in regions {
        globals.push(region.global_region);
        digest.update(region.global_region.x.to_le_bytes());
        digest.update(region.global_region.z.to_le_bytes());
        digest.update(region.local_region_id.to_le_bytes());
    }
    (Some(globals), Some(format!("{:x}", digest.finalize())))
}
