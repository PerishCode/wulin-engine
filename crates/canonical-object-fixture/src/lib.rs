use region_format::{GlobalRegion, InstanceRecord, RECORDS_PER_REGION, canonical_stable_seed};
use sha2::{Digest, Sha256};

pub const CELL_CENTER_REVISION: &str = "canonical-generated-object-cell-center-v1";
pub const ARBITRARY_Q8_REVISION: &str = "canonical-generated-object-arbitrary-q8-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fixture {
    #[default]
    CellCenter,
    ArbitraryQ8,
}

impl Fixture {
    pub const fn revision(self) -> &'static str {
        match self {
            Self::CellCenter => CELL_CENTER_REVISION,
            Self::ArbitraryQ8 => ARBITRARY_Q8_REVISION,
        }
    }

    pub fn stable_seed_namespace(self) -> [u8; 32] {
        Sha256::digest(self.revision().as_bytes()).into()
    }
}

pub fn generate_region(region: GlobalRegion, fixture: Fixture) -> Vec<InstanceRecord> {
    let stable_seed = canonical_stable_seed(fixture.stable_seed_namespace(), region);
    generate_region_with_seed(region, stable_seed, fixture)
}

pub fn generate_region_with_seed(
    region: GlobalRegion,
    stable_seed: u32,
    fixture: Fixture,
) -> Vec<InstanceRecord> {
    (0..RECORDS_PER_REGION as usize)
        .map(|local_index| {
            let local_x = local_index as u32 % 32;
            let local_z = local_index as u32 / 32;
            let (position_x, position_z) = if fixture == Fixture::CellCenter {
                (
                    ((local_x as f32 + 0.5) / 32.0 - 0.5) * 16.0,
                    ((local_z as f32 + 0.5) / 32.0 - 0.5) * 16.0,
                )
            } else {
                let (u, v) = arbitrary_fractions(region, local_index);
                (
                    -8.0 + (local_x * 256 + u) as f32 / 512.0,
                    -8.0 + (local_z * 256 + v) as f32 / 512.0,
                )
            };
            InstanceRecord {
                position: [position_x, 0.0, position_z],
                height: instance_height(canonical_stable_key(stable_seed, local_index as u32)),
                region_id: stable_seed,
            }
        })
        .collect()
}

pub fn arbitrary_fractions(region: GlobalRegion, local_index: usize) -> (u32, u32) {
    debug_assert!(local_index < RECORDS_PER_REGION as usize);
    let local_x = local_index as u32 % 32;
    let local_z = local_index as u32 / 32;
    let global_x_255 = cell_modulo(region.x, local_x, 255);
    let global_z_255 = cell_modulo(region.z, local_z, 255);
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
    let global_x_3 = cell_modulo(region.x, local_x, 3);
    let global_z_3 = cell_modulo(region.z, local_z, 3);
    match (global_x_3 + global_z_3 * 3) % 3 {
        0 => (64, 64),
        1 => (64, 192),
        _ => (192, 192),
    }
}

pub fn canonical_stable_key(region_seed: u32, local_index: u32) -> u32 {
    region_seed ^ local_index.wrapping_mul(747_796_405)
}

pub fn instance_height(reference: u32) -> f32 {
    let mut value = reference
        .wrapping_mul(747_796_405)
        .wrapping_add(2_891_336_453);
    value = ((value >> ((value >> 28) + 4)) ^ value).wrapping_mul(277_803_737);
    value = (value >> 22) ^ value;
    0.7 + (value & 1023) as f32 / 1023.0 * 2.3
}

fn cell_modulo(region: i64, local: u32, modulus: u32) -> u32 {
    let region = region.rem_euclid(i64::from(modulus)) as u32;
    (region * 32 + local) % modulus
}

fn interior_fraction(coordinate: u32, multiplier: u32, offset: u32) -> u32 {
    (coordinate.wrapping_mul(multiplier).wrapping_add(offset) % 255) + 1
}
