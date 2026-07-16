use anyhow::{Context, Result, anyhow, ensure};

use crate::terrain_query::{
    TerrainBodyMotion, TerrainHeight, TerrainPosition, advance_terrain_body,
};
use crate::timeline::SIMULATION_MAX_STEPS_PER_ADVANCE;

pub(crate) struct MotionBatch {
    pub output: TerrainBodyMotion,
    pub terrain_query_count: u32,
}

pub(crate) struct MotionBatchCommand {
    pub delta_x_q9: i32,
    pub delta_z_q9: i32,
    pub step_up_limit_q16: i32,
    pub initial_step_velocity_delta_q16: i32,
    pub step_acceleration_q16: i32,
}

pub(crate) fn advance_motion_batch(
    input: TerrainBodyMotion,
    step_count: u32,
    command: MotionBatchCommand,
    mut query: impl FnMut(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<MotionBatch> {
    ensure!(
        step_count <= SIMULATION_MAX_STEPS_PER_ADVANCE,
        "terrain-body motion batch step count must be in [0, {SIMULATION_MAX_STEPS_PER_ADVANCE}]"
    );
    let mut output = input;
    let mut terrain_query_count = 0_u32;
    for index in 0..step_count {
        if index == 0 {
            let step_velocity_q16 = output
                .step_velocity_q16()
                .checked_add(command.initial_step_velocity_delta_q16)
                .ok_or_else(|| {
                    anyhow!(
                        "terrain-body motion batch step 1 of {step_count} failed: initial step velocity delta is outside the signed 32-bit Q16 range"
                    )
                })?;
            output = TerrainBodyMotion::new(output.body(), step_velocity_q16);
        }
        let advance = advance_terrain_body(
            output,
            command.delta_x_q9,
            command.delta_z_q9,
            command.step_up_limit_q16,
            command.step_acceleration_q16,
            &mut query,
        )
        .map_err(|error| {
            anyhow!(
                "terrain-body motion batch step {} of {step_count} failed: {error:#}",
                index + 1
            )
        })?;
        output = advance.output;
        terrain_query_count = terrain_query_count
            .checked_add(advance.terrain_query_count)
            .context("terrain-body motion batch query count overflowed")?;
    }
    Ok(MotionBatch {
        output,
        terrain_query_count,
    })
}

#[cfg(test)]
#[path = "../../tests/private/motion_batch.rs"]
mod tests;
