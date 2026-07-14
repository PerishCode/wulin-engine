use crate::async_resident::ObjectSourceNamespace;
use crate::load::MAX_REGION_SIDE;

pub(super) const NAME: &str = "arbitrary-q8";
pub(super) const GROUNDING_MODE: u32 = 2;
pub(super) const GROUND_DENOMINATOR: u32 = 65_536;
pub(super) const POSITION_DENOMINATOR: u32 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TriangleClass {
    First,
    Diagonal,
    Second,
}

pub(super) fn object_source_namespace() -> ObjectSourceNamespace {
    ObjectSourceNamespace::from_bytes(canonical_object_fixture::stable_seed_namespace())
}

pub(super) fn sample_ground(
    tile: &terrain_format::TerrainTile,
    position: [f32; 3],
    semantic_region_id: u32,
) -> (i32, TriangleClass) {
    let at = |x: usize, z: usize| i32::from(tile.heights[z * terrain_format::SAMPLE_SIDE + x]);
    let region_x = semantic_region_id % MAX_REGION_SIDE;
    let region_z = semantic_region_id / MAX_REGION_SIDE;
    let minimum_x = (region_x as i32 - 64) * 16 - 8;
    let minimum_z = (region_z as i32 - 64) * 16 - 8;
    let x_q9 = ((position[0] - minimum_x as f32) * 512.0)
        .round()
        .clamp(0.0, 8192.0) as u32;
    let z_q9 = ((position[2] - minimum_z as f32) * 512.0)
        .round()
        .clamp(0.0, 8192.0) as u32;
    let cell_x = (x_q9 >> 8).min(31) as usize;
    let cell_z = (z_q9 >> 8).min(31) as usize;
    let u = x_q9 - cell_x as u32 * 256;
    let v = z_q9 - cell_z as u32 * 256;
    let sum = u + v;
    let value = if sum <= 256 {
        at(cell_x, cell_z) * (256 - sum) as i32
            + at(cell_x + 1, cell_z) * u as i32
            + at(cell_x, cell_z + 1) * v as i32
    } else {
        at(cell_x + 1, cell_z) * (256 - v) as i32
            + at(cell_x, cell_z + 1) * (256 - u) as i32
            + at(cell_x + 1, cell_z + 1) * (sum - 256) as i32
    };
    let triangle = match sum.cmp(&256) {
        std::cmp::Ordering::Less => TriangleClass::First,
        std::cmp::Ordering::Equal => TriangleClass::Diagonal,
        std::cmp::Ordering::Greater => TriangleClass::Second,
    };
    (value, triangle)
}
