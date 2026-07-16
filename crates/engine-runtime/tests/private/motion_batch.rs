use anyhow::{Result, bail};

use super::*;
use crate::RegionCoord;
use crate::terrain_query::{TERRAIN_QUERY_HEIGHT_DENOMINATOR, TerrainBody, TerrainTriangle};

fn motion() -> TerrainBodyMotion {
    let position = TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap();
    let body = TerrainBody::new(position, 65_536, 65_536).unwrap();
    TerrainBodyMotion::new(body, 0)
}

fn flat(_: TerrainPosition) -> Result<TerrainHeight> {
    Ok(TerrainHeight {
        height_numerator: 0,
        height_denominator: TERRAIN_QUERY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::First,
    })
}

fn command(
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    initial_step_velocity_delta_q16: i32,
    step_acceleration_q16: i32,
) -> MotionBatchCommand {
    MotionBatchCommand {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16,
        initial_step_velocity_delta_q16,
        step_acceleration_q16,
    }
}

#[test]
fn zero_and_maximum_batches_are_exact() {
    let input = motion();
    let zero = advance_motion_batch(input, 0, command(1, -1, 0, 7_777, 0), flat).unwrap();
    assert_eq!(zero.output, input);
    assert_eq!(zero.terrain_query_count, 0);

    let maximum = advance_motion_batch(input, 8, command(1, -1, 0, 0, 0), flat).unwrap();
    assert_eq!(maximum.output.body().position().local_x_q9(), 8);
    assert_eq!(maximum.output.body().position().local_z_q9(), -8);
    assert_eq!(maximum.terrain_query_count, 8);
}

#[test]
fn one_batch_matches_partitioned_ticks() {
    let input = motion();
    let batch = advance_motion_batch(input, 8, command(3, 5, 0, 0, 0), flat).unwrap();
    let mut partitioned = input;
    let mut queries = 0;
    for _ in 0..8 {
        let step = advance_motion_batch(partitioned, 1, command(3, 5, 0, 0, 0), flat).unwrap();
        partitioned = step.output;
        queries += step.terrain_query_count;
    }
    assert_eq!(batch.output, partitioned);
    assert_eq!(batch.terrain_query_count, queries);
}

#[test]
fn initial_velocity_delta_precedes_acceleration_and_applies_once() {
    let input = motion();
    let one = advance_motion_batch(input, 1, command(0, 0, 0, 1_000, -100), flat).unwrap();
    assert_eq!(one.output.step_velocity_q16(), 900);
    assert_eq!(one.output.body().center_height_numerator(), 66_436);
    assert_eq!(one.terrain_query_count, 1);

    let batch = advance_motion_batch(input, 3, command(0, 0, 0, 1_000, -100), flat).unwrap();
    let first = advance_motion_batch(input, 1, command(0, 0, 0, 1_000, -100), flat).unwrap();
    let continuation =
        advance_motion_batch(first.output, 2, command(0, 0, 0, 0, -100), flat).unwrap();
    assert_eq!(batch.output, continuation.output);
    assert_eq!(batch.output.step_velocity_q16(), 700);
    assert_eq!(batch.output.body().center_height_numerator(), 67_936);
    assert_eq!(batch.terrain_query_count, 3);
}

#[test]
fn initial_velocity_delta_overflow_fails_before_query() {
    let input = TerrainBodyMotion::new(motion().body(), i32::MAX);
    let mut queries = 0;
    let result = advance_motion_batch(input, 1, command(0, 0, 0, 1, 0), |position| {
        queries += 1;
        flat(position)
    });
    let error = match result {
        Ok(_) => panic!("initial velocity delta overflow unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "terrain-body motion batch step 1 of 1 failed: initial step velocity delta is outside the signed 32-bit Q16 range"
    );
    assert_eq!(queries, 0);
}

#[test]
fn bounds_and_mid_batch_failure_return_no_output() {
    let input = motion();
    let mut bound_queries = 0;
    assert!(
        advance_motion_batch(input, 9, command(0, 0, 0, 0, 0), |_| {
            bound_queries += 1;
            flat(TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap())
        })
        .is_err()
    );
    assert_eq!(bound_queries, 0);

    let mut queries = 0;
    let failed = advance_motion_batch(input, 8, command(1, 0, 0, 0, 0), |position| {
        queries += 1;
        if queries == 3 {
            bail!("controlled third query failure");
        }
        flat(position)
    });
    let error = match failed {
        Ok(_) => panic!("controlled mid-batch failure unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "terrain-body motion batch step 3 of 8 failed: controlled third query failure"
    );
    assert_eq!(queries, 3);
}
