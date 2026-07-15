use super::*;
use crate::RegionCoord;
use crate::terrain_query::{
    TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainQueryPosition, TerrainTriangle,
};
use crate::timeline::SimulationSchedule;

const HALF_HEIGHT: i32 = 65_536;

fn body(center_height_numerator: i32) -> TerrainBody {
    TerrainBody::new(
        TerrainQueryPosition::new(RegionCoord::ZERO, 0, 0).unwrap(),
        center_height_numerator,
        HALF_HEIGHT,
    )
    .unwrap()
}

fn terrain(height_numerator: i32) -> TerrainHeight {
    TerrainHeight {
        height_numerator,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::Diagonal,
    }
}

fn apply_steps(mut motion: TerrainBodyMotion, acceleration: i32, count: u32) -> TerrainBodyMotion {
    for _ in 0..count {
        motion = integrate_terrain_body_step(motion, acceleration, terrain(0))
            .unwrap()
            .output;
    }
    motion
}

fn apply_elapsed(intervals: &[u64], mut motion: TerrainBodyMotion) -> TerrainBodyMotion {
    let mut schedule = SimulationSchedule::new();
    for elapsed in intervals {
        let batch = schedule.advance(*elapsed).unwrap();
        motion = apply_steps(motion, -180, batch.step_count);
    }
    motion
}

#[test]
fn falling_landing_and_ground_hold_are_exact() {
    let mut motion = TerrainBodyMotion::new(body(HALF_HEIGHT + 1_000), 0);
    for (index, expected_center) in [900, 700, 400].into_iter().enumerate() {
        let step = integrate_terrain_body_step(motion, -100, terrain(0)).unwrap();
        assert!(!step.grounded);
        assert_eq!(
            step.output.body().center_height_numerator(),
            HALF_HEIGHT + expected_center
        );
        assert_eq!(step.output.step_velocity_q16(), -100 * (index as i32 + 1));
        motion = step.output;
    }
    let landing = integrate_terrain_body_step(motion, -100, terrain(0)).unwrap();
    assert!(landing.grounded);
    assert_eq!(
        landing.contact.classification,
        TerrainContactClassification::Touching
    );
    assert_eq!(landing.contact.correction_numerator, 0);
    assert_eq!(landing.output.body().center_height_numerator(), HALF_HEIGHT);
    assert_eq!(landing.output.step_velocity_q16(), 0);

    let held = integrate_terrain_body_step(landing.output, -100, terrain(0)).unwrap();
    assert!(held.grounded);
    assert_eq!(
        held.contact.classification,
        TerrainContactClassification::Penetrating
    );
    assert_eq!(held.contact.correction_numerator, 100);
    assert_eq!(held.output, landing.output);
}

#[test]
fn positive_motion_departs_exact_contact() {
    let touching = TerrainBodyMotion::new(body(HALF_HEIGHT), 200);
    let departure = integrate_terrain_body_step(touching, -100, terrain(0)).unwrap();
    assert!(!departure.grounded);
    assert_eq!(
        departure.contact.classification,
        TerrainContactClassification::Separated
    );
    assert_eq!(
        departure.output.body().center_height_numerator(),
        HALF_HEIGHT + 100
    );
    assert_eq!(departure.output.step_velocity_q16(), 100);

    let resting = TerrainBodyMotion::new(body(HALF_HEIGHT), 0);
    assert!(
        integrate_terrain_body_step(resting, 0, terrain(0))
            .unwrap()
            .grounded
    );
}

#[test]
fn schedule_batch_partition_does_not_change_motion() {
    let initial = TerrainBodyMotion::new(body(HALF_HEIGHT + 300_000), 12_000);
    let coarse = apply_elapsed(&[125_000_000; 8], initial);
    let mut nominal = vec![16_666_666; 20];
    nominal.extend([16_666_667; 40]);
    let ordered = apply_elapsed(&nominal, initial);
    nominal.reverse();
    let reordered = apply_elapsed(&nominal, initial);
    assert_eq!(coarse, ordered);
    assert_eq!(coarse, reordered);
    assert_eq!(apply_elapsed(&[125_000_000; 8], initial), coarse);
}

#[test]
fn failures_return_no_partial_motion() {
    let velocity_overflow = TerrainBodyMotion::new(body(0), i32::MAX);
    assert!(integrate_terrain_body_step(velocity_overflow, 1, terrain(0)).is_err());

    let position_overflow = TerrainBodyMotion::new(body(i32::MAX), 1);
    assert!(integrate_terrain_body_step(position_overflow, 0, terrain(0)).is_err());

    let unrepresentable_contact = TerrainBodyMotion::new(body(i32::MAX), 0);
    assert!(integrate_terrain_body_step(unrepresentable_contact, 0, terrain(i32::MAX)).is_err());
}
