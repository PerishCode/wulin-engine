use engine_runtime::{
    RegionCoord, RetainedTerrainBody, Runtime, TerrainBody, TerrainBodyHandle, TerrainBodyMotion,
    TerrainPosition,
};
use serde_json::{Value, json};

use super::{ControlResult, protocol_error};

pub(super) struct MotionPayload {
    pub region_x: i64,
    pub region_z: i64,
    pub local_x_q9: i32,
    pub local_z_q9: i32,
    pub center_height_numerator: i32,
    pub half_height_numerator: i32,
    pub step_velocity_q16: i32,
}

pub(super) fn spawn(runtime: &mut Runtime, payload: MotionPayload) -> ControlResult {
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
    .and_then(|motion| runtime.spawn_terrain_body(motion))
    .map(|retained| lifecycle_response("spawn", retained, 1))
    .map_err(|error| protocol_error("terrain_body_lifecycle_failed", error))
}

pub(super) fn read(runtime: &Runtime, generation: u64) -> ControlResult {
    TerrainBodyHandle::new(generation)
        .and_then(|handle| runtime.read_terrain_body(handle))
        .map(|retained| lifecycle_response("read", retained, 1))
        .map_err(|error| protocol_error("terrain_body_lifecycle_failed", error))
}

pub(super) fn despawn(runtime: &mut Runtime, generation: u64) -> ControlResult {
    TerrainBodyHandle::new(generation)
        .and_then(|handle| runtime.despawn_terrain_body(handle))
        .map(|retained| lifecycle_response("despawn", retained, 0))
        .map_err(|error| protocol_error("terrain_body_lifecycle_failed", error))
}

pub(super) fn advance(
    runtime: &mut Runtime,
    generation: u64,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
) -> ControlResult {
    TerrainBodyHandle::new(generation)
        .and_then(|handle| {
            runtime.advance_retained_terrain_body(
                handle,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            )
        })
        .map(|retained_advance| {
            json!({
                "revision": "transactional-retained-terrain-body-advance-v1",
                "terrainQueryCount": retained_advance.advance.terrain_query_count,
                "retainedAdvance": retained_advance,
                "perOperationAllocationBytes": 0,
                "sourceReadCount": 0,
                "gpuCopyCount": 0,
                "gpuReadbackCount": 0,
                "fenceWaitCount": 0,
                "synchronizationCount": 0,
                "scheduleMutationCount": 0,
                "presentationMutationCount": 0,
                "frameCount": 0,
                "rendererWorkCount": 0,
            })
        })
        .map_err(|error| protocol_error("retained_terrain_advance_failed", error))
}

pub(super) fn batch(
    runtime: &mut Runtime,
    generation: u64,
    step_count: u32,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
) -> ControlResult {
    TerrainBodyHandle::new(generation)
        .and_then(|handle| {
            runtime.advance_retained_body_batch(
                handle,
                step_count,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            )
        })
        .map(|retained_batch| {
            json!({
                "revision": "transactional-retained-terrain-body-batch-v1",
                "stepCount": retained_batch.step_count,
                "terrainQueryCount": retained_batch.terrain_query_count,
                "retainedBatch": retained_batch,
                "perOperationAllocationBytes": 0,
                "sourceReadCount": 0,
                "gpuCopyCount": 0,
                "gpuReadbackCount": 0,
                "fenceWaitCount": 0,
                "synchronizationCount": 0,
                "scheduleMutationCount": 0,
                "presentationMutationCount": 0,
                "frameCount": 0,
                "rendererWorkCount": 0,
            })
        })
        .map_err(|error| protocol_error("retained_terrain_batch_failed", error))
}

pub(super) fn simulation_advance(
    runtime: &mut Runtime,
    generation: u64,
    elapsed_nanoseconds: u64,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
) -> ControlResult {
    TerrainBodyHandle::new(generation)
        .and_then(|handle| {
            runtime.advance_simulation_body(
                handle,
                elapsed_nanoseconds,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            )
        })
        .map(|advance| {
            json!({
                "revision": "transactional-simulation-body-advance-v1",
                "stepCount": advance.simulation.step_count,
                "terrainQueryCount": advance.body.terrain_query_count,
                "simulationBodyAdvance": advance,
                "perOperationAllocationBytes": 0,
                "sourceReadCount": 0,
                "gpuCopyCount": 0,
                "gpuReadbackCount": 0,
                "fenceWaitCount": 0,
                "synchronizationCount": 0,
                "scheduleCommitCount": 1,
                "retainedCommitCount": 1,
                "presentationMutationCount": 0,
                "frameCount": 0,
                "rendererWorkCount": 0,
            })
        })
        .map_err(|error| protocol_error("simulation_body_advance_failed", error))
}

fn lifecycle_response(operation: &str, retained: RetainedTerrainBody, live_count: u32) -> Value {
    json!({
        "revision": "retained-terrain-body-lifecycle-v1",
        "operation": operation,
        "capacity": 1,
        "liveCount": live_count,
        "retained": retained,
        "perOperationAllocationBytes": 0,
        "terrainQueryCount": 0,
        "sourceReadCount": 0,
        "gpuCopyCount": 0,
        "gpuReadbackCount": 0,
        "fenceWaitCount": 0,
        "synchronizationCount": 0,
        "scheduleMutationCount": 0,
        "presentationMutationCount": 0,
        "frameCount": 0,
        "rendererWorkCount": 0,
    })
}
