mod imported;
mod procedural;

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

pub const ARCHETYPE_COUNT: u32 = 8;
pub const LOD_COUNT: u32 = 3;
pub const IMPORTED_ARCHETYPE: u32 = 7;
pub const MAX_MESHLET_VERTICES: u32 = 64;
pub const MAX_MESHLET_PRIMITIVES: u32 = 126;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexSurface {
    pub normal_uv: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Meshlet {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub primitive_offset: u32,
    pub primitive_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lod {
    pub meshlet_offset: u32,
    pub meshlet_count: u32,
    pub vertex_count: u32,
    pub primitive_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedMetadata {
    pub revision: &'static str,
    pub source_json_sha256: String,
    pub source_bin_sha256: String,
    pub source_texture_sha256: String,
    pub cooked_sha256: String,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub lod_index_counts: [u32; 3],
    pub lod_errors: [f32; 3],
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub vertices: Vec<Vertex>,
    pub surface_vertices: Vec<VertexSurface>,
    pub meshlets: Vec<Meshlet>,
    pub meshlet_vertices: Vec<u32>,
    pub primitives: Vec<u32>,
    pub lods: Vec<Lod>,
    pub imported: ImportedMetadata,
}

impl Catalog {
    pub fn build() -> Self {
        let mut catalog = Self {
            vertices: Vec::new(),
            surface_vertices: Vec::new(),
            meshlets: Vec::new(),
            meshlet_vertices: Vec::new(),
            primitives: Vec::new(),
            lods: Vec::with_capacity((ARCHETYPE_COUNT * LOD_COUNT) as usize),
            imported: ImportedMetadata {
                revision: "uninitialized",
                source_json_sha256: String::new(),
                source_bin_sha256: String::new(),
                source_texture_sha256: String::new(),
                cooked_sha256: String::new(),
                vertex_start: 0,
                vertex_count: 0,
                lod_index_counts: [0; 3],
                lod_errors: [0.0; 3],
                bounds_min: [0.0; 3],
                bounds_max: [0.0; 3],
            },
        };
        for archetype in 0..IMPORTED_ARCHETYPE {
            for lod in 0..LOD_COUNT {
                procedural::append_lod(&mut catalog, archetype, lod);
            }
        }
        append_imported(&mut catalog).expect("imported catalog is invalid");
        catalog.validate().expect("generated catalog is invalid");
        catalog
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lods.len() != (ARCHETYPE_COUNT * LOD_COUNT) as usize {
            return Err("catalog LOD count is not canonical".into());
        }
        if self.surface_vertices.len() != self.vertices.len() {
            return Err("catalog surface vertex count differs".into());
        }
        for (index, vertex) in self.vertices.iter().enumerate() {
            if !vertex.position.into_iter().all(f32::is_finite) || vertex.position[3] != 1.0 {
                return Err(format!("catalog vertex {index} is invalid"));
            }
        }
        for (index, surface) in self.surface_vertices.iter().enumerate() {
            let [x, y, u, v] = surface.normal_uv;
            if ![x, y, u, v].into_iter().all(f32::is_finite)
                || x.abs() > 1.0
                || y.abs() > 1.0
                || !(0.0..=1.0).contains(&u)
                || !(0.0..=1.0).contains(&v)
            {
                return Err(format!("catalog surface vertex {index} is invalid"));
            }
        }
        for (index, meshlet) in self.meshlets.iter().enumerate() {
            if meshlet.vertex_count == 0 || meshlet.vertex_count > MAX_MESHLET_VERTICES {
                return Err(format!("meshlet {index} vertex bound is invalid"));
            }
            if meshlet.primitive_count == 0 || meshlet.primitive_count > MAX_MESHLET_PRIMITIVES {
                return Err(format!("meshlet {index} primitive bound is invalid"));
            }
            let vertex_end = meshlet.vertex_offset as usize + meshlet.vertex_count as usize;
            let primitive_end =
                meshlet.primitive_offset as usize + meshlet.primitive_count as usize;
            if vertex_end > self.meshlet_vertices.len() || primitive_end > self.primitives.len() {
                return Err(format!("meshlet {index} range exceeds its buffer"));
            }
            for &vertex in &self.meshlet_vertices[meshlet.vertex_offset as usize..vertex_end] {
                if vertex as usize >= self.vertices.len() {
                    return Err(format!("meshlet {index} references an invalid vertex"));
                }
            }
            for &primitive in &self.primitives[meshlet.primitive_offset as usize..primitive_end] {
                let indices = [
                    primitive & 0xff,
                    (primitive >> 8) & 0xff,
                    (primitive >> 16) & 0xff,
                ];
                if indices.iter().any(|value| *value >= meshlet.vertex_count) {
                    return Err(format!("meshlet {index} has an invalid local primitive"));
                }
            }
        }
        for archetype in 0..ARCHETYPE_COUNT {
            let mut previous: Option<Lod> = None;
            for lod in 0..LOD_COUNT {
                let descriptor = self.lod(archetype, lod);
                if descriptor.meshlet_count == 0
                    || descriptor.meshlet_offset + descriptor.meshlet_count
                        > self.meshlets.len() as u32
                {
                    return Err(format!("archetype {archetype} LOD {lod} range is invalid"));
                }
                if let Some(previous) = previous
                    && (descriptor.vertex_count >= previous.vertex_count
                        || descriptor.primitive_count >= previous.primitive_count)
                {
                    return Err(format!(
                        "archetype {archetype} LOD {lod} does not reduce geometry"
                    ));
                }
                previous = Some(*descriptor);
            }
        }
        if self.imported.revision != "cooked-gltf-geometry-v1"
            || self.imported.vertex_count == 0
            || self.imported.vertex_start + self.imported.vertex_count > self.vertices.len() as u32
            || self.imported.source_json_sha256.len() != 64
            || self.imported.source_bin_sha256.len() != 64
            || self.imported.source_texture_sha256.len() != 64
            || self.imported.cooked_sha256.len() != 64
        {
            return Err("imported catalog metadata is invalid".into());
        }
        if self.imported.bounds_min[1].abs() > f32::EPSILON
            || (self.imported.bounds_max[1] - 1.0).abs() > f32::EPSILON
            || self
                .imported
                .bounds_min
                .into_iter()
                .chain(self.imported.bounds_max)
                .any(|value| !value.is_finite())
        {
            return Err("imported catalog bounds are invalid".into());
        }
        for lod in 0..LOD_COUNT {
            let descriptor = self.lod(IMPORTED_ARCHETYPE, lod);
            if descriptor.primitive_count * 3 != self.imported.lod_index_counts[lod as usize] {
                return Err(format!("imported catalog LOD {lod} metadata differs"));
            }
        }
        Ok(())
    }

    pub fn lod(&self, archetype: u32, lod: u32) -> &Lod {
        &self.lods[(archetype * LOD_COUNT + lod) as usize]
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"WLMSH002");
        for count in [
            self.vertices.len(),
            self.surface_vertices.len(),
            self.meshlets.len(),
            self.meshlet_vertices.len(),
            self.primitives.len(),
            self.lods.len(),
        ] {
            bytes.extend_from_slice(&(count as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&self.vertex_bytes());
        bytes.extend_from_slice(&self.surface_vertex_bytes());
        bytes.extend_from_slice(&self.meshlet_bytes());
        bytes.extend_from_slice(&u32_bytes(&self.meshlet_vertices));
        bytes.extend_from_slice(&u32_bytes(&self.primitives));
        bytes.extend_from_slice(&self.lod_bytes());
        for value in [
            &self.imported.source_json_sha256,
            &self.imported.source_bin_sha256,
            &self.imported.source_texture_sha256,
            &self.imported.cooked_sha256,
        ] {
            bytes.extend_from_slice(value.as_bytes());
        }
        for value in [self.imported.vertex_start, self.imported.vertex_count]
            .into_iter()
            .chain(self.imported.lod_index_counts)
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self
            .imported
            .lod_errors
            .into_iter()
            .chain(self.imported.bounds_min)
            .chain(self.imported.bounds_max)
        {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        bytes
    }

    pub fn sha256(&self) -> String {
        let digest = Sha256::digest(self.encoded_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn vertex_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.vertices.len() * 16);
        for vertex in &self.vertices {
            for value in vertex.position {
                bytes.extend_from_slice(&value.to_bits().to_le_bytes());
            }
        }
        bytes
    }

    pub fn meshlet_bytes(&self) -> Vec<u8> {
        encode_quads(self.meshlets.iter().map(|meshlet| {
            [
                meshlet.vertex_offset,
                meshlet.vertex_count,
                meshlet.primitive_offset,
                meshlet.primitive_count,
            ]
        }))
    }

    pub fn surface_vertex_bytes(&self) -> Vec<u8> {
        encode_f32(
            self.surface_vertices
                .iter()
                .flat_map(|vertex| vertex.normal_uv),
        )
    }

    pub fn meshlet_vertex_bytes(&self) -> Vec<u8> {
        u32_bytes(&self.meshlet_vertices)
    }

    pub fn primitive_bytes(&self) -> Vec<u8> {
        u32_bytes(&self.primitives)
    }

    pub fn lod_bytes(&self) -> Vec<u8> {
        encode_quads(self.lods.iter().map(|lod| {
            [
                lod.meshlet_offset,
                lod.meshlet_count,
                lod.vertex_count,
                lod.primitive_count,
            ]
        }))
    }
}

fn append_imported(catalog: &mut Catalog) -> Result<(), String> {
    let mut imported = imported::decode()?;
    let vertex_start = catalog.vertices.len() as u32;
    catalog.vertices.extend_from_slice(&imported.vertices);
    catalog
        .surface_vertices
        .extend_from_slice(&imported.surfaces);

    for indices in &imported.lod_indices {
        let meshlet_start = catalog.meshlets.len() as u32;
        let triangles = indices
            .chunks_exact(3)
            .map(|triangle| {
                [
                    vertex_start + triangle[0],
                    vertex_start + triangle[1],
                    vertex_start + triangle[2],
                ]
            })
            .collect::<Vec<_>>();
        append_partitioned_meshlets(catalog, &triangles);
        let emitted_vertex_count = catalog.meshlets[meshlet_start as usize..]
            .iter()
            .map(|meshlet| meshlet.vertex_count)
            .sum();
        catalog.lods.push(Lod {
            meshlet_offset: meshlet_start,
            meshlet_count: catalog.meshlets.len() as u32 - meshlet_start,
            vertex_count: emitted_vertex_count,
            primitive_count: indices.len() as u32 / 3,
        });
    }

    imported.metadata.vertex_start = vertex_start;
    catalog.imported = imported.metadata;
    Ok(())
}

pub(crate) fn append_partitioned_meshlets(catalog: &mut Catalog, triangles: &[[u32; 3]]) {
    let mut local_vertices = Vec::<u32>::new();
    let mut lookup = BTreeMap::<u32, u32>::new();
    let mut local_primitives = Vec::<u32>::new();
    for triangle in triangles {
        let additional = triangle
            .iter()
            .filter(|vertex| !lookup.contains_key(vertex))
            .count() as u32;
        if !local_primitives.is_empty()
            && (local_primitives.len() as u32 == MAX_MESHLET_PRIMITIVES
                || local_vertices.len() as u32 + additional > MAX_MESHLET_VERTICES)
        {
            flush_meshlet(
                catalog,
                &mut local_vertices,
                &mut lookup,
                &mut local_primitives,
            );
        }
        let mut local = [0; 3];
        for (destination, vertex) in local.iter_mut().zip(triangle) {
            *destination = *lookup.entry(*vertex).or_insert_with(|| {
                let index = local_vertices.len() as u32;
                local_vertices.push(*vertex);
                index
            });
        }
        local_primitives.push(local[0] | (local[1] << 8) | (local[2] << 16));
    }
    flush_meshlet(
        catalog,
        &mut local_vertices,
        &mut lookup,
        &mut local_primitives,
    );
}

fn flush_meshlet(
    catalog: &mut Catalog,
    vertices: &mut Vec<u32>,
    lookup: &mut BTreeMap<u32, u32>,
    primitives: &mut Vec<u32>,
) {
    if primitives.is_empty() {
        return;
    }
    catalog.meshlets.push(Meshlet {
        vertex_offset: catalog.meshlet_vertices.len() as u32,
        vertex_count: vertices.len() as u32,
        primitive_offset: catalog.primitives.len() as u32,
        primitive_count: primitives.len() as u32,
    });
    catalog.meshlet_vertices.append(vertices);
    catalog.primitives.append(primitives);
    lookup.clear();
}

fn u32_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn encode_f32(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    let iterator = values.into_iter();
    let mut bytes = Vec::with_capacity(iterator.size_hint().0 * size_of::<f32>());
    for value in iterator {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes
}

fn encode_quads(values: impl Iterator<Item = [u32; 4]>) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        for component in value {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    bytes
}
