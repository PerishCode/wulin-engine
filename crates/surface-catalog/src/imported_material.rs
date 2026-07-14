use sha2::{Digest, Sha256};

use super::{IMPORTED_MATERIAL, ImportedMaterialMetadata, MIP_COUNT, TEXTURE_SIDE};

const PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/khronos-fox-material.wltm"));
const HEADER_BYTES: usize = 8 + 32 * 2 + 4 * 4 + 4 * 6 + 4 * MIP_COUNT as usize;

pub(super) struct ImportedMaterial {
    pub metadata: ImportedMaterialMetadata,
    pub mips: Vec<Vec<u8>>,
}

pub(super) fn decode() -> Result<ImportedMaterial, String> {
    if PAYLOAD.len() < HEADER_BYTES || &PAYLOAD[..8] != b"WLFTX001" {
        return Err("imported material header is invalid".into());
    }
    let mut offset = 8;
    let source_json_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_texture_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_size = [
        take_u32(PAYLOAD, &mut offset)?,
        take_u32(PAYLOAD, &mut offset)?,
    ];
    let texture_side = take_u32(PAYLOAD, &mut offset)?;
    let mip_count = take_u32(PAYLOAD, &mut offset)?;
    let base_color = [
        take_f32(PAYLOAD, &mut offset)?,
        take_f32(PAYLOAD, &mut offset)?,
        take_f32(PAYLOAD, &mut offset)?,
        take_f32(PAYLOAD, &mut offset)?,
    ];
    let roughness = take_f32(PAYLOAD, &mut offset)?;
    let metallic = take_f32(PAYLOAD, &mut offset)?;
    if source_size != [1024, 1024]
        || texture_side != TEXTURE_SIDE
        || mip_count != MIP_COUNT
        || base_color.map(f32::to_bits) != [1.0f32.to_bits(); 4]
        || roughness.to_bits() != 0.58f32.to_bits()
        || metallic.to_bits() != 0.0f32.to_bits()
    {
        return Err("imported material shape or factors are invalid".into());
    }

    let mut mip_sizes = [0; MIP_COUNT as usize];
    for (mip, size) in mip_sizes.iter_mut().enumerate() {
        *size = take_u32(PAYLOAD, &mut offset)?;
        let side = (TEXTURE_SIDE >> mip).max(1);
        if *size != side * side * 4 {
            return Err(format!("imported material mip {mip} size is invalid"));
        }
    }
    let mut mips = Vec::with_capacity(MIP_COUNT as usize);
    for (mip, size) in mip_sizes.into_iter().enumerate() {
        let end = offset
            .checked_add(size as usize)
            .ok_or("imported material mip offset overflow")?;
        let bytes = PAYLOAD
            .get(offset..end)
            .ok_or("imported material mip exceeds payload")?
            .to_vec();
        if bytes.chunks_exact(4).any(|pixel| pixel[3] != 255) {
            return Err(format!("imported material mip {mip} alpha is not opaque"));
        }
        offset = end;
        mips.push(bytes);
    }
    if offset != PAYLOAD.len() {
        return Err("imported material payload has trailing bytes".into());
    }
    let mip_sha256 = std::array::from_fn(|mip| hex(Sha256::digest(&mips[mip])));
    Ok(ImportedMaterial {
        metadata: ImportedMaterialMetadata {
            revision: "cooked-gltf-material-v1",
            source_json_sha256,
            source_texture_sha256,
            cooked_sha256: hex(Sha256::digest(PAYLOAD)),
            material_index: IMPORTED_MATERIAL,
            texture_layer: IMPORTED_MATERIAL,
            source_size,
            texture_side,
            mip_sizes,
            mip_sha256,
            base_color,
            roughness,
            metallic,
        },
        mips,
    })
}

fn take_hash(bytes: &[u8], offset: &mut usize) -> Result<String, String> {
    let end = offset
        .checked_add(32)
        .ok_or("imported hash offset overflow")?;
    let value = bytes
        .get(*offset..end)
        .ok_or("imported hash exceeds payload")?;
    *offset = end;
    Ok(hex(value))
}

fn take_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or("imported u32 offset overflow")?;
    let value = bytes
        .get(*offset..end)
        .ok_or("imported u32 exceeds payload")?;
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
