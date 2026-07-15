#[path = "../src/actor.rs"]
mod actor;

use engine_runtime::{
    RegionCoord, TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainHeight, TerrainPosition, TerrainTriangle,
};

#[test]
fn motion_touches_terrain_and_presentation_is_imported() {
    let position =
        TerrainPosition::new(RegionCoord::new(1_i64 << 40, -(1_i64 << 40)), 0, 0).unwrap();
    let terrain = TerrainHeight {
        height_numerator: -131_072,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::Diagonal,
    };

    let motion = actor::initial_motion(position, terrain).unwrap();
    let presentation = actor::initial_presentation();

    assert_eq!(motion.body().position(), position);
    assert_eq!(motion.body().center_height_numerator(), -65_536);
    assert_eq!(motion.body().half_height_numerator(), 65_536);
    assert_eq!(motion.step_velocity_q16(), 0);
    assert_eq!(presentation.archetype, 7);
    assert_eq!(presentation.material, 63);
    assert_eq!(presentation.yaw_q16, 0);
    assert_eq!(presentation.animation_clip(), Some(1));
    assert_eq!(presentation.animation_phase_offset(), Some(0));
    assert_eq!(presentation.animation_variant(), Some(0));
    presentation.validate().unwrap();
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
