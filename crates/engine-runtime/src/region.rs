use anyhow::Result;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionCoord {
    pub x: i64,
    pub z: i64,
}

impl RegionCoord {
    pub const ZERO: Self = Self { x: 0, z: 0 };

    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }

    pub fn checked_offset(self, x: i64, z: i64) -> Result<Self> {
        Ok(Self {
            x: self
                .x
                .checked_add(x)
                .ok_or_else(|| anyhow::anyhow!("global region X overflows signed 64-bit range"))?,
            z: self
                .z
                .checked_add(z)
                .ok_or_else(|| anyhow::anyhow!("global region Z overflows signed 64-bit range"))?,
        })
    }
}
