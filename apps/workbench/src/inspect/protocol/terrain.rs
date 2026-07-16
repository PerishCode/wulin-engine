use serde::Deserialize;
use serde_json::Value;

use super::{ActorSpawnControl, ControlKind, ParsedControl, SimulationActorControl, decode};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeightPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContactPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorSpawnPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
    step_velocity_q16: i32,
    archetype: u32,
    material: u32,
    yaw_q16: u32,
    animation: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorHandlePayload {
    generation: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimulationActorPayload {
    generation: u64,
    elapsed_nanoseconds: u64,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    initial_step_velocity_delta_q16: i32,
    step_acceleration_q16: i32,
    archetype: u32,
    material: u32,
    yaw_q16: u32,
    animation: u32,
}

pub(super) fn height(value: Value) -> ParsedControl {
    let payload: HeightPayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainHeight {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
    })
}

pub(super) fn contact(value: Value) -> ParsedControl {
    let payload: ContactPayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainContact {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
    })
}

pub(super) fn actor_spawn(value: Value) -> ParsedControl {
    let payload: ActorSpawnPayload = decode(value)?;
    Ok(ControlKind::ActorSpawn(ActorSpawnControl {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
        step_velocity_q16: payload.step_velocity_q16,
        archetype: payload.archetype,
        material: payload.material,
        yaw_q16: payload.yaw_q16,
        animation: payload.animation,
    }))
}

pub(super) fn actor_read(value: Value) -> ParsedControl {
    let payload: ActorHandlePayload = decode(value)?;
    Ok(ControlKind::ActorRead {
        generation: payload.generation,
    })
}

pub(super) fn actor_despawn(value: Value) -> ParsedControl {
    let payload: ActorHandlePayload = decode(value)?;
    Ok(ControlKind::ActorDespawn {
        generation: payload.generation,
    })
}

pub(super) fn simulation_actor_advance(value: Value) -> ParsedControl {
    let payload: SimulationActorPayload = decode(value)?;
    Ok(ControlKind::SimulationActorAdvance(
        SimulationActorControl {
            generation: payload.generation,
            elapsed_nanoseconds: payload.elapsed_nanoseconds,
            delta_x_q9: payload.delta_x_q9,
            delta_z_q9: payload.delta_z_q9,
            step_up_limit_q16: payload.step_up_limit_q16,
            initial_step_velocity_delta_q16: payload.initial_step_velocity_delta_q16,
            step_acceleration_q16: payload.step_acceleration_q16,
            archetype: payload.archetype,
            material: payload.material,
            yaw_q16: payload.yaw_q16,
            animation: payload.animation,
        },
    ))
}
