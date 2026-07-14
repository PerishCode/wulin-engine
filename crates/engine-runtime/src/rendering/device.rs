use std::mem::ManuallyDrop;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32G32_UINT,
};
use windows::Win32::Graphics::Dxgi::{
    DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, IDXGIAdapter4, IDXGIFactory6,
};

const NVIDIA_VENDOR_ID: u32 = 0x10de;

#[derive(Clone, Copy)]
pub(super) struct DeviceCapabilities {
    pub mesh_shader_tier: u32,
    pub shader_model: &'static str,
    pub barycentrics: bool,
    pub rasterizer_ordered_views: bool,
    pub visibility_format: bool,
    pub color_uav_format: bool,
}

pub(super) unsafe fn enable_debug_layer() -> Result<()> {
    let mut debug = None;
    unsafe { D3D12GetDebugInterface(&mut debug) }.context("D3D12 debug layer is unavailable")?;
    let debug: ID3D12Debug = debug.context("D3D12 debug interface was empty")?;
    unsafe { debug.EnableDebugLayer() };
    Ok(())
}

pub(super) unsafe fn select_reference_adapter(
    factory: &IDXGIFactory6,
) -> Result<(IDXGIAdapter4, String)> {
    for index in 0..16 {
        let adapter = unsafe {
            factory.EnumAdapterByGpuPreference::<IDXGIAdapter4>(
                index,
                DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
            )
        };
        let Ok(adapter) = adapter else {
            break;
        };
        let desc = unsafe { adapter.GetDesc3() }?;
        if desc.VendorId == NVIDIA_VENDOR_ID {
            let end = desc
                .Description
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(desc.Description.len());
            return Ok((adapter, String::from_utf16_lossy(&desc.Description[..end])));
        }
    }
    bail!("no NVIDIA adapter was found on the reference platform")
}

pub(super) unsafe fn query_required_capabilities(
    device: &ID3D12Device,
) -> Result<DeviceCapabilities> {
    let mut options = D3D12_FEATURE_DATA_D3D12_OPTIONS7::default();
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_D3D12_OPTIONS7,
            (&raw mut options).cast(),
            size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS7>() as u32,
        )
    }
    .context("D3D12 options7 query failed")?;

    let mut shader_model = D3D12_FEATURE_DATA_SHADER_MODEL {
        HighestShaderModel: D3D_HIGHEST_SHADER_MODEL,
    };
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_SHADER_MODEL,
            (&raw mut shader_model).cast(),
            size_of::<D3D12_FEATURE_DATA_SHADER_MODEL>() as u32,
        )
    }
    .context("D3D12 shader-model query failed")?;

    let mut options3 = D3D12_FEATURE_DATA_D3D12_OPTIONS3::default();
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_D3D12_OPTIONS3,
            (&raw mut options3).cast(),
            size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS3>() as u32,
        )
    }
    .context("D3D12 options3 query failed")?;
    let mut base_options = D3D12_FEATURE_DATA_D3D12_OPTIONS::default();
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_D3D12_OPTIONS,
            (&raw mut base_options).cast(),
            size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS>() as u32,
        )
    }
    .context("D3D12 base options query failed")?;
    let visibility_format = unsafe { query_format(device, DXGI_FORMAT_R32G32_UINT) }?;
    let color_format = unsafe { query_format(device, DXGI_FORMAT_R8G8B8A8_UNORM) }?;
    let visibility_supported = visibility_format
        .Support1
        .contains(D3D12_FORMAT_SUPPORT1_RENDER_TARGET | D3D12_FORMAT_SUPPORT1_SHADER_LOAD);
    let visibility_uav_supported = visibility_format
        .Support2
        .contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_LOAD | D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE);
    let color_uav_supported = color_format
        .Support2
        .contains(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE);

    if options.MeshShaderTier.0 < D3D12_MESH_SHADER_TIER_1.0 {
        bail!(
            "reference adapter does not support mesh shaders (tier {})",
            options.MeshShaderTier.0
        );
    }
    if shader_model.HighestShaderModel.0 < D3D_SHADER_MODEL_6_6.0 {
        bail!(
            "reference adapter shader model {} is below 6.6",
            shader_model_name(shader_model.HighestShaderModel)
        );
    }
    if !options3.BarycentricsSupported.as_bool() {
        bail!("reference adapter does not support barycentrics");
    }
    if !visibility_supported {
        bail!("reference adapter does not support the R32G32_UINT visibility target");
    }
    if !base_options.ROVsSupported.as_bool() || !visibility_uav_supported {
        bail!("reference adapter does not support the deterministic ROV winner target");
    }
    if !color_uav_supported {
        bail!("reference adapter does not support R8G8B8A8_UNORM typed UAV stores");
    }
    Ok(DeviceCapabilities {
        mesh_shader_tier: (options.MeshShaderTier.0 / 10) as u32,
        shader_model: shader_model_name(shader_model.HighestShaderModel),
        barycentrics: true,
        rasterizer_ordered_views: true,
        visibility_format: true,
        color_uav_format: true,
    })
}

unsafe fn query_format(
    device: &ID3D12Device,
    format: DXGI_FORMAT,
) -> Result<D3D12_FEATURE_DATA_FORMAT_SUPPORT> {
    let mut support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: format,
        ..Default::default()
    };
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_FORMAT_SUPPORT,
            (&raw mut support).cast(),
            size_of::<D3D12_FEATURE_DATA_FORMAT_SUPPORT>() as u32,
        )
    }
    .context("D3D12 format support query failed")?;
    Ok(support)
}

fn shader_model_name(model: D3D_SHADER_MODEL) -> &'static str {
    match model {
        D3D_SHADER_MODEL_6_9 => "6.9",
        D3D_SHADER_MODEL_6_8 => "6.8",
        D3D_SHADER_MODEL_6_7 => "6.7",
        D3D_SHADER_MODEL_6_6 => "6.6",
        D3D_SHADER_MODEL_6_5 => "6.5",
        D3D_SHADER_MODEL_6_4 => "6.4",
        D3D_SHADER_MODEL_6_3 => "6.3",
        D3D_SHADER_MODEL_6_2 => "6.2",
        D3D_SHADER_MODEL_6_1 => "6.1",
        D3D_SHADER_MODEL_6_0 => "6.0",
        _ => "unknown",
    }
}

pub(super) unsafe fn transition(
    command_list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    };
    unsafe { command_list.ResourceBarrier(std::slice::from_ref(&barrier)) };
    unsafe {
        let transition = &mut *barrier.Anonymous.Transition;
        ManuallyDrop::drop(&mut transition.pResource);
    }
}
