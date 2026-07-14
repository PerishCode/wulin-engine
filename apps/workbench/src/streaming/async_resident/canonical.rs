use serde::{Serialize, Serializer};

use crate::address::AddressedRegion;
use crate::world::RegionCoord;

use super::{CacheKey, DesiredRegion, RegionAssignment};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectSourceNamespace([u8; 32]);

impl ObjectSourceNamespace {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Serialize for ObjectSourceNamespace {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex(&self.0))
    }
}

pub(super) fn desired(
    region: AddressedRegion,
    source_namespace: ObjectSourceNamespace,
    stable_seed_namespace: ObjectSourceNamespace,
) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey {
            source_namespace,
            global_region: region.global_region,
        },
        assignment: RegionAssignment {
            slot: 0,
            region_id: region.local_region_id,
            global_region: region.global_region,
            stable_seed: canonical_stable_seed(stable_seed_namespace, region.global_region),
        },
    }
}

pub fn canonical_stable_seed(source_namespace: ObjectSourceNamespace, region: RegionCoord) -> u32 {
    region_format::canonical_stable_seed(
        *source_namespace.as_bytes(),
        region_format::GlobalRegion::new(region.x, region.z),
    )
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut value, byte| {
            write!(&mut value, "{byte:02x}").expect("string formatting cannot fail");
            value
        })
}
