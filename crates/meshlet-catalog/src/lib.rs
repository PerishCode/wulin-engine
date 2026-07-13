use std::collections::BTreeMap;
use std::f32::consts::{PI, TAU};

use sha2::{Digest, Sha256};

pub const ARCHETYPE_COUNT: u32 = 8;
pub const LOD_COUNT: u32 = 3;
pub const MAX_MESHLET_VERTICES: u32 = 64;
pub const MAX_MESHLET_PRIMITIVES: u32 = 126;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 4],
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
pub struct Catalog {
    pub vertices: Vec<Vertex>,
    pub meshlets: Vec<Meshlet>,
    pub meshlet_vertices: Vec<u32>,
    pub primitives: Vec<u32>,
    pub lods: Vec<Lod>,
}

impl Catalog {
    pub fn build() -> Self {
        let mut catalog = Self {
            vertices: Vec::new(),
            meshlets: Vec::new(),
            meshlet_vertices: Vec::new(),
            primitives: Vec::new(),
            lods: Vec::with_capacity((ARCHETYPE_COUNT * LOD_COUNT) as usize),
        };
        for archetype in 0..ARCHETYPE_COUNT {
            for lod in 0..LOD_COUNT {
                append_lod(&mut catalog, archetype, lod);
            }
        }
        catalog.validate().expect("generated catalog is invalid");
        catalog
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lods.len() != (ARCHETYPE_COUNT * LOD_COUNT) as usize {
            return Err("catalog LOD count is not canonical".into());
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
        Ok(())
    }

    pub fn lod(&self, archetype: u32, lod: u32) -> &Lod {
        &self.lods[(archetype * LOD_COUNT + lod) as usize]
    }

    pub fn encoded_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"WLMSH001");
        for count in [
            self.vertices.len(),
            self.meshlets.len(),
            self.meshlet_vertices.len(),
            self.primitives.len(),
            self.lods.len(),
        ] {
            bytes.extend_from_slice(&(count as u32).to_le_bytes());
        }
        bytes.extend_from_slice(&self.vertex_bytes());
        bytes.extend_from_slice(&self.meshlet_bytes());
        bytes.extend_from_slice(&u32_bytes(&self.meshlet_vertices));
        bytes.extend_from_slice(&u32_bytes(&self.primitives));
        bytes.extend_from_slice(&self.lod_bytes());
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

fn append_lod(catalog: &mut Catalog, archetype: u32, lod: u32) {
    let segments = [16, 12, 8][lod as usize];
    let bands = [8, 4, 2][lod as usize];
    let vertex_start = catalog.vertices.len() as u32;
    for ring in 0..=bands {
        let y = ring as f32 / bands as f32;
        let radius = profile_radius(archetype, y);
        let phase = profile_phase(archetype, y);
        for segment in 0..segments {
            let angle = segment as f32 * TAU / segments as f32 + phase;
            catalog.vertices.push(Vertex {
                position: [radius * angle.cos(), y, radius * angle.sin(), 1.0],
            });
        }
    }
    let bottom_center = catalog.vertices.len() as u32;
    catalog.vertices.push(Vertex {
        position: [0.0, 0.0, 0.0, 1.0],
    });
    let top_center = catalog.vertices.len() as u32;
    catalog.vertices.push(Vertex {
        position: [0.0, 1.0, 0.0, 1.0],
    });

    let meshlet_start = catalog.meshlets.len() as u32;
    let mut side = Vec::with_capacity((bands * segments * 2) as usize);
    for band in 0..bands {
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            let lower = vertex_start + band * segments + segment;
            let lower_next = vertex_start + band * segments + next;
            let upper = vertex_start + (band + 1) * segments + segment;
            let upper_next = vertex_start + (band + 1) * segments + next;
            side.push([lower, upper, upper_next]);
            side.push([lower, upper_next, lower_next]);
        }
    }
    append_partitioned_meshlets(catalog, &side);

    let mut bottom = Vec::with_capacity(segments as usize);
    let mut top = Vec::with_capacity(segments as usize);
    let top_ring = vertex_start + bands * segments;
    for segment in 0..segments {
        let next = (segment + 1) % segments;
        bottom.push([bottom_center, vertex_start + next, vertex_start + segment]);
        top.push([top_center, top_ring + segment, top_ring + next]);
    }
    append_partitioned_meshlets(catalog, &bottom);
    append_partitioned_meshlets(catalog, &top);

    let emitted_vertex_count = catalog.meshlets[meshlet_start as usize..]
        .iter()
        .map(|meshlet| meshlet.vertex_count)
        .sum();
    catalog.lods.push(Lod {
        meshlet_offset: meshlet_start,
        meshlet_count: catalog.meshlets.len() as u32 - meshlet_start,
        vertex_count: emitted_vertex_count,
        primitive_count: bands * segments * 2 + segments * 2,
    });
}

fn append_partitioned_meshlets(catalog: &mut Catalog, triangles: &[[u32; 3]]) {
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

fn profile_radius(archetype: u32, y: f32) -> f32 {
    let base = 0.17 + archetype as f32 * 0.012;
    let shape = match archetype {
        0 => 1.0 - 0.18 * y,
        1 => 0.82 + 0.28 * (PI * y).sin(),
        2 => 1.08 - 0.38 * (PI * y).sin(),
        3 => 1.18 - 0.68 * y,
        4 => 0.84 + 0.18 * (4.0 * PI * y).cos(),
        5 => 0.58 + 0.62 * (PI * y).sin(),
        6 => 0.72 + 0.52 * y * y,
        _ => 1.03 - 0.26 * (2.0 * PI * y).sin(),
    };
    base * shape
}

fn profile_phase(archetype: u32, y: f32) -> f32 {
    if archetype == 7 { y * PI * 0.75 } else { 0.0 }
}

fn u32_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
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
