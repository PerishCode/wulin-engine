use anyhow::Result;
use serde::Serialize;

use super::RetainedTerrainBodyBatch;
use super::retained_batch::{MotionBatch, advance_motion_batch};
use crate::terrain_query::{TerrainBodyMotion, TerrainHeight, TerrainPosition};
use crate::timeline::{SimulationAdvance, SimulationSchedule};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedSimulationAdvance {
    pub simulation: SimulationAdvance,
    pub body: RetainedTerrainBodyBatch,
}

pub(crate) struct PreparedSimulationBody {
    pub schedule: SimulationSchedule,
    pub simulation: SimulationAdvance,
    pub body: MotionBatch,
}

#[derive(Clone, Copy)]
pub(crate) struct SimulationBodyCommand {
    pub delta_x_q9: i32,
    pub delta_z_q9: i32,
    pub step_up_limit_q16: i32,
    pub step_acceleration_q16: i32,
}

pub(crate) fn prepare_simulation_body(
    mut schedule: SimulationSchedule,
    input: TerrainBodyMotion,
    elapsed_nanoseconds: u64,
    command: SimulationBodyCommand,
    query: impl FnMut(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<PreparedSimulationBody> {
    let simulation = schedule.advance(elapsed_nanoseconds)?;
    let body = advance_motion_batch(
        input,
        simulation.step_count,
        command.delta_x_q9,
        command.delta_z_q9,
        command.step_up_limit_q16,
        command.step_acceleration_q16,
        query,
    )?;
    Ok(PreparedSimulationBody {
        schedule,
        simulation,
        body,
    })
}

#[cfg(test)]
#[path = "../../tests/private/simulation_body.rs"]
mod tests;
