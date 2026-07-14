use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use gltf::mesh::Mode;
use meshopt::{SimplifyOptions, simplify_decoder};
use sha2::{Digest, Sha256};

const JSON_SHA256: &str = "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002";
const BIN_SHA256: &str = "c7d0d8de28a84d5b25623037f88e063e1502495a2ee6c55f182c61161ad12f80";
const TEXTURE_SHA256: &str = "61c8b109ee7f8bf262791933380fafb1465f7b51cbe6472c2d21eff0b31f83a1";

#[derive(Clone, Copy)]
struct SourceVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

#[derive(Clone, Copy)]
struct CookedVertex {
    position: [f32; 4],
    normal_uv: [f32; 4],
}

struct CookedLod {
    indices: Vec<u32>,
    error: f32,
}

fn main() {
    if let Err(error) = cook() {
        panic!("failed to cook the pinned Fox geometry: {error}");
    }
}

fn cook() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest")?);
    let source = manifest.join("../../assets/third-party/khronos-fox");
    let json_path = source.join("Fox.gltf");
    let bin_path = source.join("Fox.bin");
    let texture_path = source.join("Texture.png");
    for path in [&json_path, &bin_path, &texture_path] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let json = verified_read(&json_path, JSON_SHA256)?;
    let bin = verified_read(&bin_path, BIN_SHA256)?;
    let _texture = verified_read(&texture_path, TEXTURE_SHA256)?;
    let gltf = gltf::Gltf::from_slice(&json)?;
    if gltf.document.scenes().count() != 1 || gltf.document.meshes().count() != 1 {
        return Err("source must contain exactly one scene and one mesh".into());
    }
    let mesh = gltf
        .document
        .meshes()
        .next()
        .ok_or("source mesh is missing")?;
    if mesh.primitives().count() != 1 {
        return Err("source mesh must contain exactly one primitive".into());
    }
    let mesh_nodes = gltf
        .document
        .nodes()
        .filter(|node| node.mesh().is_some())
        .collect::<Vec<_>>();
    if mesh_nodes.len() != 1
        || mesh_nodes[0].mesh().map(|value| value.index()) != Some(mesh.index())
    {
        return Err("source mesh must have exactly one node owner".into());
    }
    if mesh_nodes[0].transform().matrix() != identity_matrix() {
        return Err("source mesh node transform must be identity".into());
    }

    let primitive = mesh.primitives().next().unwrap();
    if primitive.mode() != Mode::Triangles || primitive.morph_targets().next().is_some() {
        return Err("source primitive must be non-morphed triangles".into());
    }
    let reader = primitive.reader(|buffer| (buffer.index() == 0).then_some(bin.as_slice()));
    let positions = reader
        .read_positions()
        .ok_or("source primitive has no positions")?
        .collect::<Vec<_>>();
    let uvs = reader
        .read_tex_coords(0)
        .ok_or("source primitive has no UV0")?
        .into_f32()
        .collect::<Vec<_>>();
    if positions.len() != uvs.len() || positions.is_empty() {
        return Err("source position and UV streams differ or are empty".into());
    }
    let source_indices = reader
        .read_indices()
        .map(|values| values.into_u32().collect::<Vec<_>>())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());
    if source_indices.len() % 3 != 0
        || source_indices
            .iter()
            .any(|index| *index as usize >= positions.len())
    {
        return Err("source triangle indices are invalid".into());
    }

    let source_vertices = positions
        .into_iter()
        .zip(uvs)
        .map(|(position, uv)| SourceVertex { position, uv })
        .collect::<Vec<_>>();
    let (vertices, indices) = normalize_and_deduplicate(&source_vertices, &source_indices)?;
    let lods = build_lods(&vertices, &indices)?;
    let payload = encode_payload(&vertices, &lods);
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    fs::write(out.join("khronos-fox.wlm"), payload)?;
    Ok(())
}

fn verified_read(path: &Path, expected: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let actual = hex(Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "{} SHA-256 {actual} differs from {expected}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn normalize_and_deduplicate(
    source: &[SourceVertex],
    source_indices: &[u32],
) -> Result<(Vec<CookedVertex>, Vec<u32>), Box<dyn Error>> {
    if source.iter().any(|vertex| {
        !vertex.position.into_iter().all(f32::is_finite)
            || !vertex.uv.into_iter().all(f32::is_finite)
    }) {
        return Err("source contains non-finite geometry".into());
    }
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for vertex in source {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(vertex.position[axis]);
            maximum[axis] = maximum[axis].max(vertex.position[axis]);
        }
    }
    let height = maximum[1] - minimum[1];
    if !height.is_finite() || height <= f32::EPSILON {
        return Err("source height is invalid".into());
    }
    let center_x = (minimum[0] + maximum[0]) * 0.5;
    let center_z = (minimum[2] + maximum[2]) * 0.5;
    let scale = height.recip();

    let mut unique = BTreeMap::<[u32; 5], u32>::new();
    let mut positions = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut indices = Vec::with_capacity(source_indices.len());
    for source_index in source_indices {
        let vertex = source[*source_index as usize];
        let position = [
            (vertex.position[0] - center_x) * scale,
            (vertex.position[1] - minimum[1]) * scale,
            (vertex.position[2] - center_z) * scale,
        ];
        let key = [
            position[0].to_bits(),
            position[1].to_bits(),
            position[2].to_bits(),
            vertex.uv[0].to_bits(),
            vertex.uv[1].to_bits(),
        ];
        let index = *unique.entry(key).or_insert_with(|| {
            let index = positions.len() as u32;
            positions.push(position);
            uvs.push(vertex.uv);
            index
        });
        indices.push(index);
    }
    let normals = generate_normals(&positions, &indices)?;
    let vertices = positions
        .into_iter()
        .zip(normals)
        .zip(uvs)
        .map(|((position, normal), uv)| CookedVertex {
            position: [position[0], position[1], position[2], 1.0],
            normal_uv: [
                encode_octahedral(normal)[0],
                encode_octahedral(normal)[1],
                uv[0],
                uv[1],
            ],
        })
        .collect();
    Ok((vertices, indices))
}

fn generate_normals(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Result<Vec<[f32; 3]>, Box<dyn Error>> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let [a, b, c] = [
            positions[triangle[0] as usize],
            positions[triangle[1] as usize],
            positions[triangle[2] as usize],
        ];
        let edge_a = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let edge_b = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let face = [
            edge_a[1] * edge_b[2] - edge_a[2] * edge_b[1],
            edge_a[2] * edge_b[0] - edge_a[0] * edge_b[2],
            edge_a[0] * edge_b[1] - edge_a[1] * edge_b[0],
        ];
        for index in triangle {
            for axis in 0..3 {
                normals[*index as usize][axis] += face[axis];
            }
        }
    }
    for normal in &mut normals {
        *normal =
            normalize(*normal).ok_or("source contains an unreferenced or zero-area vertex")?;
    }
    Ok(normals)
}

fn build_lods(
    vertices: &[CookedVertex],
    indices: &[u32],
) -> Result<Vec<CookedLod>, Box<dyn Error>> {
    let positions = vertices
        .iter()
        .map(|vertex| [vertex.position[0], vertex.position[1], vertex.position[2]])
        .collect::<Vec<_>>();
    let mut lods = vec![CookedLod {
        indices: indices.to_vec(),
        error: 0.0,
    }];
    for divisor in [2, 4] {
        let mut error = 0.0;
        let target = (indices.len() / divisor / 3).max(1) * 3;
        let simplified = simplify_decoder(
            indices,
            &positions,
            target,
            1.0,
            SimplifyOptions::Permissive | SimplifyOptions::Prune,
            Some(&mut error),
        );
        if simplified.is_empty()
            || simplified.len() % 3 != 0
            || simplified.len() >= lods.last().unwrap().indices.len()
        {
            return Err(format!("LOD {divisor} simplification did not reduce triangles").into());
        }
        lods.push(CookedLod {
            indices: simplified,
            error,
        });
    }
    Ok(lods)
}

fn encode_payload(vertices: &[CookedVertex], lods: &[CookedLod]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"WLFOX001");
    bytes.extend_from_slice(&decode_hex(JSON_SHA256));
    bytes.extend_from_slice(&decode_hex(BIN_SHA256));
    bytes.extend_from_slice(&decode_hex(TEXTURE_SHA256));
    bytes.extend_from_slice(&(vertices.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(lods.len() as u32).to_le_bytes());
    for lod in lods {
        bytes.extend_from_slice(&(lod.indices.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&lod.error.to_bits().to_le_bytes());
    }
    for vertex in vertices {
        for value in vertex.position.into_iter().chain(vertex.normal_uv) {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    for lod in lods {
        for index in &lod.indices {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
    }
    bytes
}

fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn normalize(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = value
        .into_iter()
        .map(|axis| axis * axis)
        .sum::<f32>()
        .sqrt();
    (length > f32::EPSILON).then(|| [value[0] / length, value[1] / length, value[2] / length])
}

fn encode_octahedral(normal: [f32; 3]) -> [f32; 2] {
    let scale = 1.0 / (normal[0].abs() + normal[1].abs() + normal[2].abs());
    let mut encoded = [normal[0] * scale, normal[1] * scale];
    if normal[2] < 0.0 {
        let x = encoded[0];
        encoded[0] = (1.0 - encoded[1].abs()) * x.signum();
        encoded[1] = (1.0 - x.abs()) * encoded[1].signum();
    }
    encoded
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_hex(value: &str) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for (index, output) in bytes.iter_mut().enumerate() {
        *output = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).unwrap();
    }
    bytes
}
