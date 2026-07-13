use crate::load::LoadConfig;
use crate::rendering::Renderer;

use super::protocol::{ControlKind, ControlResult, ProtocolError};

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
        ControlKind::CompositionOrder { terrain_first } => {
            renderer.set_composition_order(terrain_first);
            Ok(renderer.composition_status())
        }
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
