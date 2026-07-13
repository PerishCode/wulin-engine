use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::address::AddressedRegion;
use crate::world::RegionCoord;

use super::{CacheKey, DesiredRegion, RegionAssignment};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ObjectSourceNamespace([u8; 32]);

impl ObjectSourceNamespace {
    pub fn from_revision(revision: &str) -> Self {
        Self(Sha256::digest(revision.as_bytes()).into())
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
) -> DesiredRegion {
    DesiredRegion {
        key: CacheKey::CanonicalGlobal {
            source_namespace,
            global_region: region.global_region,
        },
        assignment: RegionAssignment {
            slot: 0,
            region_id: region.local_region_id,
            global_region: Some(region.global_region),
            stable_seed: Some(canonical_stable_seed(
                source_namespace,
                region.global_region,
            )),
        },
    }
}

pub fn canonical_stable_seed(source_namespace: ObjectSourceNamespace, region: RegionCoord) -> u32 {
    let mut digest = Sha256::new();
    digest.update(source_namespace.as_bytes());
    digest.update(region.x.to_le_bytes());
    digest.update(region.z.to_le_bytes());
    let hash = digest.finalize();
    u32::from_le_bytes(hash[..4].try_into().expect("SHA-256 prefix has four bytes"))
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
