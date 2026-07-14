use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::address::GlobalRegionConfig;
use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::world::RegionCoord;

use super::{TraversalBasis, TraversalTarget};

const MINIMUM_LOCAL_CENTER: u32 = 32;
const MAXIMUM_LOCAL_CENTER: u32 = 96;
const RECENTER_LOCAL_CENTER: u32 = MAX_REGION_SIDE / 2;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RolloverPolicy {
    minimum_local_center: u32,
    maximum_local_center: u32,
    recenter_local_center: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RolloverEvent {
    token: u64,
    old_origin: RegionCoord,
    new_origin: RegionCoord,
    global_center: RegionCoord,
    local_center: [u32; 2],
    camera_delta_regions: [i32; 2],
    camera_delta_meters: [i32; 2],
}

#[derive(Default)]
pub(super) struct RolloverState {
    active: bool,
    count: u64,
    cumulative_camera_delta_regions: [i64; 2],
    camera_delta: Option<[i32; 2]>,
    last: Option<RolloverEvent>,
}

impl RolloverPolicy {
    pub(super) fn canonical() -> Self {
        Self {
            minimum_local_center: MINIMUM_LOCAL_CENTER,
            maximum_local_center: MAXIMUM_LOCAL_CENTER,
            recenter_local_center: RECENTER_LOCAL_CENTER,
        }
    }

    pub(super) fn target(
        self,
        basis: TraversalBasis,
        config: LoadConfig,
    ) -> Result<TraversalTarget> {
        let origin = basis.global_origin;
        let center = basis.global_center(config.active_center_x, config.active_center_z)?;
        let recenter_x = !(self.minimum_local_center..=self.maximum_local_center)
            .contains(&config.active_center_x);
        let recenter_z = !(self.minimum_local_center..=self.maximum_local_center)
            .contains(&config.active_center_z);
        let next_origin = RegionCoord::new(
            if recenter_x { center.x } else { origin.x },
            if recenter_z { center.z } else { origin.z },
        );
        let next_local = LoadConfig::new(
            basis.world_region_side,
            if recenter_x {
                self.recenter_local_center
            } else {
                config.active_center_x
            },
            if recenter_z {
                self.recenter_local_center
            } else {
                config.active_center_z
            },
            basis.active_radius,
        )?;
        let global_config = GlobalRegionConfig::new(
            next_origin.x,
            next_origin.z,
            center.x,
            center.z,
            basis.active_radius,
        )?;
        ensure!(
            global_config.local_config()? == next_local,
            "canonical rollover target does not preserve signed center"
        );
        Ok(TraversalTarget {
            config: next_local,
            global_config,
        })
    }
}

impl RolloverState {
    pub(super) fn activate(&mut self) {
        self.active = true;
        self.camera_delta = None;
    }

    pub(super) fn deactivate(&mut self) {
        self.active = false;
        self.camera_delta = None;
    }

    pub(super) fn commit(
        &mut self,
        token: u64,
        basis: &mut TraversalBasis,
        target: TraversalTarget,
    ) -> Result<()> {
        let global = target.global_config;
        ensure!(
            global.local_config()? == target.config,
            "canonical rollover publication local/global configs diverged"
        );
        let old_origin = basis.global_origin;
        let new_origin = global.global_origin;
        if old_origin == new_origin {
            return Ok(());
        }
        let delta = [
            checked_camera_delta(old_origin.x, new_origin.x)?,
            checked_camera_delta(old_origin.z, new_origin.z)?,
        ];
        let region_meters = terrain_format::REGION_SIDE_METERS as i32;
        let meters = [
            delta[0]
                .checked_mul(region_meters)
                .context("rollover X camera delta overflowed meters")?,
            delta[1]
                .checked_mul(region_meters)
                .context("rollover Z camera delta overflowed meters")?,
        ];
        basis.global_origin = new_origin;
        self.count += 1;
        self.cumulative_camera_delta_regions[0] += i64::from(delta[0]);
        self.cumulative_camera_delta_regions[1] += i64::from(delta[1]);
        self.camera_delta = Some(delta);
        self.last = Some(RolloverEvent {
            token,
            old_origin,
            new_origin,
            global_center: global.global_center,
            local_center: [target.config.active_center_x, target.config.active_center_z],
            camera_delta_regions: delta,
            camera_delta_meters: meters,
        });
        Ok(())
    }

    pub(super) fn take_camera_delta(&mut self) -> Option<[i32; 2]> {
        self.camera_delta.take()
    }

    pub(super) fn status_json(&self) -> Option<Value> {
        self.active.then(|| {
            json!({
                "revision": "canonical-origin-rollover-v1",
                "minimumLocalCenter": MINIMUM_LOCAL_CENTER,
                "maximumLocalCenter": MAXIMUM_LOCAL_CENTER,
                "recenterLocalCenter": RECENTER_LOCAL_CENTER,
                "count": self.count,
                "cumulativeCameraDeltaRegions": self.cumulative_camera_delta_regions,
                "pendingCameraDeltaRegions": self.camera_delta,
                "last": self.last,
            })
        })
    }
}

fn checked_camera_delta(old: i64, new: i64) -> Result<i32> {
    let delta = old
        .checked_sub(new)
        .context("rollover camera delta overflowed signed regions")?;
    i32::try_from(delta).context("rollover camera delta exceeds the bounded local extent")
}
