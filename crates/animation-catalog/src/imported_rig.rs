use sha2::{Digest, Sha256};

use crate::{Affine, Bone, ImportedRigMetadata};

const PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/khronos-fox-skin.wla"));
const SOURCE_CLIP_NAMES: [&str; 3] = ["Survey", "Walk", "Run"];

pub(super) struct ImportedRig {
    pub bones: Vec<Bone>,
    pub inverse_bind: Vec<Affine>,
    pub samples: Vec<Affine>,
    pub metadata: ImportedRigMetadata,
}

pub(super) fn decode() -> Result<ImportedRig, String> {
    if PAYLOAD.len() < 152 || &PAYLOAD[..8] != b"WLSKN001" {
        return Err("imported rig header is invalid".into());
    }
    let mut offset = 8;
    let source_json_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_bin_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_joint_count = take_u32(PAYLOAD, &mut offset)?;
    let maximum_joint_depth = take_u32(PAYLOAD, &mut offset)?;
    let source_clip_count = take_u32(PAYLOAD, &mut offset)?;
    let clip_count = take_u32(PAYLOAD, &mut offset)?;
    let sample_count = take_u32(PAYLOAD, &mut offset)?;
    let bone_count = take_u32(PAYLOAD, &mut offset)?;
    if source_joint_count != 24
        || maximum_joint_depth > 7
        || source_clip_count != 3
        || clip_count != 8
        || sample_count != 64
        || bone_count != 128
    {
        return Err("imported rig shape is invalid".into());
    }
    let mut source_clip_durations = [0.0; 3];
    for duration in &mut source_clip_durations {
        *duration = take_f32(PAYLOAD, &mut offset)?;
        if !duration.is_finite() || *duration <= 0.0 {
            return Err("imported rig clip duration is invalid".into());
        }
    }
    let mut source_clip_key_counts = [0; 3];
    for count in &mut source_clip_key_counts {
        *count = take_u32(PAYLOAD, &mut offset)?;
    }
    if source_clip_key_counts != [83, 18, 25] {
        return Err("imported rig source key counts differ".into());
    }
    let mut clip_aliases = [0; 8];
    for alias in &mut clip_aliases {
        *alias = take_u32(PAYLOAD, &mut offset)?;
        if *alias >= source_clip_count {
            return Err("imported rig clip alias is invalid".into());
        }
    }
    if clip_aliases != [0, 1, 2, 0, 1, 2, 0, 1] {
        return Err("imported rig clip aliases differ".into());
    }
    let mut bones = Vec::with_capacity(bone_count as usize);
    for _ in 0..bone_count {
        bones.push(Bone {
            parent: take_u32(PAYLOAD, &mut offset)?,
            depth: take_u32(PAYLOAD, &mut offset)?,
            local_translation: [
                take_f32(PAYLOAD, &mut offset)?,
                take_f32(PAYLOAD, &mut offset)?,
                take_f32(PAYLOAD, &mut offset)?,
            ],
            reserved: take_f32(PAYLOAD, &mut offset)?,
        });
    }
    let mut inverse_bind = Vec::with_capacity(bone_count as usize);
    for _ in 0..bone_count {
        inverse_bind.push(take_affine(PAYLOAD, &mut offset)?);
    }
    let mut samples = Vec::with_capacity((clip_count * sample_count * bone_count) as usize);
    for _ in 0..clip_count * sample_count * bone_count {
        samples.push(take_affine(PAYLOAD, &mut offset)?);
    }
    if offset != PAYLOAD.len() {
        return Err("imported rig payload has trailing bytes".into());
    }
    Ok(ImportedRig {
        bones,
        inverse_bind,
        samples,
        metadata: ImportedRigMetadata {
            revision: "cooked-gltf-skeletal-animation-v1",
            source_json_sha256,
            source_bin_sha256,
            cooked_sha256: hex(Sha256::digest(PAYLOAD)),
            source_joint_count,
            maximum_joint_depth,
            source_clip_names: SOURCE_CLIP_NAMES,
            source_clip_durations,
            source_clip_key_counts,
            clip_aliases,
        },
    })
}

fn take_affine(bytes: &[u8], offset: &mut usize) -> Result<Affine, String> {
    let mut rows = [[0.0; 4]; 3];
    for row in &mut rows {
        for value in row {
            *value = take_f32(bytes, offset)?;
            if !value.is_finite() {
                return Err("imported rig affine contains a non-finite value".into());
            }
        }
    }
    Ok(Affine { rows })
}

fn take_hash(bytes: &[u8], offset: &mut usize) -> Result<String, String> {
    let end = offset.checked_add(32).ok_or("imported rig hash overflow")?;
    let value = bytes
        .get(*offset..end)
        .ok_or("imported rig hash exceeds payload")?;
    *offset = end;
    Ok(hex(value))
}

fn take_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let end = offset.checked_add(4).ok_or("imported rig u32 overflow")?;
    let value = bytes
        .get(*offset..end)
        .ok_or("imported rig u32 exceeds payload")?;
    *offset = end;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn take_f32(bytes: &[u8], offset: &mut usize) -> Result<f32, String> {
    Ok(f32::from_bits(take_u32(bytes, offset)?))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
