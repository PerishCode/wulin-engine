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

pub(super) fn query(value: Value) -> ParsedControl {
    let payload: ObjectQueryPayload = decode(value)?;
    Ok(ControlKind::CanonicalObjectQuery {
        region_x: payload.region_x,
        region_z: payload.region_z,
        authored_local_id: payload.authored_local_id,
    })
}
