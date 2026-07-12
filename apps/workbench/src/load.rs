use anyhow::{Result, bail};
use serde::Serialize;
use serde_json::{Value, json};

pub const LOAD_REVISION: &str = "region-grid-v1";
pub const MAX_REGION_SIDE: u32 = 128;
pub const REGION_INSTANCE_SIDE: u32 = 32;
pub const INSTANCES_PER_REGION: u32 = REGION_INSTANCE_SIDE * REGION_INSTANCE_SIDE;
pub const REGION_OBJECT_ID_BASE: u32 = 65_536;
pub const MAX_ACTIVE_RADIUS: u32 = 4;
pub const MAX_VISIBLE_INSTANCES: u32 =
    (MAX_ACTIVE_RADIUS * 2 + 1) * (MAX_ACTIVE_RADIUS * 2 + 1) * INSTANCES_PER_REGION;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadConfig {
    pub world_region_side: u32,
    pub active_center_x: u32,
    pub active_center_z: u32,
    pub active_radius: u32,
}

pub struct RegionSemantic {
    pub name: String,
    pub kind: String,
    pub color: [f32; 4],
}

impl LoadConfig {
    pub fn new(
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    ) -> Result<Self> {
        if !matches!(world_region_side, 32 | 64 | 128) {
            bail!("worldRegionSide must be one of 32, 64, or 128");
        }
        if active_radius > MAX_ACTIVE_RADIUS {
            bail!("activeRadius must be in the range 0..={MAX_ACTIVE_RADIUS}");
        }
        let world_start = (MAX_REGION_SIDE - world_region_side) / 2;
        let world_end = world_start + world_region_side;
        let radius = active_radius;
        if active_center_x < world_start + radius
            || active_center_z < world_start + radius
            || active_center_x + radius >= world_end
            || active_center_z + radius >= world_end
        {
            bail!("active region window must remain inside the configured world");
        }
        Ok(Self {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        })
    }

    pub fn logical_instance_count(self) -> u64 {
        u64::from(self.world_region_side).pow(2) * u64::from(INSTANCES_PER_REGION)
    }

    pub fn active_region_count(self) -> u32 {
        let diameter = self.active_radius * 2 + 1;
        diameter * diameter
    }

    pub fn candidate_instance_count(self) -> u32 {
        self.active_region_count() * INSTANCES_PER_REGION
    }

    pub fn dispatch(self) -> [u32; 3] {
        [
            self.active_region_count(),
            INSTANCES_PER_REGION.div_ceil(256),
            1,
        ]
    }

    pub fn json(self) -> Value {
        json!({
            "revision": LOAD_REVISION,
            "config": self,
            "logicalInstanceCount": self.logical_instance_count(),
            "activeRegionCount": self.active_region_count(),
            "candidateInstanceCount": self.candidate_instance_count(),
            "dispatch": self.dispatch(),
            "indirectDrawCount": 1,
            "instancesPerRegion": INSTANCES_PER_REGION,
            "maxRegionSide": MAX_REGION_SIDE
        })
    }
}

pub fn region_semantic(id: u32) -> Option<RegionSemantic> {
    let index = id.checked_sub(REGION_OBJECT_ID_BASE + 1)?;
    if index >= MAX_REGION_SIDE * MAX_REGION_SIDE {
        return None;
    }
    let x = index % MAX_REGION_SIDE;
    let z = index / MAX_REGION_SIDE;
    Some(RegionSemantic {
        name: format!("load.region.{x:03}.{z:03}"),
        kind: "region-proxy".into(),
        color: region_color(x, z),
    })
}

fn region_color(x: u32, z: u32) -> [f32; 4] {
    let relative_x = x as i32 - (MAX_REGION_SIDE / 2) as i32;
    let relative_z = z as i32 - (MAX_REGION_SIDE / 2) as i32;
    let key = ((relative_x + 8) * 17 + (relative_z + 8) * 31) as u32;
    [
        0.25 + 0.7 * ((key * 37) & 255) as f32 / 255.0,
        0.25 + 0.7 * ((key * 73) & 255) as f32 / 255.0,
        0.25 + 0.7 * ((key * 109) & 255) as f32 / 255.0,
        1.0,
    ]
}
