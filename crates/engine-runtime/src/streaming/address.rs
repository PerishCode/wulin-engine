use anyhow::{Result, ensure};
use serde::Serialize;

use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::region::RegionCoord;
use crate::resident::active_region_ids;

const LOCAL_ORIGIN: u32 = MAX_REGION_SIDE / 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalRegionConfig {
    pub global_origin: RegionCoord,
    pub global_center: RegionCoord,
    pub active_radius: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressedRegion {
    pub global_region: RegionCoord,
    pub local_region_id: u32,
}

impl GlobalRegionConfig {
    pub fn new(
        origin_x: i64,
        origin_z: i64,
        center_x: i64,
        center_z: i64,
        active_radius: u32,
    ) -> Result<Self> {
        let config = Self {
            global_origin: RegionCoord::new(origin_x, origin_z),
            global_center: RegionCoord::new(center_x, center_z),
            active_radius,
        };
        config.local_config()?;
        Ok(config)
    }

    pub fn local_config(self) -> Result<LoadConfig> {
        let offset_x = checked_delta(self.global_center.x, self.global_origin.x, "X")?;
        let offset_z = checked_delta(self.global_center.z, self.global_origin.z, "Z")?;
        let center_x = checked_local_axis(offset_x, "X")?;
        let center_z = checked_local_axis(offset_z, "Z")?;
        LoadConfig::new(MAX_REGION_SIDE, center_x, center_z, self.active_radius)
    }

    pub fn addressed_regions(self) -> Result<Vec<AddressedRegion>> {
        let local = self.local_config()?;
        let local_ids = active_region_ids(local)?;
        let diameter = i64::from(self.active_radius * 2 + 1);
        let mut regions = Vec::with_capacity(local_ids.len());
        for offset_z in 0..diameter {
            for offset_x in 0..diameter {
                let global_region = self.global_center.checked_offset(
                    offset_x - i64::from(self.active_radius),
                    offset_z - i64::from(self.active_radius),
                )?;
                let local_region_id = local_ids[regions.len()];
                regions.push(AddressedRegion {
                    global_region,
                    local_region_id,
                });
            }
        }
        ensure!(
            regions.len() == local.active_region_count() as usize,
            "global region mapping is incomplete"
        );
        Ok(regions)
    }

    pub(crate) fn addressed_region(
        self,
        global_region: RegionCoord,
    ) -> Result<Option<AddressedRegion>> {
        let offset_x = i128::from(global_region.x) - i128::from(self.global_center.x);
        let offset_z = i128::from(global_region.z) - i128::from(self.global_center.z);
        let radius = i128::from(self.active_radius);
        if !(-radius..=radius).contains(&offset_x) || !(-radius..=radius).contains(&offset_z) {
            return Ok(None);
        }

        let local = self.local_config()?;
        let local_x = i128::from(local.active_center_x) + offset_x;
        let local_z = i128::from(local.active_center_z) + offset_z;
        let local_region_id = u32::try_from(local_z * i128::from(MAX_REGION_SIDE) + local_x)
            .map_err(|_| anyhow::anyhow!("addressed local region ID overflowed"))?;
        Ok(Some(AddressedRegion {
            global_region,
            local_region_id,
        }))
    }
}

fn checked_delta(value: i64, origin: i64, axis: &str) -> Result<i64> {
    value
        .checked_sub(origin)
        .ok_or_else(|| anyhow::anyhow!("global region {axis} delta overflowed"))
}

fn checked_local_axis(offset: i64, axis: &str) -> Result<u32> {
    let value = i64::from(LOCAL_ORIGIN)
        .checked_add(offset)
        .ok_or_else(|| anyhow::anyhow!("local region {axis} alias overflowed"))?;
    ensure!(
        (0..i64::from(MAX_REGION_SIDE)).contains(&value),
        "global region {axis} maps outside the bounded projection extent"
    );
    Ok(value as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_mapping_is_exact() {
        let far = 1_i64 << 40;
        let config = GlobalRegionConfig::new(far, -far, far + 1, -far, 2).unwrap();
        let local = config.local_config().unwrap();
        assert_eq!((local.active_center_x, local.active_center_z), (65, 64));
        let regions = config.addressed_regions().unwrap();
        assert_eq!(regions.len(), 25);
        assert_eq!(
            regions[0].global_region,
            RegionCoord::new(far - 1, -far - 2)
        );
        assert_eq!(regions[0].local_region_id, 62 * 128 + 63);
    }

    #[test]
    fn overflow_is_rejected() {
        assert!(GlobalRegionConfig::new(i64::MIN, 0, i64::MAX, 0, 2).is_err());
    }

    #[test]
    fn address_avoids_signed_overflow() {
        let config =
            GlobalRegionConfig::new(i64::MAX - 4, i64::MIN + 4, i64::MAX - 3, i64::MIN + 5, 2)
                .unwrap();
        let addressed = config
            .addressed_region(RegionCoord::new(i64::MAX - 1, i64::MIN + 3))
            .unwrap()
            .unwrap();
        assert_eq!(addressed.local_region_id, 63 * MAX_REGION_SIDE + 67);
        assert_eq!(
            config
                .addressed_region(RegionCoord::new(i64::MIN, i64::MAX))
                .unwrap(),
            None
        );
    }
}
