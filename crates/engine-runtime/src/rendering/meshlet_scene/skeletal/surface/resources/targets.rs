use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32_TYPELESS,
    DXGI_FORMAT_R32G32_UINT, DXGI_SAMPLE_DESC,
};

use crate::rendering::resident::create_buffer;

pub unsafe fn visibility_target(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<ID3D12Resource> {
    unsafe {
        texture_resource(
            device,
            width,
            height,
            DXGI_FORMAT_R32G32_UINT,
            D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
            Some(D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_R32G32_UINT,
                Anonymous: D3D12_CLEAR_VALUE_0 { Color: [0.0; 4] },
            }),
        )
    }
}

pub unsafe fn color_target(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<ID3D12Resource> {
    unsafe {
        texture_resource(
            device,
            width,
            height,
            DXGI_FORMAT_R8G8B8A8_UNORM,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            None,
        )
    }
}

pub unsafe fn winner_target(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<ID3D12Resource> {
    unsafe {
        texture_resource(
            device,
            width,
            height,
            DXGI_FORMAT_R32G32_UINT,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            None,
        )
    }
}

pub unsafe fn shadow_target(device: &ID3D12Device, side: u32) -> Result<ID3D12Resource> {
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Width: u64::from(side),
        Height: side,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_R32_TYPELESS,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
        ..Default::default()
    };
    let clear = D3D12_CLEAR_VALUE {
        Format: DXGI_FORMAT_D32_FLOAT,
        Anonymous: D3D12_CLEAR_VALUE_0 {
            DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                Depth: 1.0,
                Stencil: 0,
            },
        },
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            },
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&clear),
            &mut resource,
        )
    }
    .context("surface shadow allocation failed")?;
    resource.context("surface shadow allocation returned no resource")
}

unsafe fn texture_resource(
    device: &ID3D12Device,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    flags: D3D12_RESOURCE_FLAGS,
    state: D3D12_RESOURCE_STATES,
    clear: Option<D3D12_CLEAR_VALUE>,
) -> Result<ID3D12Resource> {
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(width),
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: flags,
    };
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            state,
            clear.as_ref().map(std::ptr::from_ref),
            &mut resource,
        )
    }
    .context("surface texture allocation failed")?;
    resource.context("surface texture allocation returned no resource")
}

pub unsafe fn uav_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
    unsafe {
        create_buffer(
            device,
            bytes,
            D3D12_HEAP_TYPE_DEFAULT,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        )
    }
}

pub unsafe fn readback_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
    unsafe {
        create_buffer(
            device,
            bytes,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_FLAG_NONE,
        )
    }
}

pub unsafe fn create_visibility_rtv(
    device: &ID3D12Device,
    visibility: &ID3D12Resource,
) -> Result<ID3D12DescriptorHeap> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 1,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        })
    }
    .context("surface visibility RTV heap creation failed")?;
    unsafe {
        device.CreateRenderTargetView(visibility, None, heap.GetCPUDescriptorHandleForHeapStart())
    };
    Ok(heap)
}

pub unsafe fn create_shadow_dsv(
    device: &ID3D12Device,
    shadow: &ID3D12Resource,
) -> Result<ID3D12DescriptorHeap> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            NumDescriptors: 1,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        })
    }
    .context("surface shadow DSV heap creation failed")?;
    unsafe {
        device.CreateDepthStencilView(
            shadow,
            Some(&D3D12_DEPTH_STENCIL_VIEW_DESC {
                Format: DXGI_FORMAT_D32_FLOAT,
                ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
                ..Default::default()
            }),
            heap.GetCPUDescriptorHandleForHeapStart(),
        )
    };
    Ok(heap)
}
