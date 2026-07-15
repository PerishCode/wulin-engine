use anyhow::Result;
use serde::Serialize;

use super::{
    TERRAIN_BODY_HEIGHT_DENOMINATOR, TERRAIN_POSITION_DENOMINATOR, TerrainBodyMotion,
    TerrainBodyStep, TerrainBodyTranslation, TerrainHeight, TerrainPosition,
    integrate_terrain_body_step, translate_terrain_body,
};
use crate::timeline::SIMULATION_STEPS_PER_SECOND;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyAdvance {
    pub input: TerrainBodyMotion,
    pub delta_x_q9: i32,
    pub delta_z_q9: i32,
    pub step_up_limit_q16: i32,
    pub step_acceleration_q16: i32,
    pub translation: TerrainBodyTranslation,
    pub vertical_step: TerrainBodyStep,
    pub output: TerrainBodyMotion,
    pub grounded: bool,
    pub terrain_query_count: u32,
    pub position_denominator: i32,
    pub height_denominator: u32,
    pub steps_per_second: u64,
}

pub(crate) fn advance_terrain_body(
    input: TerrainBodyMotion,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
    mut query: impl FnMut(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<TerrainBodyAdvance> {
    let mut terrain_query_count = 0;
    let translation = translate_terrain_body(
        input,
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16,
        |position| {
            terrain_query_count += 1;
            query(position)
        },
    )?;
    let vertical_input = translation.output;
    let vertical_position = vertical_input.body().position();
    let terrain = if vertical_position == translation.candidate_body.position() {
        translation.contact.terrain
    } else {
        terrain_query_count += 1;
        query(vertical_position)?
    };
    let vertical_step =
        integrate_terrain_body_step(vertical_input, step_acceleration_q16, terrain)?;

    Ok(TerrainBodyAdvance {
        input,
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16,
        step_acceleration_q16,
        translation,
        vertical_step,
        output: vertical_step.output,
        grounded: vertical_step.grounded,
        terrain_query_count,
        position_denominator: TERRAIN_POSITION_DENOMINATOR,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        steps_per_second: SIMULATION_STEPS_PER_SECOND,
    })
}

#[cfg(test)]
#[path = "../../tests/private/terrain_advance.rs"]
mod tests;
