use serde_json::json;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, IsWindowVisible,
};

use crate::rendering::Renderer;
use crate::{WorkbenchState, scene};

use super::protocol::{ControlResult, ProtocolError};

pub(crate) fn load_status(renderer: &Renderer) -> serde_json::Value {
    if renderer.terrain_mode_enabled() {
        json!({
            "mode": "gpu-streamed-terrain",
            "load": renderer.terrain_config().map(|config| config.json()),
            "terrain": renderer.terrain_status(),
        })
    } else if renderer.async_resident_enabled() {
        json!({
            "mode": "async-resident-load",
            "load": renderer.async_resident_config().map(|config| config.json()),
            "async": renderer.async_resident_status(),
            "cooked": renderer.cooked_status(),
            "meshlet": renderer.meshlet_scene_status(),
            "skeletal": renderer.skeletal_scene_status(),
            "surface": renderer.surface_status(),
        })
    } else if let Some(config) = renderer.resident_config() {
        json!({"mode": "resident-load", "load": config.json()})
    } else if let Some(config) = renderer.load_config() {
        json!({"mode": "region-load", "load": config.json()})
    } else {
        json!({"mode": "calibration", "load": null})
    }
}

pub(super) fn workbench(
    hwnd: HWND,
    renderer: &Renderer,
    state: &WorkbenchState,
    scene: &scene::SceneState,
) -> ControlResult {
    let mut client = RECT::default();
    unsafe { GetClientRect(hwnd, &mut client) }.map_err(internal_error)?;
    let device_removed_reason = unsafe { renderer.device_removed_reason() };
    Ok(json!({
        "schemaVersion": 1,
        "processId": std::process::id(),
        "launchedBySidecar": state.launched_by_sidecar,
        "uptimeMs": state.started_at.elapsed().as_millis(),
        "state": if state.paused { "paused" } else { "running" },
        "frameIndex": state.frame_index,
        "lastFrameMs": state.last_frame_ms,
        "clearColor": state.clear_color,
        "spatial": scene.spatial_json(),
        "workload": load_status(renderer),
        "window": {
            "handle": format!("0x{:X}", hwnd.0 as usize),
            "width": client.right - client.left,
            "height": client.bottom - client.top,
            "visible": unsafe { IsWindowVisible(hwnd) }.as_bool(),
            "foreground": unsafe { GetForegroundWindow() } == hwnd
        },
        "renderer": {
            "api": "D3D12",
            "adapter": renderer.adapter_name(),
            "featureLevel": "12_1",
            "meshShaderTier": renderer.mesh_shader_tier(),
            "shaderModel": renderer.shader_model(),
            "barycentrics": renderer.barycentrics_supported(),
            "rasterizerOrderedViews": renderer.rasterizer_ordered_views_supported(),
            "visibilityFormat": renderer.visibility_format_supported(),
            "colorUavFormat": renderer.color_uav_format_supported(),
            "swapChainBuffers": 2,
            "format": "R8G8B8A8_UNORM",
            "vsync": true,
            "debugLayer": renderer.debug_layer(),
            "deviceRemovedReason": device_removed_reason
        },
        "lastError": state.last_error
    }))
}

fn internal_error(error: windows::core::Error) -> ProtocolError {
    ProtocolError {
        code: "internal_error",
        message: error.to_string(),
    }
}
