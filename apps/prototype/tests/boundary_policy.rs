#[path = "../src/boundary.rs"]
mod boundary;
#[allow(dead_code)]
#[path = "../src/locomotion.rs"]
mod locomotion;

use engine_runtime::{RegionCoord, TerrainPosition};
use reference_host::bootstrap::PlayableRegionBounds;

fn position(region: RegionCoord, local_x_q9: i32, local_z_q9: i32) -> TerrainPosition {
    TerrainPosition::new(region, local_x_q9, local_z_q9).unwrap()
}

fn bounds(minimum: RegionCoord, maximum: RegionCoord) -> PlayableRegionBounds {
    PlayableRegionBounds::new(minimum, maximum).unwrap()
}

fn command(delta_x_q9: i32, delta_z_q9: i32) -> locomotion::Command {
    locomotion::Command {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 32_768,
        running: false,
    }
}

fn run_command(delta_x_q9: i32, delta_z_q9: i32) -> locomotion::Command {
    locomotion::Command {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 32_768,
        running: delta_x_q9 != 0 || delta_z_q9 != 0,
    }
}

#[test]
fn exact_maximum_batch_contact_is_admitted_and_one_q9_crossing_is_reduced() {
    let extent = bounds(RegionCoord::ZERO, RegionCoord::ZERO);
    assert_eq!(
        boundary::admit(
            position(RegionCoord::ZERO, -3840, 3839),
            extent,
            command(-32, 32),
        )
        .unwrap(),
        command(-32, 32)
    );
    assert_eq!(
        boundary::admit(
            position(RegionCoord::ZERO, -3841, 3840),
            extent,
            command(-32, 32),
        )
        .unwrap(),
        command(0, 0)
    );
}

#[test]
fn unsafe_axis_is_reduced_independently_and_inward_motion_remains_live() {
    let extent = bounds(RegionCoord::new(-3, -5), RegionCoord::new(7, 11));
    assert_eq!(
        boundary::admit(
            position(RegionCoord::new(7, 4), 3912, 0),
            extent,
            command(23, -23),
        )
        .unwrap(),
        command(0, -23)
    );
    assert_eq!(
        boundary::admit(
            position(RegionCoord::new(7, 4), 3912, 0),
            extent,
            command(-32, 0),
        )
        .unwrap(),
        command(-32, 0)
    );
}

#[test]
fn run_uses_the_exact_maximum_batch_candidate_and_clears_when_fully_reduced() {
    let extent = bounds(RegionCoord::ZERO, RegionCoord::ZERO);
    assert_eq!(
        boundary::admit(
            position(RegionCoord::ZERO, 0, 3583),
            extent,
            run_command(0, 64),
        )
        .unwrap(),
        run_command(0, 64)
    );
    assert_eq!(
        boundary::admit(
            position(RegionCoord::ZERO, 0, 3584),
            extent,
            run_command(0, 64),
        )
        .unwrap(),
        run_command(0, 0)
    );
    assert_eq!(
        boundary::admit(
            position(RegionCoord::ZERO, 3736, 0),
            extent,
            run_command(45, -45),
        )
        .unwrap(),
        run_command(0, -45)
    );
}

#[test]
fn signed_far_regions_and_stationary_commands_are_exact() {
    let base = RegionCoord::new(1_i64 << 40, -(1_i64 << 40));
    let extent = bounds(base, base);
    let origin = position(base, 0, 0);
    assert_eq!(
        boundary::admit(origin, extent, command(0, 0)).unwrap(),
        command(0, 0)
    );
    assert_eq!(
        boundary::admit(origin, extent, command(32, -32)).unwrap(),
        command(32, -32)
    );
}

#[test]
fn unrepresentable_maximum_candidate_fails_without_an_admitted_command() {
    let region = RegionCoord::new(i64::MAX, 0);
    let extent = bounds(region, region);
    assert!(boundary::admit(position(region, 4095, 0), extent, command(1, 0)).is_err());
}
