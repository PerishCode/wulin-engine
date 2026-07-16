use engine_runtime::Runtime;
use serde_json::json;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, IsWindowVisible,
};

use crate::WorkbenchState;

use super::protocol::{ControlResult, ProtocolError};

pub(crate) fn workload(runtime: &Runtime) -> serde_json::Value {
    json!({
        "mode": if runtime.composition_enabled() {
            "canonical-runtime"
        } else {
            "idle-shell"
        },
        "composition": runtime.composition_status(),
    })
}

pub(super) fn workbench(hwnd: HWND, runtime: &Runtime, state: &WorkbenchState) -> ControlResult {
    let mut client = RECT::default();
    unsafe { GetClientRect(hwnd, &mut client) }.map_err(internal_error)?;
    let device_removed_reason = unsafe { runtime.device_removed_reason() };
    Ok(json!({
        "schemaVersion": 2,
        "processId": std::process::id(),
        "launchedBySidecar": state.launched_by_sidecar,
        "uptimeMs": state.started_at.elapsed().as_millis(),
        "state": if state.paused { "paused" } else { "running" },
        "frameIndex": state.frame_index,
        "lastFrameMs": state.last_frame_ms,
        "clearColor": state.clear_color,
        "objectTargetFeedback": state.object_target_feedback,
        "objectSuppression": state.object_suppression,
        "startup": state.startup,
        "spatial": runtime.spatial_json(),
        "workload": workload(runtime),
        "window": {
            "handle": format!("0x{:X}", hwnd.0 as usize),
            "width": client.right - client.left,
            "height": client.bottom - client.top,
            "visible": unsafe { IsWindowVisible(hwnd) }.as_bool(),
            "foreground": unsafe { GetForegroundWindow() } == hwnd
        },
        "renderer": {
            "api": "D3D12",
            "adapter": runtime.adapter_name(),
            "featureLevel": "12_1",
            "meshShaderTier": runtime.mesh_shader_tier(),
            "shaderModel": runtime.shader_model(),
            "barycentrics": runtime.barycentrics_supported(),
            "rasterizerOrderedViews": runtime.rasterizer_ordered_views_supported(),
            "visibilityFormat": runtime.visibility_format_supported(),
            "colorUavFormat": runtime.color_uav_format_supported(),
            "swapChainBuffers": 2,
            "format": "R8G8B8A8_UNORM",
            "vsync": true,
            "debugLayer": runtime.debug_layer(),
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
