#[path = "../src/actor.rs"]
mod actor;
#[path = "../src/camera.rs"]
mod camera;

use engine_runtime::{
    RegionCoord, SIMULATION_STEPS_PER_SECOND, TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainHeight,
    TerrainPosition, TerrainTriangle,
};

#[test]
fn gravity_is_nearest_fixed_step_encoding_of_earth_gravity() {
    assert_eq!(actor::GRAVITY_STEP_ACCELERATION_Q16, -179);

    let actual_centimeters_q16 = -i64::from(actor::GRAVITY_STEP_ACCELERATION_Q16)
        * i64::try_from(SIMULATION_STEPS_PER_SECOND).unwrap()
        * i64::try_from(SIMULATION_STEPS_PER_SECOND).unwrap()
        * 100;
    let earth_centimeters_q16 = 981_i64 * i64::from(TERRAIN_BODY_HEIGHT_DENOMINATOR);
    let half_encoding_step_centimeters_q16 = i64::try_from(SIMULATION_STEPS_PER_SECOND).unwrap()
        * i64::try_from(SIMULATION_STEPS_PER_SECOND).unwrap()
        * 50;
    assert!(
        (actual_centimeters_q16 - earth_centimeters_q16).abs()
            <= half_encoding_step_centimeters_q16
    );
}

#[test]
fn camera_rig_is_fixed_actor_relative_policy() {
    assert_eq!(
        camera::POSITION_OFFSET.map(f32::to_bits),
        [9.0, 4.0, 12.0].map(f32::to_bits)
    );
    assert_eq!(
        camera::TARGET_OFFSET.map(f32::to_bits),
        [0.0, -1.0, -3.0].map(f32::to_bits)
    );
    assert_eq!(camera::VERTICAL_FOV_DEGREES.to_bits(), 60.0_f32.to_bits());
}

#[test]
fn motion_touches_terrain() {
    let position =
        TerrainPosition::new(RegionCoord::new(1_i64 << 40, -(1_i64 << 40)), 0, 0).unwrap();
    let terrain = TerrainHeight {
        height_numerator: -131_072,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::Diagonal,
    };

    let motion = actor::initial_motion(position, terrain).unwrap();
    assert_eq!(motion.body().position(), position);
    assert_eq!(motion.body().center_height_numerator(), -65_536);
    assert_eq!(motion.body().half_height_numerator(), 65_536);
    assert_eq!(motion.step_velocity_q16(), 0);
}

#[test]
fn invalid_heights_fail() {
    let position = TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap();
    let invalid = TerrainHeight {
        height_numerator: 0,
        height_denominator: 1,
        triangle: TerrainTriangle::First,
    };
    assert!(actor::initial_motion(position, invalid).is_err());

    let overflow = TerrainHeight {
        height_numerator: i32::MAX,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::Second,
    };
    assert!(actor::initial_motion(position, overflow).is_err());
}
