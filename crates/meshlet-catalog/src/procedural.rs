use std::f32::consts::{PI, TAU};

use super::{Catalog, Lod, Vertex, VertexSurface, append_partitioned_meshlets};

pub(super) fn append_lod(catalog: &mut Catalog, archetype: u32, lod: u32) {
    let segments = [16, 12, 8][lod as usize];
    let bands = [8, 4, 2][lod as usize];
    let vertex_start = catalog.vertices.len() as u32;
    for ring in 0..=bands {
        let y = ring as f32 / bands as f32;
        let radius = profile_radius(archetype, y);
        for segment in 0..segments {
            let angle = segment as f32 * TAU / segments as f32;
            push_vertex(
                catalog,
                [radius * angle.cos(), y, radius * angle.sin(), 1.0],
            );
        }
    }
    let bottom_center = catalog.vertices.len() as u32;
    push_vertex(catalog, [0.0, 0.0, 0.0, 1.0]);
    let top_center = catalog.vertices.len() as u32;
    push_vertex(catalog, [0.0, 1.0, 0.0, 1.0]);

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

fn push_vertex(catalog: &mut Catalog, position: [f32; 4]) {
    let [x, y, z, _] = position;
    let normal = if x.abs() + z.abs() < f32::EPSILON {
        [0.0, if y < 0.5 { -1.0 } else { 1.0 }, 0.0]
    } else {
        normalize([x, 0.0, z])
    };
    let oct = encode_octahedral(normal);
    let u = (z.atan2(x) / TAU + 1.0).fract();
    catalog.vertices.push(Vertex { position });
    catalog.surface_vertices.push(VertexSurface {
        normal_uv: [oct[0], oct[1], u, y.clamp(0.0, 1.0)],
    });
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
        _ => unreachable!("procedural archetype is out of range"),
    };
    base * shape
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = value
        .into_iter()
        .map(|axis| axis * axis)
        .sum::<f32>()
        .sqrt();
    [value[0] / length, value[1] / length, value[2] / length]
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
