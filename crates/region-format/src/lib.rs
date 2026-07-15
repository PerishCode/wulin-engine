use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::Result;
use anyhow::ensure;
use serde::Serialize;
use sha2::{Digest, Sha256};

mod global;

pub use global::{
    GLOBAL_HEADER_BYTES, GLOBAL_INDEX_ENTRY_BYTES, GLOBAL_MAGIC, GLOBAL_PAYLOAD_SCHEMA,
    GLOBAL_REGION_BYTES, GLOBAL_VERSION, GlobalPackMetadata, GlobalRegion, GlobalRegionPack,
    GlobalRegionRead, IDENTITY_BYTES, IDENTITY_PLANE_BYTES, PRESENTATION_PLANE_BYTES,
    canonical_stable_seed, write_global_pack,
};

pub const RECORDS_PER_REGION: u32 = 1_024;
pub const RECORD_BYTES: u32 = 20;
pub const REGION_BYTES: u32 = RECORDS_PER_REGION * RECORD_BYTES;
pub const PAYLOAD_ALIGNMENT: u64 = 4_096;
pub const PRESENTATION_BYTES: u32 = 16;
pub const PRESENTATION_ARCHETYPE_COUNT: u32 = 8;
pub const PRESENTATION_MATERIAL_COUNT: u32 = 64;
pub const PRESENTATION_ANIMATION_CLIP_COUNT: u32 = 8;
pub const PRESENTATION_ANIMATION_PHASE_COUNT: u32 = 64;
pub const STATIC_PRESENTATION_ANIMATION: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceRecord {
    pub position: [f32; 3],
    pub height: f32,
    pub region_id: u32,
}

const _: [(); RECORD_BYTES as usize] = [(); size_of::<InstanceRecord>()];

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationRecord {
    pub archetype: u32,
    pub material: u32,
    pub yaw_q16: u32,
    pub animation: u32,
}

const _: [(); PRESENTATION_BYTES as usize] = [(); size_of::<PresentationRecord>()];

impl PresentationRecord {
    pub const fn static_object(archetype: u32, material: u32, yaw_q16: u32) -> Self {
        Self {
            archetype,
            material,
            yaw_q16,
            animation: STATIC_PRESENTATION_ANIMATION,
        }
    }

    pub const fn animated(
        archetype: u32,
        material: u32,
        yaw_q16: u32,
        clip: u32,
        phase_offset: u32,
        variant: u32,
    ) -> Self {
        Self {
            archetype,
            material,
            yaw_q16,
            animation: (variant << 16) | (phase_offset << 8) | clip,
        }
    }

    pub const fn is_animated(self) -> bool {
        self.animation != STATIC_PRESENTATION_ANIMATION
    }

    pub const fn animation_clip(self) -> Option<u32> {
        if self.is_animated() {
            Some(self.animation & 0xff)
        } else {
            None
        }
    }

    pub const fn animation_phase_offset(self) -> Option<u32> {
        if self.is_animated() {
            Some((self.animation >> 8) & 0xff)
        } else {
            None
        }
    }

    pub const fn animation_variant(self) -> Option<u32> {
        if self.is_animated() {
            Some(self.animation >> 16)
        } else {
            None
        }
    }

    pub fn validate(self) -> Result<()> {
        ensure!(
            self.archetype < PRESENTATION_ARCHETYPE_COUNT,
            "presentation archetype {} exceeds catalog capacity",
            self.archetype
        );
        ensure!(
            self.material < PRESENTATION_MATERIAL_COUNT,
            "presentation material {} exceeds catalog capacity",
            self.material
        );
        ensure!(
            self.yaw_q16 <= u16::MAX.into(),
            "presentation yaw {} exceeds Q16 range",
            self.yaw_q16
        );
        if self.is_animated() {
            ensure!(
                self.animation_clip().unwrap() < PRESENTATION_ANIMATION_CLIP_COUNT,
                "presentation animation clip exceeds catalog capacity"
            );
            ensure!(
                self.animation_phase_offset().unwrap() < PRESENTATION_ANIMATION_PHASE_COUNT,
                "presentation animation phase exceeds catalog capacity"
            );
        }
        Ok(())
    }
}

fn encode_record(record: &InstanceRecord, output: &mut Vec<u8>) {
    for value in record.position {
        output.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    output.extend_from_slice(&record.height.to_bits().to_le_bytes());
    output.extend_from_slice(&record.region_id.to_le_bytes());
}

fn decode_record(bytes: &[u8]) -> InstanceRecord {
    InstanceRecord {
        position: [
            f32::from_bits(u32_at(bytes, 0)),
            f32::from_bits(u32_at(bytes, 4)),
            f32::from_bits(u32_at(bytes, 8)),
        ],
        height: f32::from_bits(u32_at(bytes, 12)),
        region_id: u32_at(bytes, 16),
    }
}

fn encode_presentation(record: &PresentationRecord, output: &mut Vec<u8>) {
    output.extend_from_slice(&record.archetype.to_le_bytes());
    output.extend_from_slice(&record.material.to_le_bytes());
    output.extend_from_slice(&record.yaw_q16.to_le_bytes());
    output.extend_from_slice(&record.animation.to_le_bytes());
}

fn decode_presentation(bytes: &[u8]) -> PresentationRecord {
    PresentationRecord {
        archetype: u32_at(bytes, 0),
        material: u32_at(bytes, 4),
        yaw_q16: u32_at(bytes, 8),
        animation: u32_at(bytes, 12),
    }
}

pub fn file_sha256(path: impl AsRef<Path>) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut bytes = [0u8; 64 * 1_024];
    loop {
        let read = file.read(&mut bytes)?;
        if read == 0 {
            break;
        }
        hash.update(&bytes[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn align_up(value: u64) -> u64 {
    value.div_ceil(PAYLOAD_ALIGNMENT) * PAYLOAD_ALIGNMENT
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("u64 slice"))
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32_to(writer: &mut impl Write, value: u32) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn push_u64_to(writer: &mut impl Write, value: u64) -> Result<()> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
