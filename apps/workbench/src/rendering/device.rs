use std::mem::ManuallyDrop;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::{
    DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE, IDXGIAdapter4, IDXGIFactory6,
};

const NVIDIA_VENDOR_ID: u32 = 0x10de;

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
