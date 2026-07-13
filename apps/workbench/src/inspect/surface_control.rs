use crate::rendering::Renderer;

use super::{ControlResult, ProtocolError};

pub fn status(renderer: &Renderer) -> ControlResult {
    Ok(renderer.surface_status())
}

pub fn configure(renderer: &mut Renderer, material_count: u32, mip_level: u32) -> ControlResult {
    renderer
        .configure_surface(material_count, mip_level)
        .map(|()| renderer.surface_status())
        .map_err(|error| ProtocolError {
            code: "invalid_surface_config",
            message: error.to_string(),
        })
}

pub fn enable(renderer: &mut Renderer) -> ControlResult {
    renderer
        .enable_surface()
        .map(|()| renderer.surface_status())
        .map_err(|error| ProtocolError {
            code: "surface_unavailable",
            message: error.to_string(),
        })
}

pub fn disable(renderer: &mut Renderer) -> ControlResult {
    renderer.disable_surface();
    Ok(renderer.surface_status())
}

pub fn enable_occlusion(renderer: &mut Renderer) -> ControlResult {
    renderer.enable_surface_occlusion();
    Ok(renderer.surface_status())
}

pub fn disable_occlusion(renderer: &mut Renderer) -> ControlResult {
    renderer.disable_surface_occlusion();
    Ok(renderer.surface_status())
}

pub fn reset_occlusion(renderer: &mut Renderer) -> ControlResult {
    renderer.reset_surface_occlusion();
    Ok(renderer.surface_status())
}
