use sha2::{Digest, Sha256};

use super::{ImportedMetadata, ImportedVertexBinding, Vertex, VertexSurface};

const PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/khronos-fox.wlm"));
const HEADER_BYTES: usize = 8 + 32 * 3 + 4 * 4 + 8 * 3;

pub(super) struct ImportedGeometry {
    pub vertices: Vec<Vertex>,
    pub surfaces: Vec<VertexSurface>,
    pub bindings: Vec<ImportedVertexBinding>,
    pub lod_indices: [Vec<u32>; 3],
    pub metadata: ImportedMetadata,
}

pub(super) fn decode() -> Result<ImportedGeometry, String> {
    if PAYLOAD.len() < HEADER_BYTES || &PAYLOAD[..8] != b"WLFOX002" {
        return Err("imported geometry header is invalid".into());
    }
    let mut offset = 8;
    let source_json_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_bin_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let source_texture_sha256 = take_hash(PAYLOAD, &mut offset)?;
    let vertex_count = take_u32(PAYLOAD, &mut offset)? as usize;
    let lod_count = take_u32(PAYLOAD, &mut offset)? as usize;
    let source_joint_count = take_u32(PAYLOAD, &mut offset)?;
    let maximum_joint_depth = take_u32(PAYLOAD, &mut offset)?;
    if vertex_count == 0 || lod_count != 3 || source_joint_count != 24 || maximum_joint_depth > 7 {
        return Err("imported geometry shape is invalid".into());
    }
    let mut lod_index_counts = [0u32; 3];
    let mut lod_errors = [0.0f32; 3];
    for lod in 0..3 {
        lod_index_counts[lod] = take_u32(PAYLOAD, &mut offset)?;
        lod_errors[lod] = take_f32(PAYLOAD, &mut offset)?;
        if lod_index_counts[lod] == 0
            || lod_index_counts[lod] % 3 != 0
            || !lod_errors[lod].is_finite()
            || lod_errors[lod] < 0.0
            || (lod > 0 && lod_index_counts[lod] >= lod_index_counts[lod - 1])
        {
            return Err(format!("imported geometry LOD {lod} metadata is invalid"));
        }
    }
    if lod_errors[0] != 0.0 {
        return Err("imported full-resolution LOD error is not zero".into());
    }

    let mut vertices = Vec::with_capacity(vertex_count);
    let mut surfaces = Vec::with_capacity(vertex_count);
    let mut bindings = Vec::with_capacity(vertex_count);
    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    for index in 0..vertex_count {
        let position = [
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
        ];
        let normal_uv = [
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
            take_f32(PAYLOAD, &mut offset)?,
        ];
        let binding = ImportedVertexBinding {
            indices: take_u32(PAYLOAD, &mut offset)?,
            weights: take_u32(PAYLOAD, &mut offset)?,
        };
        if !position.into_iter().all(f32::is_finite)
            || position[3] != 1.0
            || !normal_uv.into_iter().all(f32::is_finite)
            || normal_uv[0].abs() > 1.0
            || normal_uv[1].abs() > 1.0
            || !(0.0..=1.0).contains(&normal_uv[2])
            || !(0.0..=1.0).contains(&normal_uv[3])
        {
            return Err(format!("imported geometry vertex {index} is invalid"));
        }
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(position[axis]);
            bounds_max[axis] = bounds_max[axis].max(position[axis]);
        }
        vertices.push(Vertex { position });
        surfaces.push(VertexSurface { normal_uv });
        bindings.push(binding);
    }
    if bounds_min[1].abs() > f32::EPSILON || (bounds_max[1] - 1.0).abs() > f32::EPSILON {
        return Err("imported geometry height is not normalized to [0,1]".into());
    }

    let mut lod_indices = [Vec::new(), Vec::new(), Vec::new()];
    for lod in 0..3 {
        let indices = &mut lod_indices[lod];
        indices.reserve(lod_index_counts[lod] as usize);
        for _ in 0..lod_index_counts[lod] {
            let index = take_u32(PAYLOAD, &mut offset)?;
            if index as usize >= vertex_count {
                return Err(format!("imported geometry LOD {lod} index is out of range"));
            }
            indices.push(index);
        }
        if indices.chunks_exact(3).any(|triangle| {
            triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2]
        }) {
            return Err(format!(
                "imported geometry LOD {lod} contains a degenerate triangle"
            ));
        }
    }
    if offset != PAYLOAD.len() {
        return Err("imported geometry payload has trailing bytes".into());
    }

    let cooked_sha256 = hex(Sha256::digest(PAYLOAD));
    Ok(ImportedGeometry {
        vertices,
        surfaces,
        bindings,
        lod_indices,
        metadata: ImportedMetadata {
            revision: "cooked-gltf-geometry-v2-skin",
            source_json_sha256,
            source_bin_sha256,
            source_texture_sha256,
            cooked_sha256,
            vertex_start: 0,
            vertex_count: vertex_count as u32,
            source_joint_count,
            maximum_joint_depth,
            lod_index_counts,
            lod_errors,
            bounds_min,
            bounds_max,
        },
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
