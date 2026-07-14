use anyhow::{Result, bail};
use region_format::{InstanceRecord, RECORDS_PER_REGION};

pub const IDENTITY_A_REVISION: &str = "canonical-object-identity-order-a-v1";
pub const IDENTITY_B_REVISION: &str = "canonical-object-identity-order-b-v1";

#[derive(Clone, Copy)]
pub enum IdentityOrder {
    A,
    B,
}

impl IdentityOrder {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            _ => bail!("identity order must be a or b"),
        }
    }

    pub const fn revision(self) -> &'static str {
        match self {
            Self::A => IDENTITY_A_REVISION,
            Self::B => IDENTITY_B_REVISION,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }
}

pub fn reorder_identity_records(
    records: Vec<InstanceRecord>,
    order: IdentityOrder,
) -> (Vec<InstanceRecord>, Vec<u32>) {
    assert_eq!(records.len(), RECORDS_PER_REGION as usize);
    let (multiplier, offset) = match order {
        IdentityOrder::A => (769_u32, 73_u32),
        IdentityOrder::B => (641_u32, 419_u32),
    };
    let mut reordered = Vec::with_capacity(records.len());
    let mut local_ids = Vec::with_capacity(records.len());
    for index in 0..RECORDS_PER_REGION {
        let local_id = index.wrapping_mul(multiplier).wrapping_add(offset) % RECORDS_PER_REGION;
        reordered.push(records[local_id as usize]);
        local_ids.push(local_id);
    }
    (reordered, local_ids)
}
