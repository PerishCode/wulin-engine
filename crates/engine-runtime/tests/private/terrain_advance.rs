use std::cell::RefCell;

use anyhow::anyhow;

use super::*;
use crate::RegionCoord;
use crate::terrain_query::{TerrainBody, TerrainContactClassification, TerrainTriangle};

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
fn accepted_uphill_reuses_destination_and_grounds() {
    let origin = position(RegionCoord::ZERO, 0, 0);
    let destination = position(RegionCoord::ZERO, 128, -64);
    let queries = RefCell::new(Vec::new());
    let advance = advance_terrain_body(
        motion(origin, HALF_HEIGHT, 0),
        128,
        -64,
        100,
        -10,
        |queried| {
            queries.borrow_mut().push(queried);
            assert_eq!(queried, destination);
            Ok(terrain(100))
        },
    )
    .unwrap();

    assert_eq!(&*queries.borrow(), &[destination]);
    assert_eq!(advance.terrain_query_count, 1);
    assert!(!advance.translation.blocked);
    assert_eq!(advance.translation.contact.correction_numerator, 100);
    assert!(advance.grounded);
    assert_eq!(advance.output.body().position(), destination);
    assert_eq!(
        advance.output.body().center_height_numerator(),
        HALF_HEIGHT + 100
    );
    assert_eq!(advance.output.step_velocity_q16(), 0);
    assert_eq!(advance.output, advance.vertical_step.output);
    assert_eq!(advance.position_denominator, 512);
    assert_eq!(advance.height_denominator, 65_536);
    assert_eq!(advance.steps_per_second, 60);
}

#[test]
fn downhill_begins_falling_in_the_same_tick() {
    let origin = position(RegionCoord::ZERO, 0, 0);
    let destination = position(RegionCoord::ZERO, 64, 0);
    let advance = advance_terrain_body(
        motion(origin, HALF_HEIGHT + 100, 0),
        64,
        0,
        0,
        -10,
        |queried| {
            assert_eq!(queried, destination);
            Ok(terrain(0))
        },
    )
    .unwrap();

    assert!(!advance.translation.blocked);
    assert_eq!(
        advance.translation.contact.classification,
        TerrainContactClassification::Separated
    );
    assert!(!advance.grounded);
    assert_eq!(
        advance.output.body().center_height_numerator(),
        HALF_HEIGHT + 90
    );
    assert_eq!(advance.output.step_velocity_q16(), -10);
    assert_eq!(advance.terrain_query_count, 1);
}

#[test]
fn blocked_planar_intent_still_advances_vertically_at_origin() {
    let origin = position(RegionCoord::ZERO, 0, 0);
    let destination = position(RegionCoord::ZERO, 128, 0);
    let queries = RefCell::new(Vec::new());
    let advance = advance_terrain_body(
        motion(origin, HALF_HEIGHT, 0),
        128,
        0,
        100,
        -10,
        |queried| {
            queries.borrow_mut().push(queried);
            if queried == destination {
                Ok(terrain(200))
            } else if queried == origin {
                Ok(terrain(0))
            } else {
                Err(anyhow!("unexpected query"))
            }
        },
    )
    .unwrap();

    assert_eq!(&*queries.borrow(), &[destination, origin]);
    assert_eq!(advance.terrain_query_count, 2);
    assert!(advance.translation.blocked);
    assert_eq!(advance.translation.output, advance.input);
    assert!(advance.grounded);
    assert_eq!(advance.output.body().position(), origin);
    assert_eq!(advance.output.body().center_height_numerator(), HALF_HEIGHT);
    assert_eq!(advance.output.step_velocity_q16(), 0);
}

#[test]
fn zero_move_and_signed_seam_preserve_planar_first_order() {
    let seam_origin = position(RegionCoord::ZERO, 4095, -4096);
    let seam_destination = position(RegionCoord::new(1, -1), -4096, 4095);
    let crossed = advance_terrain_body(
        motion(seam_origin, HALF_HEIGHT + 20, 10),
        1,
        -1,
        0,
        0,
        |queried| {
            assert_eq!(queried, seam_destination);
            Ok(terrain(0))
        },
    )
    .unwrap();
    assert_eq!(crossed.output.body().position(), seam_destination);
    assert_eq!(
        crossed.output.body().center_height_numerator(),
        HALF_HEIGHT + 30
    );
    assert_eq!(crossed.output.step_velocity_q16(), 10);
    assert_eq!(crossed.terrain_query_count, 1);

    let zero = advance_terrain_body(crossed.output, 0, 0, 0, -10, |queried| {
        assert_eq!(queried, seam_destination);
        Ok(terrain(0))
    })
    .unwrap();
    assert_eq!(zero.terrain_query_count, 1);
    assert_eq!(zero.output.body().position(), seam_destination);
    assert_eq!(zero.output.step_velocity_q16(), 0);
}

#[test]
fn validation_and_query_failures_return_no_advance() {
    let origin = position(RegionCoord::ZERO, 0, 0);
    let input = motion(origin, HALF_HEIGHT, 0);
    let queries = RefCell::new(Vec::new());
    assert!(
        advance_terrain_body(input, 0, 0, -1, 0, |queried| {
            queries.borrow_mut().push(queried);
            Ok(terrain(0))
        })
        .is_err()
    );
    assert!(queries.borrow().is_empty());

    let overflow = motion(
        position(RegionCoord::new(i64::MAX, 0), 4095, 0),
        HALF_HEIGHT,
        0,
    );
    assert!(
        advance_terrain_body(overflow, 1, 0, 0, 0, |queried| {
            queries.borrow_mut().push(queried);
            Ok(terrain(0))
        })
        .is_err()
    );
    assert!(queries.borrow().is_empty());

    assert!(
        advance_terrain_body(input, 1, 0, 0, 0, |queried| {
            queries.borrow_mut().push(queried);
            Err(anyhow!("destination unavailable"))
        })
        .is_err()
    );
    assert_eq!(queries.borrow().len(), 1);
}

#[test]
fn blocked_origin_and_vertical_arithmetic_fail_transactionally() {
    let origin = position(RegionCoord::ZERO, 0, 0);
    let destination = position(RegionCoord::ZERO, 1, 0);
    let queries = RefCell::new(Vec::new());
    assert!(
        advance_terrain_body(motion(origin, HALF_HEIGHT, 0), 1, 0, 0, 0, |queried| {
            queries.borrow_mut().push(queried);
            if queried == destination {
                Ok(terrain(1))
            } else {
                Err(anyhow!("retained origin unavailable"))
            }
        },)
        .is_err()
    );
    assert_eq!(&*queries.borrow(), &[destination, origin]);

    let velocity_overflow = motion(origin, HALF_HEIGHT, i32::MAX);
    assert!(advance_terrain_body(velocity_overflow, 0, 0, 0, 1, |_| Ok(terrain(0))).is_err());
    assert_eq!(velocity_overflow.step_velocity_q16(), i32::MAX);

    let contact_overflow = motion(origin, i32::MAX, 0);
    assert!(
        advance_terrain_body(contact_overflow, 0, 0, i32::MAX, 0, |_| {
            Ok(terrain(i32::MAX))
        })
        .is_err()
    );
    assert_eq!(contact_overflow.body().center_height_numerator(), i32::MAX);
}
