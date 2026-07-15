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

#[test]
fn zero_and_maximum_batches_are_exact() {
    let input = motion();
    let zero = advance_motion_batch(input, 0, 1, -1, 0, 0, flat).unwrap();
    assert_eq!(zero.output, input);
    assert_eq!(zero.terrain_query_count, 0);

    let maximum = advance_motion_batch(input, 8, 1, -1, 0, 0, flat).unwrap();
    assert_eq!(maximum.output.body().position().local_x_q9(), 8);
    assert_eq!(maximum.output.body().position().local_z_q9(), -8);
    assert_eq!(maximum.terrain_query_count, 8);
}

#[test]
fn one_batch_matches_partitioned_ticks() {
    let input = motion();
    let batch = advance_motion_batch(input, 8, 3, 5, 0, 0, flat).unwrap();
    let mut partitioned = input;
    let mut queries = 0;
    for _ in 0..8 {
        let step = advance_motion_batch(partitioned, 1, 3, 5, 0, 0, flat).unwrap();
        partitioned = step.output;
        queries += step.terrain_query_count;
    }
    assert_eq!(batch.output, partitioned);
    assert_eq!(batch.terrain_query_count, queries);
}

#[test]
fn bounds_and_mid_batch_failure_return_no_output() {
    let input = motion();
    let mut bound_queries = 0;
    assert!(
        advance_motion_batch(input, 9, 0, 0, 0, 0, |_| {
            bound_queries += 1;
            flat(TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap())
        })
        .is_err()
    );
    assert_eq!(bound_queries, 0);

    let mut queries = 0;
    let failed = advance_motion_batch(input, 8, 1, 0, 0, 0, |position| {
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
        "retained terrain body batch step 3 of 8 failed: controlled third query failure"
    );
    assert_eq!(queries, 3);
}
