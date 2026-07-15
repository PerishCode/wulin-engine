use anyhow::{Result, ensure};
use serde::Serialize;

use super::{
    TERRAIN_BODY_HEIGHT_DENOMINATOR, TERRAIN_POSITION_DENOMINATOR, TerrainBody, TerrainBodyContact,
    TerrainBodyMotion, TerrainHeight, TerrainPosition, resolve_body_contact,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyTranslation {
    pub input: TerrainBodyMotion,
    pub delta_x_q9: i32,
    pub delta_z_q9: i32,
    pub step_up_limit_q16: i32,
    pub candidate_body: TerrainBody,
    pub contact: TerrainBodyContact,
    pub output: TerrainBodyMotion,
    pub blocked: bool,
    pub position_denominator: i32,
    pub height_denominator: u32,
}

pub(crate) fn translate_terrain_body(
    input: TerrainBodyMotion,
    delta_x_q9: i32,
    delta_z_q9: i32,
    step_up_limit_q16: i32,
    query: impl FnOnce(TerrainPosition) -> Result<TerrainHeight>,
) -> Result<TerrainBodyTranslation> {
    ensure!(
        step_up_limit_q16 >= 0,
        "terrain body step-up limit must be nonnegative"
    );
    let position = input
        .body()
        .position()
        .translated_q9(delta_x_q9, delta_z_q9)?;
    let candidate_body = TerrainBody::new(
        position,
        input.body().center_height_numerator(),
        input.body().half_height_numerator(),
    )?;
    let contact = resolve_body_contact(candidate_body, query(position)?)?;
    let blocked = contact.correction_numerator > i64::from(step_up_limit_q16);
    let output = if blocked {
        input
    } else {
        TerrainBodyMotion::new(contact.resolved_body, input.step_velocity_q16())
    };

    Ok(TerrainBodyTranslation {
        input,
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16,
        candidate_body,
        contact,
        output,
        blocked,
        position_denominator: TERRAIN_POSITION_DENOMINATOR,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
    })
}

#[cfg(test)]
#[path = "../../tests/private/terrain_translation.rs"]
mod tests;
