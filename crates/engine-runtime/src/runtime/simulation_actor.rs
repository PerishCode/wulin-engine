use anyhow::Result;
use serde::Serialize;

use super::motion_batch::{MotionBatch, MotionBatchCommand, advance_motion_batch};
use super::{ActorPresentation, RuntimeActor};
use crate::terrain_query::{TerrainBodyMotion, TerrainHeight, TerrainPosition};
use crate::timeline::{SimulationAdvance, SimulationSchedule};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorStateTransition {
    pub input: RuntimeActor,
    pub output: RuntimeActor,
    pub step_count: u32,
    pub terrain_query_count: u32,
    pub last_step_grounded: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorSimulationAdvance {
    pub simulation: SimulationAdvance,
    pub actor: ActorStateTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorSimulationCommand {
    pub delta_x_q9: i32,
    pub delta_z_q9: i32,
    pub step_up_limit_q16: i32,
    pub initial_step_velocity_delta_q16: i32,
    pub step_acceleration_q16: i32,
    pub presentation: ActorPresentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorSimulationRenderBlock {
    pub prepared_step_count: u32,
    pub terrain_query_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", content = "evidence", rename_all = "kebab-case")]
pub enum ActorSimulationOutcome {
    Advanced(ActorSimulationAdvance),
    RenderBlocked(ActorSimulationRenderBlock),
}

pub(crate) struct PreparedSimulationActor {
    pub schedule: SimulationSchedule,
    pub simulation: SimulationAdvance,
    pub motion: MotionBatch,
}

pub(crate) fn prepare_simulation_actor(
    mut schedule: SimulationSchedule,
    input: TerrainBodyMotion,
    elapsed_nanoseconds: u64,
    command: ActorSimulationCommand,
    query: impl FnMut(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<PreparedSimulationActor> {
    command.presentation.validate()?;
    let simulation = schedule.advance(elapsed_nanoseconds)?;
    let motion = advance_motion_batch(
        input,
        simulation.step_count,
        MotionBatchCommand {
            delta_x_q9: command.delta_x_q9,
            delta_z_q9: command.delta_z_q9,
            step_up_limit_q16: command.step_up_limit_q16,
            initial_step_velocity_delta_q16: command.initial_step_velocity_delta_q16,
            step_acceleration_q16: command.step_acceleration_q16,
        },
        query,
    )?;
    Ok(PreparedSimulationActor {
        schedule,
        simulation,
        motion,
    })
}

#[cfg(test)]
#[path = "../../tests/private/simulation_actor.rs"]
mod tests;
