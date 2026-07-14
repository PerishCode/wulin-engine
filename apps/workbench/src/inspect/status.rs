use serde_json::json;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, IsWindowVisible,
};

use crate::rendering::Renderer;
use crate::{WorkbenchState, scene};

use super::protocol::{ControlResult, ProtocolError};

pub(crate) fn workload(renderer: &Renderer) -> serde_json::Value {
    json!({
        "mode": if renderer.composition_enabled() {
            "canonical-runtime"
        } else {
            "idle-shell"
        },
        "composition": renderer.composition_status(),
    })
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
        "schemaVersion": 2,
        "processId": std::process::id(),
        "launchedBySidecar": state.launched_by_sidecar,
        "uptimeMs": state.started_at.elapsed().as_millis(),
        "state": if state.paused { "paused" } else { "running" },
        "frameIndex": state.frame_index,
        "lastFrameMs": state.last_frame_ms,
        "clearColor": state.clear_color,
        "spatial": scene.spatial_json(),
        "workload": workload(renderer),
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
