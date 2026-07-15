use serde::Deserialize;
use serde_json::Value;

use super::{ControlKind, ParsedControl, decode};

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
struct BodyStepPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
    step_velocity_q16: i32,
    step_acceleration_q16: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyTranslatePayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
    step_velocity_q16: i32,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyAdvancePayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
    step_velocity_q16: i32,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodySpawnPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    center_height_numerator: i32,
    half_height_numerator: i32,
    step_velocity_q16: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyHandlePayload {
    generation: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedAdvancePayload {
    generation: u64,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    step_acceleration_q16: i32,
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

pub(super) fn body_step(value: Value) -> ParsedControl {
    let payload: BodyStepPayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyStep {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
        step_velocity_q16: payload.step_velocity_q16,
        step_acceleration_q16: payload.step_acceleration_q16,
    })
}

pub(super) fn body_translate(value: Value) -> ParsedControl {
    let payload: BodyTranslatePayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyTranslate {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
        step_velocity_q16: payload.step_velocity_q16,
        delta_x_q9: payload.delta_x_q9,
        delta_z_q9: payload.delta_z_q9,
        step_up_limit_q16: payload.step_up_limit_q16,
    })
}

pub(super) fn body_advance(value: Value) -> ParsedControl {
    let payload: BodyAdvancePayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyAdvance {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
        step_velocity_q16: payload.step_velocity_q16,
        delta_x_q9: payload.delta_x_q9,
        delta_z_q9: payload.delta_z_q9,
        step_up_limit_q16: payload.step_up_limit_q16,
        step_acceleration_q16: payload.step_acceleration_q16,
    })
}

pub(super) fn body_spawn(value: Value) -> ParsedControl {
    let payload: BodySpawnPayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodySpawn {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        center_height_numerator: payload.center_height_numerator,
        half_height_numerator: payload.half_height_numerator,
        step_velocity_q16: payload.step_velocity_q16,
    })
}

pub(super) fn body_read(value: Value) -> ParsedControl {
    let payload: BodyHandlePayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyRead {
        generation: payload.generation,
    })
}

pub(super) fn body_despawn(value: Value) -> ParsedControl {
    let payload: BodyHandlePayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyDespawn {
        generation: payload.generation,
    })
}

pub(super) fn body_retained_advance(value: Value) -> ParsedControl {
    let payload: RetainedAdvancePayload = decode(value)?;
    Ok(ControlKind::CanonicalTerrainBodyRetainedAdvance {
        generation: payload.generation,
        delta_x_q9: payload.delta_x_q9,
        delta_z_q9: payload.delta_z_q9,
        step_up_limit_q16: payload.step_up_limit_q16,
        step_acceleration_q16: payload.step_acceleration_q16,
    })
}
