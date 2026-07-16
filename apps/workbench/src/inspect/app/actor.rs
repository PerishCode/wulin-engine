use engine_runtime::{
    ActorHandle, ActorPresentation, ActorSimulationAdvance, ActorSimulationCommand,
    ActorSimulationOutcome, ActorSimulationRenderBlock, RegionCoord, Runtime, RuntimeActor,
    TerrainBody, TerrainBodyMotion, TerrainPosition,
};
use serde_json::{Value, json};

use super::{ControlResult, protocol_error};
use crate::inspect::protocol::{ActorSpawnControl, SimulationActorControl};

pub(super) fn spawn(runtime: &mut Runtime, payload: ActorSpawnControl) -> ControlResult {
    TerrainPosition::new(
        RegionCoord::new(payload.region_x, payload.region_z),
        payload.local_x_q9,
        payload.local_z_q9,
    )
    .and_then(|position| {
        TerrainBody::new(
            position,
            payload.center_height_numerator,
            payload.half_height_numerator,
        )
    })
    .map(|body| TerrainBodyMotion::new(body, payload.step_velocity_q16))
    .and_then(|motion| {
        runtime.spawn_actor(
            motion,
            ActorPresentation {
                archetype: payload.archetype,
                material: payload.material,
                yaw_q16: payload.yaw_q16,
                animation: payload.animation,
            },
        )
    })
    .map(|actor| lifecycle_response("spawn", actor, 1, 1))
    .map_err(|error| protocol_error("actor_lifecycle_failed", error))
}

pub(super) fn read(runtime: &Runtime, generation: u64) -> ControlResult {
    ActorHandle::new(generation)
        .and_then(|handle| runtime.read_actor(handle))
        .map(|actor| lifecycle_response("read", actor, 1, 0))
        .map_err(|error| protocol_error("actor_lifecycle_failed", error))
}

pub(super) fn despawn(runtime: &mut Runtime, generation: u64) -> ControlResult {
    ActorHandle::new(generation)
        .and_then(|handle| runtime.despawn_actor(handle))
        .map(|actor| lifecycle_response("despawn", actor, 0, 1))
        .map_err(|error| protocol_error("actor_lifecycle_failed", error))
}

pub(super) fn simulation_advance(
    runtime: &mut Runtime,
    payload: SimulationActorControl,
) -> ControlResult {
    ActorHandle::new(payload.generation)
        .and_then(|handle| {
            runtime.advance_simulation_actor(
                handle,
                payload.elapsed_nanoseconds,
                ActorSimulationCommand {
                    delta_x_q9: payload.delta_x_q9,
                    delta_z_q9: payload.delta_z_q9,
                    step_up_limit_q16: payload.step_up_limit_q16,
                    initial_step_velocity_delta_q16: payload.initial_step_velocity_delta_q16,
                    step_acceleration_q16: payload.step_acceleration_q16,
                    presentation: ActorPresentation {
                        archetype: payload.archetype,
                        material: payload.material,
                        yaw_q16: payload.yaw_q16,
                        animation: payload.animation,
                    },
                },
            )
        })
        .map(simulation_response)
        .map_err(|error| protocol_error("actor_simulation_advance_failed", error))
}

fn simulation_response(outcome: ActorSimulationOutcome) -> Value {
    match outcome {
        ActorSimulationOutcome::Advanced(advance) => advanced_response(advance),
        ActorSimulationOutcome::RenderBlocked(blocked) => blocked_response(blocked),
    }
}

fn advanced_response(advance: ActorSimulationAdvance) -> Value {
    let presentation_mutation_count =
        u32::from(advance.actor.input.presentation != advance.actor.output.presentation);
    json!({
        "revision": "runtime-actor-simulation-v5",
        "outcome": "advanced",
        "preparedStepCount": advance.simulation.step_count,
        "terrainQueryCount": advance.actor.terrain_query_count,
        "actorSimulationAdvance": advance,
        "perOperationAllocationBytes": 0,
        "sourceReadCount": 0,
        "gpuCopyCount": 0,
        "gpuReadbackCount": 0,
        "fenceWaitCount": 0,
        "synchronizationCount": 0,
        "scheduleCommitCount": 1,
        "actorCommitCount": 1,
        "presentationMutationCount": presentation_mutation_count,
        "frameCount": 0,
        "rendererWorkCount": 0,
    })
}

fn blocked_response(blocked: ActorSimulationRenderBlock) -> Value {
    json!({
        "revision": "runtime-actor-simulation-v5",
        "outcome": "render-blocked",
        "preparedStepCount": blocked.prepared_step_count,
        "terrainQueryCount": blocked.terrain_query_count,
        "perOperationAllocationBytes": 0,
        "sourceReadCount": 0,
        "gpuCopyCount": 0,
        "gpuReadbackCount": 0,
        "fenceWaitCount": 0,
        "synchronizationCount": 0,
        "scheduleCommitCount": 0,
        "actorCommitCount": 0,
        "presentationMutationCount": 0,
        "frameCount": 0,
        "rendererWorkCount": 0,
    })
}

fn lifecycle_response(
    operation: &str,
    actor: RuntimeActor,
    live_count: u32,
    actor_mutation_count: u32,
) -> Value {
    json!({
        "revision": "retained-runtime-actor-v2",
        "operation": operation,
        "capacity": 1,
        "liveCount": live_count,
        "actor": actor,
        "perOperationAllocationBytes": 0,
        "terrainQueryCount": 0,
        "sourceReadCount": 0,
        "gpuCopyCount": 0,
        "gpuReadbackCount": 0,
        "fenceWaitCount": 0,
        "synchronizationCount": 0,
        "scheduleMutationCount": 0,
        "actorMutationCount": actor_mutation_count,
        "presentationMutationCount": 0,
        "frameCount": 0,
        "rendererWorkCount": 0,
    })
}
