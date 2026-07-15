use anyhow::{Result, ensure};
use serde::Serialize;

use crate::region::RegionCoord;

pub const TERRAIN_POSITION_DENOMINATOR: i32 = 512;
pub const TERRAIN_POSITION_LOCAL_MIN_Q9: i32 = -4096;
pub const TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE: i32 = 4096;
pub const TERRAIN_POSITION_REGION_SIDE_Q9: i32 =
    TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE - TERRAIN_POSITION_LOCAL_MIN_Q9;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainPosition {
    region: RegionCoord,
    local_x_q9: i32,
    local_z_q9: i32,
}

impl TerrainPosition {
    pub fn new(region: RegionCoord, local_x_q9: i32, local_z_q9: i32) -> Result<Self> {
        ensure_local_coordinate("X", local_x_q9)?;
        ensure_local_coordinate("Z", local_z_q9)?;
        Ok(Self {
            region,
            local_x_q9,
            local_z_q9,
        })
    }

    pub fn translated_q9(self, delta_x_q9: i32, delta_z_q9: i32) -> Result<Self> {
        let (region_delta_x, local_x_q9) = normalize_axis(self.local_x_q9, delta_x_q9);
        let (region_delta_z, local_z_q9) = normalize_axis(self.local_z_q9, delta_z_q9);
        let region = self.region.checked_offset(region_delta_x, region_delta_z)?;
        Ok(Self {
            region,
            local_x_q9,
            local_z_q9,
        })
    }

    pub const fn region(self) -> RegionCoord {
        self.region
    }

    pub const fn local_x_q9(self) -> i32 {
        self.local_x_q9
    }

    pub const fn local_z_q9(self) -> i32 {
        self.local_z_q9
    }
}

fn ensure_local_coordinate(axis: &str, value: i32) -> Result<()> {
    ensure!(
        (TERRAIN_POSITION_LOCAL_MIN_Q9..TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE).contains(&value),
        "terrain position local {axis} Q9 coordinate must be in [{TERRAIN_POSITION_LOCAL_MIN_Q9}, {TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE})"
    );
    Ok(())
}

fn normalize_axis(local_q9: i32, delta_q9: i32) -> (i64, i32) {
    let displaced = i64::from(local_q9) + i64::from(delta_q9);
    let biased = displaced - i64::from(TERRAIN_POSITION_LOCAL_MIN_Q9);
    let region_side = i64::from(TERRAIN_POSITION_REGION_SIDE_Q9);
    let region_delta = biased.div_euclid(region_side);
    let local_q9 = biased.rem_euclid(region_side) + i64::from(TERRAIN_POSITION_LOCAL_MIN_Q9);
    (
        region_delta,
        i32::try_from(local_q9).expect("normalized terrain position local coordinate must fit i32"),
    )
}

#[cfg(test)]
#[path = "../../tests/private/terrain_position.rs"]
mod tests;
