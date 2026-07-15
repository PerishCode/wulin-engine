use anyhow::{Context, Result};
use serde::Serialize;

use super::{
    TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainBody, TerrainBodyContact, TerrainContactClassification,
    TerrainHeight, resolve_body_contact,
};
use crate::timeline::SIMULATION_STEPS_PER_SECOND;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyMotion {
    body: TerrainBody,
    step_velocity_q16: i32,
}

impl TerrainBodyMotion {
    pub const fn new(body: TerrainBody, step_velocity_q16: i32) -> Self {
        Self {
            body,
            step_velocity_q16,
        }
    }

    pub const fn body(self) -> TerrainBody {
        self.body
    }

    pub const fn step_velocity_q16(self) -> i32 {
        self.step_velocity_q16
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyStep {
    pub input: TerrainBodyMotion,
    pub step_acceleration_q16: i32,
    pub predicted_body: TerrainBody,
    pub contact: TerrainBodyContact,
    pub output: TerrainBodyMotion,
    pub grounded: bool,
    pub height_denominator: u32,
    pub steps_per_second: u64,
}

pub(crate) fn integrate_terrain_body_step(
    input: TerrainBodyMotion,
    step_acceleration_q16: i32,
    terrain: TerrainHeight,
) -> Result<TerrainBodyStep> {
    let next_velocity = i64::from(input.step_velocity_q16)
        .checked_add(i64::from(step_acceleration_q16))
        .context("terrain body vertical velocity overflowed signed 64-bit arithmetic")?;
    let next_velocity = i32::try_from(next_velocity)
        .context("terrain body vertical velocity is outside the signed 32-bit Q16 range")?;
    let predicted_center = i64::from(input.body.center_height_numerator())
        .checked_add(i64::from(next_velocity))
        .context("terrain body predicted center overflowed signed 64-bit arithmetic")?;
    let predicted_body = TerrainBody::new(
        input.body.position(),
        i32::try_from(predicted_center)
            .context("terrain body predicted center is outside the signed 32-bit Q16 range")?,
        input.body.half_height_numerator(),
    )?;
    let contact = resolve_body_contact(predicted_body, terrain)?;
    let grounded =
        contact.classification != TerrainContactClassification::Separated && next_velocity <= 0;
    let output = TerrainBodyMotion::new(
        contact.resolved_body,
        if grounded { 0 } else { next_velocity },
    );

    Ok(TerrainBodyStep {
        input,
        step_acceleration_q16,
        predicted_body,
        contact,
        output,
        grounded,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        steps_per_second: SIMULATION_STEPS_PER_SECOND,
    })
}

#[cfg(test)]
#[path = "../../tests/private/terrain_motion.rs"]
mod tests;
