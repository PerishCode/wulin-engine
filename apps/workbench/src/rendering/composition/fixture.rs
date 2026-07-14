use std::time::Instant;

use anyhow::{Result, ensure};
use canonical_object_fixture::{Fixture as CanonicalFixture, generate_region_with_seed};
use serde::Serialize;

use crate::async_resident::{AsyncReservationReport, ObjectSourceNamespace, RegionAssignment};
use crate::load::{INSTANCES_PER_REGION, MAX_REGION_SIDE};
use crate::resident::{InstanceRecord, RegionUpload, generate_region};
use crate::world::RegionCoord;

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
    pub(super) fn object_source_namespace(self) -> ObjectSourceNamespace {
        ObjectSourceNamespace::from_bytes(self.canonical().stable_seed_namespace())
    }

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

    const fn canonical(self) -> CanonicalFixture {
        match self {
            Self::CellCenter => CanonicalFixture::CellCenter,
            Self::ArbitraryQ8 => CanonicalFixture::ArbitraryQ8,
        }
    }
}

pub(super) unsafe fn submit_generated_instances(
    renderer: &mut Renderer,
    reservation: AsyncReservationReport,
    fixture: CompositionFixture,
) -> Result<()> {
    let started = Instant::now();
    if let Some(source_namespace) = reservation.object_source_namespace {
        ensure!(
            source_namespace == fixture.object_source_namespace(),
            "canonical object source does not match the composition fixture"
        );
    }
    ensure!(
        reservation.object_stable_seed_namespace == reservation.object_source_namespace,
        "generated canonical object source requires identical source and stable-seed namespaces"
    );
    let uploads = reservation
        .assignments
        .iter()
        .map(|assignment| RegionUpload {
            slot: assignment.slot,
            records: generate_reserved_region(*assignment, fixture),
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

pub(super) fn generate_reserved_region(
    assignment: RegionAssignment,
    fixture: CompositionFixture,
) -> Vec<InstanceRecord> {
    match (assignment.global_region, assignment.stable_seed) {
        (Some(global_region), Some(stable_seed)) => {
            generate_canonical_fixture_region(global_region, stable_seed, fixture)
        }
        (None | Some(_), None) => generate_fixture_region(assignment.region_id, fixture),
        (None, Some(_)) => unreachable!("canonical object seed has no signed region"),
    }
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

pub(super) fn generate_canonical_fixture_region(
    global_region: RegionCoord,
    stable_seed: u32,
    fixture: CompositionFixture,
) -> Vec<InstanceRecord> {
    generate_region_with_seed(
        region_format::GlobalRegion::new(global_region.x, global_region.z),
        stable_seed,
        fixture.canonical(),
    )
}

pub(super) fn sample_ground(
    tile: &terrain_format::TerrainTile,
    local_index: usize,
    fixture: CompositionFixture,
    position: [f32; 3],
    semantic_region_id: u32,
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

fn arbitrary_fractions(region_id: u32, local_index: usize) -> (u32, u32) {
    debug_assert!(local_index < INSTANCES_PER_REGION as usize);
    let local_x = local_index as u32 % 32;
    let local_z = local_index as u32 / 32;
    let [region_x, region_z] = [
        i64::from(region_id % MAX_REGION_SIDE),
        i64::from(region_id / MAX_REGION_SIDE),
    ];
    let global_x_255 = cell_modulo(region_x, local_x, 255);
    let global_z_255 = cell_modulo(region_z, local_z, 255);
    if local_x == 0 || local_x == 31 || local_z == 0 || local_z == 31 {
        let u = match local_x {
            0 => 0,
            31 => 256,
            _ => interior_fraction(global_x_255, 73, 41),
        };
        let v = match local_z {
            0 => 0,
            31 => 256,
            _ => interior_fraction(global_z_255, 151, 17),
        };
        return (u, v);
    }
    let global_x_3 = cell_modulo(region_x, local_x, 3);
    let global_z_3 = cell_modulo(region_z, local_z, 3);
    match (global_x_3 + global_z_3 * 3) % 3 {
        0 => (64, 64),
        1 => (64, 192),
        _ => (192, 192),
    }
}

fn cell_modulo(region: i64, local: u32, modulus: u32) -> u32 {
    let region = region.rem_euclid(i64::from(modulus)) as u32;
    (region * 32 + local) % modulus
}

fn interior_fraction(coordinate: u32, multiplier: u32, offset: u32) -> u32 {
    (coordinate.wrapping_mul(multiplier).wrapping_add(offset) % 255) + 1
}
