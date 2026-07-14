use meshlet_catalog::Catalog as MeshletCatalog;
use sha2::{Digest, Sha256};

pub const MATERIAL_COUNT: u32 = 64;
pub const TEXTURE_SIDE: u32 = 64;
pub const MIP_COUNT: u32 = 7;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceVertex {
    pub oct_normal_uv: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePrimitive {
    pub vertex_indices: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    pub base_color: [f32; 4],
    pub texture_layer: u32,
    pub roughness: f32,
    pub metallic: f32,
    pub reserved: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub vertices: Vec<SurfaceVertex>,
    pub primitives: Vec<SurfacePrimitive>,
    pub materials: Vec<Material>,
    pub texture_mips: Vec<Vec<u8>>,
}

impl Catalog {
    pub fn build(mesh: &MeshletCatalog) -> Self {
        let catalog = Self {
            vertices: build_vertices(mesh),
            primitives: build_primitives(mesh),
            materials: build_materials(),
            texture_mips: build_texture_mips(),
        };
        catalog
            .validate(mesh)
            .expect("generated surface catalog is invalid");
        catalog
    }

    pub fn validate(&self, mesh: &MeshletCatalog) -> Result<(), String> {
        if self.vertices.len() != mesh.vertices.len() {
            return Err("surface vertex count differs from mesh catalog".into());
        }
        if self.primitives.len() != mesh.primitives.len() || self.primitives.len() >= 65_536 {
            return Err("expanded surface primitive count is invalid".into());
        }
        if self.materials.len() != MATERIAL_COUNT as usize {
            return Err("material count is not canonical".into());
        }
        if self.texture_mips.len() != MIP_COUNT as usize {
            return Err("texture mip count is not canonical".into());
        }
        for (index, vertex) in self.vertices.iter().enumerate() {
            let [x, y, u, v] = vertex.oct_normal_uv;
            if ![x, y, u, v].into_iter().all(f32::is_finite)
                || x.abs() > 1.0
                || y.abs() > 1.0
                || !(0.0..=1.0).contains(&u)
                || !(0.0..=1.0).contains(&v)
            {
                return Err(format!("surface vertex {index} is invalid"));
            }
        }
        for (index, primitive) in self.primitives.iter().enumerate() {
            if primitive.vertex_indices[3] != 0
                || primitive.vertex_indices[..3]
                    .iter()
                    .any(|value| *value as usize >= self.vertices.len())
            {
                return Err(format!("surface primitive {index} is invalid"));
            }
        }
        for (index, material) in self.materials.iter().enumerate() {
            if material.texture_layer != index as u32
                || material.reserved != 0
                || !material.base_color.into_iter().all(f32::is_finite)
                || !(0.0..=1.0).contains(&material.roughness)
                || !(0.0..=1.0).contains(&material.metallic)
            {
                return Err(format!("material {index} is invalid"));
            }
        }
        for (mip, bytes) in self.texture_mips.iter().enumerate() {
            let side = (TEXTURE_SIDE >> mip).max(1);
            let expected = MATERIAL_COUNT as usize * side as usize * side as usize * 4;
            if bytes.len() != expected {
                return Err(format!("texture mip {mip} has invalid byte length"));
            }
        }
        Ok(())
    }

    pub fn sha256(&self) -> String {
        let digest = Sha256::digest(self.encoded_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"WLSRF001");
        for count in [
            self.vertices.len(),
            self.primitives.len(),
            self.materials.len(),
            self.texture_mips.len(),
        ] {
            bytes.extend_from_slice(&(count as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&self.vertex_bytes());
        bytes.extend_from_slice(&self.primitive_bytes());
        bytes.extend_from_slice(&self.material_bytes());
        for mip in &self.texture_mips {
            bytes.extend_from_slice(&(mip.len() as u32).to_le_bytes());
            bytes.extend_from_slice(mip);
        }
        bytes
    }

    pub fn vertex_bytes(&self) -> Vec<u8> {
        encode_f32(self.vertices.iter().flat_map(|vertex| vertex.oct_normal_uv))
    }

    pub fn primitive_bytes(&self) -> Vec<u8> {
        encode_u32(
            self.primitives
                .iter()
                .flat_map(|primitive| primitive.vertex_indices),
        )
    }

    pub fn material_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.materials.len() * 32);
        for material in &self.materials {
            bytes.extend_from_slice(&encode_f32(material.base_color));
            bytes.extend_from_slice(&material.texture_layer.to_le_bytes());
            bytes.extend_from_slice(&material.roughness.to_bits().to_le_bytes());
            bytes.extend_from_slice(&material.metallic.to_bits().to_le_bytes());
            bytes.extend_from_slice(&material.reserved.to_le_bytes());
        }
        bytes
    }

    pub fn gpu_bytes(&self) -> usize {
        self.vertices.len() * size_of::<SurfaceVertex>()
            + self.primitives.len() * size_of::<SurfacePrimitive>()
            + self.materials.len() * size_of::<Material>()
            + self.texture_mips.iter().map(Vec::len).sum::<usize>()
    }
}

pub fn decode_octahedral(encoded: [f32; 2]) -> [f32; 3] {
    let mut normal = [
        encoded[0],
        encoded[1],
        1.0 - encoded[0].abs() - encoded[1].abs(),
    ];
    if normal[2] < 0.0 {
        let x = normal[0];
        normal[0] = (1.0 - normal[1].abs()) * if x < 0.0 { -1.0 } else { 1.0 };
        normal[1] = (1.0 - x.abs()) * if normal[1] < 0.0 { -1.0 } else { 1.0 };
    }
    normalize(normal)
}

fn build_vertices(mesh: &MeshletCatalog) -> Vec<SurfaceVertex> {
    mesh.surface_vertices
        .iter()
        .map(|vertex| SurfaceVertex {
            oct_normal_uv: vertex.normal_uv,
        })
        .collect()
}

fn build_primitives(mesh: &MeshletCatalog) -> Vec<SurfacePrimitive> {
    let mut expanded = Vec::with_capacity(mesh.primitives.len());
    for meshlet in &mesh.meshlets {
        for &primitive in &mesh.primitives[meshlet.primitive_offset as usize
            ..(meshlet.primitive_offset + meshlet.primitive_count) as usize]
        {
            let local = [
                primitive & 0xff,
                (primitive >> 8) & 0xff,
                (primitive >> 16) & 0xff,
            ];
            let mut indices = [0u32; 4];
            for (destination, local) in indices[..3].iter_mut().zip(local) {
                *destination = mesh.meshlet_vertices[(meshlet.vertex_offset + local) as usize];
            }
            expanded.push(SurfacePrimitive {
                vertex_indices: indices,
            });
        }
    }
    expanded
}

fn build_materials() -> Vec<Material> {
    (0..MATERIAL_COUNT)
        .map(|index| Material {
            base_color: [
                0.2 + 0.7 * unit_hash(index * 3),
                0.2 + 0.7 * unit_hash(index * 3 + 1),
                0.2 + 0.7 * unit_hash(index * 3 + 2),
                1.0,
            ],
            texture_layer: index,
            roughness: 0.2 + 0.7 * unit_hash(index + 193),
            metallic: 0.1 * (index % 5) as f32,
            reserved: 0,
        })
        .collect()
}

fn build_texture_mips() -> Vec<Vec<u8>> {
    (0..MIP_COUNT)
        .map(|mip| {
            let side = (TEXTURE_SIDE >> mip).max(1);
            let mut bytes =
                Vec::with_capacity(MATERIAL_COUNT as usize * side as usize * side as usize * 4);
            for layer in 0..MATERIAL_COUNT {
                for y in 0..side {
                    for x in 0..side {
                        let pattern = ((x + y + layer + mip) & 3) as u8;
                        let base = (48 + ((layer * 29 + mip * 17) % 160)) as u8;
                        bytes.extend_from_slice(&[
                            base.saturating_add(pattern * 12),
                            base.saturating_add(((x * 5 + layer) & 3) as u8 * 10),
                            base.saturating_add(((y * 7 + mip) & 3) as u8 * 8),
                            255,
                        ]);
                    }
                }
            }
            bytes
        })
        .collect()
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let scale = 1.0
        / value
            .into_iter()
            .map(|axis| axis * axis)
            .sum::<f32>()
            .sqrt();
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn unit_hash(value: u32) -> f32 {
    let mixed = value.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    ((mixed ^ (mixed >> 16)) & 0xffff) as f32 / 65_535.0
}

fn encode_f32(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(|value| value.to_bits().to_le_bytes())
        .collect()
}

fn encode_u32(values: impl IntoIterator<Item = u32>) -> Vec<u8> {
    values.into_iter().flat_map(u32::to_le_bytes).collect()
}

use std::mem::size_of;
