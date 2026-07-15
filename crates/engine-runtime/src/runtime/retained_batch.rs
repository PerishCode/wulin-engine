use anyhow::{Context, Result, anyhow, ensure};
use serde::Serialize;

use super::RetainedTerrainBody;
use crate::terrain_query::{
    TerrainBodyMotion, TerrainHeight, TerrainPosition, advance_terrain_body,
};
use crate::timeline::SIMULATION_MAX_STEPS_PER_ADVANCE;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedTerrainBodyBatch {
    pub input: RetainedTerrainBody,
    pub output: RetainedTerrainBody,
    pub step_count: u32,
    pub terrain_query_count: u32,
}

pub(crate) struct MotionBatch {
    pub output: TerrainBodyMotion,
    pub terrain_query_count: u32,
}

pub(crate) fn advance_motion_batch(
    input: TerrainBodyMotion,
    step_count: u32,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
    mut query: impl FnMut(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<MotionBatch> {
    ensure!(
        step_count <= SIMULATION_MAX_STEPS_PER_ADVANCE,
        "retained terrain body batch step count must be in [0, {SIMULATION_MAX_STEPS_PER_ADVANCE}]"
    );
    let mut output = input;
    let mut terrain_query_count = 0_u32;
    for index in 0..step_count {
        let advance = advance_terrain_body(
            output,
            delta_x_q9,
            delta_z_q9,
            step_up_limit_q16,
            step_acceleration_q16,
            &mut query,
        )
        .map_err(|error| {
            anyhow!(
                "retained terrain body batch step {} of {step_count} failed: {error:#}",
                index + 1
            )
        })?;
        output = advance.output;
        terrain_query_count = terrain_query_count
            .checked_add(advance.terrain_query_count)
            .context("retained terrain body batch query count overflowed")?;
    }
    Ok(MotionBatch {
        output,
        terrain_query_count,
    })
}

#[cfg(test)]
#[path = "../../tests/private/retained_batch.rs"]
mod tests;
