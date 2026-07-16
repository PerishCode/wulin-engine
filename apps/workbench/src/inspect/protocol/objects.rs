use serde::Deserialize;
use serde_json::Value;

use super::{ControlKind, ParsedControl, ProtocolError, decode};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectResolvePayload {
    source_namespace: String,
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

pub(super) fn resolve(value: Value) -> ParsedControl {
    let payload: ObjectResolvePayload = decode(value)?;
    Ok(ControlKind::CanonicalObjectResolve {
        source_namespace: decode_source_namespace(&payload.source_namespace)?,
        region_x: payload.region_x,
        region_z: payload.region_z,
        authored_local_id: payload.authored_local_id,
    })
}

fn decode_source_namespace(value: &str) -> Result<[u8; 32], ProtocolError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProtocolError {
            code: "invalid_payload",
            message: "object source namespace must be exactly 64 lowercase hexadecimal digits"
                .into(),
        });
    }
    let mut decoded = [0_u8; 32];
    for (index, byte) in decoded.iter_mut().enumerate() {
        let pair = &value[index * 2..index * 2 + 2];
        *byte = u8::from_str_radix(pair, 16).expect("validated hexadecimal pair must decode");
    }
    Ok(decoded)
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
