use std::time::Instant;

use anyhow::Result;
use serde::Serialize;

use crate::async_resident::AsyncReservationReport;
use crate::load::{INSTANCES_PER_REGION, MAX_REGION_SIDE};
use crate::resident::{InstanceRecord, RegionUpload, generate_region};

use super::super::renderer::Renderer;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompositionFixture {
    #[default]
    CellCenter,
    ArbitraryQ8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TriangleClass {
    First,
    Diagonal,
    Second,
}

impl CompositionFixture {
    pub(super) const fn grounding_mode(self) -> u32 {
        match self {
            Self::CellCenter => 1,
            Self::ArbitraryQ8 => 2,
        }
    }

    pub(super) const fn ground_denominator(self) -> u32 {
        match self {
            Self::CellCenter => 512,
            Self::ArbitraryQ8 => 65_536,
        }
    }

    pub(super) const fn position_denominator(self) -> u32 {
        match self {
            Self::CellCenter => 64,
            Self::ArbitraryQ8 => 512,
        }
    }
}

pub(super) unsafe fn submit_generated_instances(
    renderer: &mut Renderer,
    reservation: AsyncReservationReport,
    fixture: CompositionFixture,
) -> Result<()> {
    let started = Instant::now();
    let uploads = reservation
        .assignments
        .iter()
        .map(|assignment| RegionUpload {
            slot: assignment.slot,
            records: generate_fixture_region(assignment.region_id, fixture),
        })
        .collect();
    let generation_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let release_fence = renderer.next_fence_value;
    renderer.next_fence_value += 1;
    unsafe {
        renderer.async_resident_renderer.submit_generated(
            reservation.transaction_id,
            uploads,
            generation_ms,
            &renderer.queue,
            &renderer.fence,
            release_fence,
        )
    }?;
    Ok(())
}

pub(super) fn generate_fixture_region(
    region_id: u32,
    fixture: CompositionFixture,
) -> Vec<InstanceRecord> {
    let mut records = generate_region(region_id);
    if fixture == CompositionFixture::CellCenter {
        return records;
    }
    let region_x = region_id % MAX_REGION_SIDE;
    let region_z = region_id / MAX_REGION_SIDE;
    let minimum_x = (region_x as i32 - 64) * 16 - 8;
    let minimum_z = (region_z as i32 - 64) * 16 - 8;
    for (local_index, record) in records.iter_mut().enumerate() {
        let (u, v) = arbitrary_fractions(region_id, local_index);
        let cell_x = local_index as u32 % 32;
        let cell_z = local_index as u32 / 32;
        let x_q9 = cell_x * 256 + u;
        let z_q9 = cell_z * 256 + v;
        record.position[0] = minimum_x as f32 + x_q9 as f32 / 512.0;
        record.position[2] = minimum_z as f32 + z_q9 as f32 / 512.0;
    }
    records
}

pub(super) fn sample_ground(
    tile: &terrain_format::TerrainTile,
    local_index: usize,
    fixture: CompositionFixture,
) -> (i32, TriangleClass) {
    let cell_x = local_index % terrain_format::CELL_SIDE;
    let cell_z = local_index / terrain_format::CELL_SIDE;
    let at = |x: usize, z: usize| i32::from(tile.heights[z * terrain_format::SAMPLE_SIDE + x]);
    if fixture == CompositionFixture::CellCenter {
        return (
            at(cell_x + 1, cell_z) + at(cell_x, cell_z + 1),
            TriangleClass::Diagonal,
        );
    }

    let (u, v) = arbitrary_fractions(tile.region_id, local_index);
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

fn arbitrary_fractions(region_id: u32, local_index: usize) -> (u32, u32) {
    debug_assert!(local_index < INSTANCES_PER_REGION as usize);
    let local_x = local_index as u32 % 32;
    let local_z = local_index as u32 / 32;
    let region_x = region_id % MAX_REGION_SIDE;
    let region_z = region_id / MAX_REGION_SIDE;
    let global_x = region_x * 32 + local_x;
    let global_z = region_z * 32 + local_z;
    if local_x == 0 || local_x == 31 || local_z == 0 || local_z == 31 {
        let u = match local_x {
            0 => 0,
            31 => 256,
            _ => interior_fraction(global_x, 73, 41),
        };
        let v = match local_z {
            0 => 0,
            31 => 256,
            _ => interior_fraction(global_z, 151, 17),
        };
        return (u, v);
    }
    match (global_x + global_z * 3) % 3 {
        0 => (64, 64),
        1 => (64, 192),
        _ => (192, 192),
    }
}

fn interior_fraction(coordinate: u32, multiplier: u32, offset: u32) -> u32 {
    (coordinate.wrapping_mul(multiplier).wrapping_add(offset) % 255) + 1
}
