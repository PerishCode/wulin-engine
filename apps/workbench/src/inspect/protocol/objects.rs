use serde::Deserialize;
use serde_json::Value;

use super::{ControlKind, ParsedControl, decode};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectQueryPayload {
    region_x: i64,
    region_z: i64,
    authored_local_id: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectNearestPayload {
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    max_distance_q9: u32,
}

pub(super) fn query(value: Value) -> ParsedControl {
    let payload: ObjectQueryPayload = decode(value)?;
    Ok(ControlKind::CanonicalObjectQuery {
        region_x: payload.region_x,
        region_z: payload.region_z,
        authored_local_id: payload.authored_local_id,
    })
}

pub(super) fn nearest(value: Value) -> ParsedControl {
    let payload: ObjectNearestPayload = decode(value)?;
    Ok(ControlKind::CanonicalObjectNearest {
        region_x: payload.region_x,
        region_z: payload.region_z,
        local_x_q9: payload.local_x_q9,
        local_z_q9: payload.local_z_q9,
        max_distance_q9: payload.max_distance_q9,
    })
}
