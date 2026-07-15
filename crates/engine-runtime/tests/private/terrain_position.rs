use std::time::Instant;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::*;

fn oracle(position: TerrainPosition, delta_x_q9: i32, delta_z_q9: i32) -> Result<TerrainPosition> {
    let translate_axis = |region: i64, local: i32, delta: i32| -> Result<(i64, i32)> {
        let absolute = i128::from(region) * i128::from(TERRAIN_POSITION_REGION_SIDE_Q9)
            + i128::from(local)
            + i128::from(delta);
        let biased = absolute - i128::from(TERRAIN_POSITION_LOCAL_MIN_Q9);
        let side = i128::from(TERRAIN_POSITION_REGION_SIDE_Q9);
        let next_region = biased.div_euclid(side);
        let next_local = biased.rem_euclid(side) + i128::from(TERRAIN_POSITION_LOCAL_MIN_Q9);
        Ok((
            i64::try_from(next_region).context("oracle region is outside signed 64-bit range")?,
            i32::try_from(next_local)
                .context("oracle local coordinate is outside signed 32-bit")?,
        ))
    };
    let (region_x, local_x) =
        translate_axis(position.region().x, position.local_x_q9(), delta_x_q9)?;
    let (region_z, local_z) =
        translate_axis(position.region().z, position.local_z_q9(), delta_z_q9)?;
    TerrainPosition::new(RegionCoord::new(region_x, region_z), local_x, local_z)
}

#[test]
fn translates_positive_negative_and_multiple_region_seams() {
    let origin = TerrainPosition::new(RegionCoord::ZERO, 4095, -4096).unwrap();
    assert_eq!(
        origin.translated_q9(1, -1).unwrap(),
        TerrainPosition::new(RegionCoord::new(1, -1), -4096, 4095).unwrap()
    );

    let multi = TerrainPosition::new(RegionCoord::new(1_i64 << 40, -(1_i64 << 40)), 0, 0).unwrap();
    assert_eq!(
        multi
            .translated_q9(3 * TERRAIN_POSITION_REGION_SIDE_Q9 + 10, -16391)
            .unwrap(),
        oracle(multi, 3 * TERRAIN_POSITION_REGION_SIDE_Q9 + 10, -16391).unwrap()
    );
}

#[test]
fn translation_is_partition_and_round_trip_invariant() {
    let initial = TerrainPosition::new(RegionCoord::new(17, -23), 4070, -4080).unwrap();
    let partitioned = initial
        .translated_q9(15_000, -12_000)
        .unwrap()
        .translated_q9(-9_000, 15_500)
        .unwrap();
    let combined = initial.translated_q9(6_000, 3_500).unwrap();
    assert_eq!(partitioned, combined);
    assert_eq!(combined.translated_q9(-6_000, -3_500).unwrap(), initial);
    assert_eq!(initial.translated_q9(0, 0).unwrap(), initial);
}

#[test]
fn signed_region_overflow_returns_no_position() {
    let maximum = TerrainPosition::new(RegionCoord::new(i64::MAX, 0), 4095, 0).unwrap();
    let minimum = TerrainPosition::new(RegionCoord::new(0, i64::MIN), 0, -4096).unwrap();
    assert!(maximum.translated_q9(1, 0).is_err());
    assert!(minimum.translated_q9(0, -1).is_err());
    assert_eq!(maximum.region(), RegionCoord::new(i64::MAX, 0));
    assert_eq!(minimum.region(), RegionCoord::new(0, i64::MIN));
}

fn sweep() -> (String, u64) {
    let started = Instant::now();
    let regions = [
        RegionCoord::ZERO,
        RegionCoord::new(1_i64 << 40, -(1_i64 << 40)),
        RegionCoord::new(-(1_i64 << 40), 1_i64 << 40),
        RegionCoord::new(i64::MAX - 300_000, i64::MIN + 300_000),
        RegionCoord::new(i64::MIN + 300_000, i64::MAX - 300_000),
    ];
    let mut digest = Sha256::new();
    for index in 0_u32..65_536 {
        let mixed = u64::from(index)
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let local_x = TERRAIN_POSITION_LOCAL_MIN_Q9 + (index as i32 & 8191);
        let local_z = TERRAIN_POSITION_LOCAL_MIN_Q9 + ((index.wrapping_mul(4051) as i32) & 8191);
        let input = TerrainPosition::new(regions[index as usize % regions.len()], local_x, local_z)
            .unwrap();
        let delta_x = mixed as u32 as i32;
        let delta_z = mixed.rotate_left(29) as u32 as i32;
        let expected = oracle(input, delta_x, delta_z).unwrap();
        let actual = input.translated_q9(delta_x, delta_z).unwrap();
        assert_eq!(actual, expected, "translation sweep case {index} diverged");

        digest.update(index.to_le_bytes());
        digest.update(input.region().x.to_le_bytes());
        digest.update(input.region().z.to_le_bytes());
        digest.update(input.local_x_q9().to_le_bytes());
        digest.update(input.local_z_q9().to_le_bytes());
        digest.update(delta_x.to_le_bytes());
        digest.update(delta_z.to_le_bytes());
        digest.update(actual.region().x.to_le_bytes());
        digest.update(actual.region().z.to_le_bytes());
        digest.update(actual.local_x_q9().to_le_bytes());
        digest.update(actual.local_z_q9().to_le_bytes());
    }
    (
        format!("{:x}", digest.finalize()),
        u64::try_from(started.elapsed().as_nanos()).unwrap(),
    )
}

#[test]
fn exhaustive_translation_sweep_is_exact_and_replayable() {
    let (result_hash, elapsed_nanoseconds) = sweep();
    let (replay_hash, _) = sweep();
    eprintln!(
        "terrain position sweep: cases=65536 hash={result_hash} elapsed_ns={elapsed_nanoseconds}"
    );
    assert_eq!(result_hash, replay_hash);
    assert_eq!(
        result_hash,
        "8bf1a9181426aadf6970009165f1269e9358463c58e2cca734435a5bc02ff683"
    );
}
