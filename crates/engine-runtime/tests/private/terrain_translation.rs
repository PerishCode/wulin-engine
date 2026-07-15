use std::cell::Cell;

use anyhow::{Result, anyhow};

use super::*;
use crate::RegionCoord;
use crate::terrain_query::{TerrainContactClassification, TerrainTriangle};

const HALF_HEIGHT: i32 = 65_536;

fn position(region: RegionCoord, local_x_q9: i32, local_z_q9: i32) -> TerrainPosition {
    TerrainPosition::new(region, local_x_q9, local_z_q9).unwrap()
}

fn motion(position: TerrainPosition, center: i32, velocity: i32) -> TerrainBodyMotion {
    TerrainBodyMotion::new(
        TerrainBody::new(position, center, HALF_HEIGHT).unwrap(),
        velocity,
    )
}

fn terrain(height_numerator: i32) -> TerrainHeight {
    TerrainHeight {
        height_numerator,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::Diagonal,
    }
}

#[test]
fn accepts_bounded_correction_and_preserves_velocity() {
    let input = motion(position(RegionCoord::ZERO, 0, 0), HALF_HEIGHT, -321);
    for (limit, expected_correction) in [(101, 100), (100, 100)] {
        let result = translate_terrain_body(input, 17, -29, limit, |_| Ok(terrain(100))).unwrap();
        assert!(!result.blocked);
        assert_eq!(
            result.contact.classification,
            TerrainContactClassification::Penetrating
        );
        assert_eq!(result.contact.correction_numerator, expected_correction);
        assert_eq!(
            result.output.body().center_height_numerator(),
            HALF_HEIGHT + expected_correction as i32
        );
        assert_eq!(result.output.step_velocity_q16(), -321);
        assert_eq!(result.position_denominator, 512);
        assert_eq!(result.height_denominator, 65_536);
    }
}

#[test]
fn blocks_excess_correction_with_exact_input_output() {
    let input = motion(position(RegionCoord::ZERO, 0, 0), HALF_HEIGHT, 777);
    let blocked = translate_terrain_body(input, 64, 32, 99, |_| Ok(terrain(100))).unwrap();
    assert!(blocked.blocked);
    assert_eq!(blocked.contact.correction_numerator, 100);
    assert_eq!(blocked.output, blocked.input);
    assert_eq!(
        serde_json::to_value(blocked.output).unwrap(),
        serde_json::to_value(blocked.input).unwrap()
    );

    let zero_blocked = translate_terrain_body(input, 1, 0, 0, |_| Ok(terrain(1))).unwrap();
    assert!(zero_blocked.blocked);
    let zero_accepted = translate_terrain_body(input, 1, 0, 0, |_| Ok(terrain(0))).unwrap();
    assert!(!zero_accepted.blocked);
    assert_eq!(
        zero_accepted.contact.classification,
        TerrainContactClassification::Touching
    );
}

#[test]
fn crosses_signed_seams_and_does_not_snap_downhill() {
    let initial_position = position(RegionCoord::ZERO, 4095, -4096);
    let input = motion(initial_position, HALF_HEIGHT + 500, -123);
    let expected_position = position(RegionCoord::new(1, -1), -4096, 4095);
    let result = translate_terrain_body(input, 1, -1, 0, |queried| {
        assert_eq!(queried, expected_position);
        Ok(terrain(0))
    })
    .unwrap();
    assert!(!result.blocked);
    assert_eq!(result.candidate_body.position(), expected_position);
    assert_eq!(result.output.body().position(), expected_position);
    assert_eq!(
        result.output.body().center_height_numerator(),
        HALF_HEIGHT + 500
    );
    assert_eq!(result.output.step_velocity_q16(), -123);
    assert_eq!(
        result.contact.classification,
        TerrainContactClassification::Separated
    );
    assert_eq!(result.contact.correction_numerator, 0);
}

#[test]
fn accepted_partitions_converge_exactly() {
    let input = motion(
        position(RegionCoord::new(17, -23), 4070, -4080),
        HALF_HEIGHT,
        88,
    );
    let first = translate_terrain_body(input, 15_000, -12_000, 0, |_| Ok(terrain(0)))
        .unwrap()
        .output;
    let partitioned = translate_terrain_body(first, -9_000, 15_500, 0, |_| Ok(terrain(0)))
        .unwrap()
        .output;
    let combined = translate_terrain_body(input, 6_000, 3_500, 0, |_| Ok(terrain(0)))
        .unwrap()
        .output;
    assert_eq!(partitioned, combined);
    assert_eq!(partitioned.step_velocity_q16(), 88);
}

#[test]
fn validation_precedes_query_and_query_errors_are_transactional() {
    let queries = Cell::new(0_u32);
    let input = motion(position(RegionCoord::ZERO, 0, 0), HALF_HEIGHT, 7);
    assert!(
        translate_terrain_body(input, 0, 0, -1, |_| {
            queries.set(queries.get() + 1);
            Ok(terrain(0))
        })
        .is_err()
    );
    assert_eq!(queries.get(), 0);

    let overflow = motion(
        position(RegionCoord::new(i64::MAX, 0), 4095, 0),
        HALF_HEIGHT,
        7,
    );
    assert!(
        translate_terrain_body(overflow, 1, 0, 0, |_| {
            queries.set(queries.get() + 1);
            Ok(terrain(0))
        })
        .is_err()
    );
    assert_eq!(queries.get(), 0);

    let outside: Result<TerrainHeight> = Err(anyhow!("outside committed snapshot"));
    assert!(
        translate_terrain_body(input, 10, 20, 0, |queried| {
            queries.set(queries.get() + 1);
            assert_eq!(queried, position(RegionCoord::ZERO, 10, 20));
            outside
        })
        .is_err()
    );
    assert_eq!(queries.get(), 1);
}

#[test]
fn unrepresentable_contact_returns_no_translation() {
    let input = motion(position(RegionCoord::ZERO, 0, 0), i32::MAX, 11);
    let queries = Cell::new(0_u32);
    assert!(
        translate_terrain_body(input, 0, 0, i32::MAX, |_| {
            queries.set(queries.get() + 1);
            Ok(terrain(i32::MAX))
        })
        .is_err()
    );
    assert_eq!(queries.get(), 1);
    assert_eq!(input.body().center_height_numerator(), i32::MAX);
    assert_eq!(input.step_velocity_q16(), 11);
}
