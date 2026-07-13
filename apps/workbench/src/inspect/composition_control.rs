use serde::Deserialize;
use serde_json::Value;

use crate::address::GlobalRegionConfig;
use crate::load::LoadConfig;
use crate::rendering::{CompositionFixture, Renderer};

use super::protocol::{ControlKind, ControlResult, ProtocolError};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OrderPayload {
    order: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixturePayload {
    fixture: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GlobalPayload {
    origin_x: i64,
    origin_z: i64,
    center_x: i64,
    center_z: i64,
    active_radius: u32,
}

pub(super) fn parse_order(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload: OrderPayload = decode(value)?;
    match payload.order.as_str() {
        "terrain-first" => Ok(ControlKind::CompositionOrder {
            terrain_first: true,
        }),
        "object-first" => Ok(ControlKind::CompositionOrder {
            terrain_first: false,
        }),
        _ => Err(ProtocolError {
            code: "invalid_payload",
            message: "order must be terrain-first or object-first".into(),
        }),
    }
}

pub(super) fn parse_fixture(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload: FixturePayload = decode(value)?;
    match payload.fixture.as_str() {
        "cell-center" => Ok(ControlKind::CompositionFixture {
            arbitrary_q8: false,
        }),
        "arbitrary-q8" => Ok(ControlKind::CompositionFixture { arbitrary_q8: true }),
        _ => Err(ProtocolError {
            code: "invalid_payload",
            message: "fixture must be cell-center or arbitrary-q8".into(),
        }),
    }
}

pub(super) fn parse_global(value: Value) -> Result<ControlKind, ProtocolError> {
    let payload: GlobalPayload = decode(value)?;
    Ok(ControlKind::CompositionGlobalSchedule(
        payload.origin_x,
        payload.origin_z,
        payload.center_x,
        payload.center_z,
        payload.active_radius,
    ))
}

fn decode<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, ProtocolError> {
    serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })
}

pub fn dispatch(renderer: &mut Renderer, kind: ControlKind) -> ControlResult {
    match kind {
        ControlKind::CompositionStatus => Ok(renderer.composition_status()),
        ControlKind::CompositionSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        } => {
            let config = LoadConfig::new(
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            )
            .map_err(|error| ProtocolError {
                code: "invalid_load_config",
                message: error.to_string(),
            })?;
            unsafe { renderer.schedule_composition(config) }.map_err(stream_error)
        }
        ControlKind::CompositionGlobalSchedule(
            origin_x,
            origin_z,
            center_x,
            center_z,
            active_radius,
        ) => {
            let config =
                GlobalRegionConfig::new(origin_x, origin_z, center_x, center_z, active_radius)
                    .map_err(|error| ProtocolError {
                        code: "invalid_global_composition_config",
                        message: error.to_string(),
                    })?;
            unsafe { renderer.schedule_global_composition(config) }.map_err(stream_error)
        }
        ControlKind::CompositionEnable => renderer
            .enable_composition()
            .map(|()| renderer.composition_status())
            .map_err(|error| ProtocolError {
                code: "composition_unavailable",
                message: error.to_string(),
            }),
        ControlKind::CompositionDisable => {
            renderer.disable_composition();
            Ok(renderer.composition_status())
        }
        ControlKind::CompositionTraversalEnable => renderer
            .enable_composition_traversal()
            .map(|()| renderer.composition_status())
            .map_err(stream_error),
        ControlKind::CompositionTraversalDisable => {
            renderer.disable_composition_traversal();
            Ok(renderer.composition_status())
        }
        ControlKind::CompositionPrefetchEnable => renderer
            .enable_composition_prefetch()
            .map(|()| renderer.composition_status())
            .map_err(stream_error),
        ControlKind::CompositionPrefetchDisable => renderer
            .disable_composition_prefetch()
            .map(|()| renderer.composition_status())
            .map_err(stream_error),
        ControlKind::CompositionOrder { terrain_first } => {
            renderer.set_composition_order(terrain_first);
            Ok(renderer.composition_status())
        }
        ControlKind::CompositionFixture { arbitrary_q8 } => renderer
            .set_composition_fixture(if arbitrary_q8 {
                CompositionFixture::ArbitraryQ8
            } else {
                CompositionFixture::CellCenter
            })
            .map(|()| renderer.composition_status())
            .map_err(stream_error),
        _ => unreachable!("non-composition command reached composition dispatcher"),
    }
}

fn stream_error(error: anyhow::Error) -> ProtocolError {
    let message = error.to_string();
    ProtocolError {
        code: if message.contains("busy") {
            "stream_busy"
        } else {
            "stream_failed"
        },
        message,
    }
}
