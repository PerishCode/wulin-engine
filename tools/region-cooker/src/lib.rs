use anyhow::{Result, bail};
use region_format::{
    InstanceRecord, PRESENTATION_ANIMATION_CLIP_COUNT, PRESENTATION_ANIMATION_PHASE_COUNT,
    PRESENTATION_ARCHETYPE_COUNT, PRESENTATION_MATERIAL_COUNT, PresentationRecord,
    RECORDS_PER_REGION,
};

pub const ORDER_A_REVISION: &str = "authored-object-presentation-order-a-v1";
pub const ORDER_B_REVISION: &str = "authored-object-presentation-order-b-v1";

#[derive(Clone, Copy)]
pub enum PhysicalOrder {
    A,
    B,
}

impl PhysicalOrder {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            _ => bail!("physical order must be a or b"),
        }
    }

    pub const fn revision(self) -> &'static str {
        match self {
            Self::A => ORDER_A_REVISION,
            Self::B => ORDER_B_REVISION,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }
}

#[derive(Clone, Copy)]
pub enum PresentationProfile {
    Base,
    Archetype,
    Material,
    Yaw,
    Animation,
    Imported,
}

impl PresentationProfile {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "base" => Ok(Self::Base),
            "archetype" => Ok(Self::Archetype),
            "material" => Ok(Self::Material),
            "yaw" => Ok(Self::Yaw),
            "animation" => Ok(Self::Animation),
            "imported" => Ok(Self::Imported),
            _ => bail!(
                "presentation profile must be base, archetype, material, yaw, animation, or imported"
            ),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Archetype => "archetype",
            Self::Material => "material",
            Self::Yaw => "yaw",
            Self::Animation => "animation",
            Self::Imported => "imported",
        }
    }
}

pub fn author_presentations(
    records: &[InstanceRecord],
    profile: PresentationProfile,
) -> Vec<PresentationRecord> {
    assert_eq!(records.len(), RECORDS_PER_REGION as usize);
    records
        .iter()
        .enumerate()
        .map(|(local_id, record)| {
            let key = record.region_id ^ (local_id as u32).wrapping_mul(747_796_405);
            let mut presentation = if key & 3 == 0 {
                PresentationRecord::static_object(
                    key % PRESENTATION_ARCHETYPE_COUNT,
                    key.rotate_left(11) % PRESENTATION_MATERIAL_COUNT,
                    key.wrapping_mul(2_891_336_453) & 0xffff,
                )
            } else {
                PresentationRecord::animated(
                    key % PRESENTATION_ARCHETYPE_COUNT,
                    key.rotate_left(11) % PRESENTATION_MATERIAL_COUNT,
                    key.wrapping_mul(2_891_336_453) & 0xffff,
                    key.rotate_left(5) % PRESENTATION_ANIMATION_CLIP_COUNT,
                    key.rotate_right(9) % PRESENTATION_ANIMATION_PHASE_COUNT,
                    key.rotate_left(17) & 0xffff,
                )
            };
            match profile {
                PresentationProfile::Base => {}
                PresentationProfile::Archetype => {
                    presentation.archetype =
                        (presentation.archetype + 1) % PRESENTATION_ARCHETYPE_COUNT;
                }
                PresentationProfile::Material => {
                    presentation.material =
                        (presentation.material + 1) % PRESENTATION_MATERIAL_COUNT;
                }
                PresentationProfile::Yaw => {
                    presentation.yaw_q16 = (presentation.yaw_q16 + 16_384) & 0xffff;
                }
                PresentationProfile::Animation => {
                    presentation.animation = if presentation.is_animated() {
                        region_format::STATIC_PRESENTATION_ANIMATION
                    } else {
                        let clip = key.rotate_left(5) % PRESENTATION_ANIMATION_CLIP_COUNT;
                        let phase = key.rotate_right(9) % PRESENTATION_ANIMATION_PHASE_COUNT;
                        (key.rotate_left(17) & 0xffff) << 16 | phase << 8 | clip
                    };
                }
                PresentationProfile::Imported => {
                    presentation.archetype = PRESENTATION_ARCHETYPE_COUNT - 1;
                    presentation.material = PRESENTATION_MATERIAL_COUNT - 1;
                }
            }
            presentation
        })
        .collect()
}

pub fn reorder_object_triples(
    records: Vec<InstanceRecord>,
    presentations: Vec<PresentationRecord>,
    order: PhysicalOrder,
) -> (Vec<InstanceRecord>, Vec<u32>, Vec<PresentationRecord>) {
    assert_eq!(records.len(), RECORDS_PER_REGION as usize);
    assert_eq!(presentations.len(), RECORDS_PER_REGION as usize);
    let (multiplier, offset) = match order {
        PhysicalOrder::A => (769_u32, 73_u32),
        PhysicalOrder::B => (641_u32, 419_u32),
    };
    let mut reordered_records = Vec::with_capacity(records.len());
    let mut local_ids = Vec::with_capacity(records.len());
    let mut reordered_presentations = Vec::with_capacity(records.len());
    for index in 0..RECORDS_PER_REGION {
        let local_id = index.wrapping_mul(multiplier).wrapping_add(offset) % RECORDS_PER_REGION;
        reordered_records.push(records[local_id as usize]);
        local_ids.push(local_id);
        reordered_presentations.push(presentations[local_id as usize]);
    }
    (reordered_records, local_ids, reordered_presentations)
}
