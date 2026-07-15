use anyhow::{Context, Result, ensure};
use engine_runtime::{
    ActorPresentation, TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainBody, TerrainBodyMotion,
    TerrainHeight, TerrainPosition,
};

const HALF_HEIGHT_Q16: i32 = 65_536;

// Nearest integer Q16-per-step acceleration to -9.81 m/s² at the fixed 60 Hz schedule.
pub(crate) const GRAVITY_STEP_ACCELERATION_Q16: i32 = -179;

pub(crate) fn initial_motion(
    position: TerrainPosition,
    terrain: TerrainHeight,
) -> Result<TerrainBodyMotion> {
    ensure!(
        terrain.height_denominator == TERRAIN_BODY_HEIGHT_DENOMINATOR,
        "prototype initial terrain height denominator diverged"
    );
    let center_height_numerator = terrain
        .height_numerator
        .checked_add(HALF_HEIGHT_Q16)
        .context("prototype initial actor center height overflowed")?;
    let body = TerrainBody::new(position, center_height_numerator, HALF_HEIGHT_Q16)?;
    Ok(TerrainBodyMotion::new(body, 0))
}

pub(crate) const fn initial_presentation() -> ActorPresentation {
    ActorPresentation::animated(7, 63, 0, 1, 0, 0)
}
