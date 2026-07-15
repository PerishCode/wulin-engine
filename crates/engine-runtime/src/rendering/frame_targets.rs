use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R32_UINT, DXGI_SAMPLE_DESC,
};

pub struct FrameTargets {
    _depth: ID3D12Resource,
    depth_heap: ID3D12DescriptorHeap,
    semantic_ids: ID3D12Resource,
    semantic_heap: ID3D12DescriptorHeap,
}

impl FrameTargets {
    pub unsafe fn new(device: &ID3D12Device, width: u32, height: u32) -> Result<Self> {
        let (_depth, depth_heap) = unsafe { create_depth(device, width, height) }?;
        let (semantic_ids, semantic_heap) = unsafe { create_semantic_ids(device, width, height) }?;
        Ok(Self {
            _depth,
            depth_heap,
            semantic_ids,
            semantic_heap,
        })
    }

    pub unsafe fn clear_idle(&self, command_list: &ID3D12GraphicsCommandList) {
        let semantic = unsafe { self.semantic_handle() };
        let depth = unsafe { self.depth_handle() };
        unsafe {
            command_list.OMSetRenderTargets(1, Some(&semantic), true, Some(&depth));
            command_list.ClearRenderTargetView(semantic, &[0.0; 4], None);
            command_list.ClearDepthStencilView(depth, D3D12_CLEAR_FLAG_DEPTH, 0.0, 0, None);
        }
    }

    pub fn semantic_resource(&self) -> &ID3D12Resource {
        &self.semantic_ids
    }

    pub unsafe fn semantic_handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.semantic_heap.GetCPUDescriptorHandleForHeapStart() }
    }

    pub unsafe fn depth_handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.depth_heap.GetCPUDescriptorHandleForHeapStart() }
    }
}

unsafe fn create_semantic_ids(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<(ID3D12Resource, ID3D12DescriptorHeap)> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(width),
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_R32_UINT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
    };
    let clear = D3D12_CLEAR_VALUE {
        Format: DXGI_FORMAT_R32_UINT,
        Anonymous: D3D12_CLEAR_VALUE_0 { Color: [0.0; 4] },
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
            Some(&clear),
            &mut resource,
        )
    }
    .context("semantic-ID render target allocation failed")?;
    let resource = resource.context("semantic-ID render target allocation returned no resource")?;
    let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
        Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        NumDescriptors: 1,
        Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        NodeMask: 0,
    };
    let descriptor_heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc) }
        .context("semantic-ID descriptor heap creation failed")?;
    unsafe {
        device.CreateRenderTargetView(
            &resource,
            None,
            descriptor_heap.GetCPUDescriptorHandleForHeapStart(),
        );
    }
    Ok((resource, descriptor_heap))
}

unsafe fn create_depth(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<(ID3D12Resource, ID3D12DescriptorHeap)> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(width),
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_D32_FLOAT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
    };
    let clear = D3D12_CLEAR_VALUE {
        Format: DXGI_FORMAT_D32_FLOAT,
        Anonymous: D3D12_CLEAR_VALUE_0 {
            DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                Depth: 0.0,
                Stencil: 0,
            },
        },
    };
    let mut depth = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&clear),
            &mut depth,
        )
    }
    .context("depth allocation failed")?;
    let depth = depth.context("depth allocation returned no resource")?;
    let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
        Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        NumDescriptors: 1,
        Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        NodeMask: 0,
    };
    let descriptor_heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc) }
        .context("depth descriptor heap creation failed")?;
    unsafe {
        device.CreateDepthStencilView(
            &depth,
            None,
            descriptor_heap.GetCPUDescriptorHandleForHeapStart(),
        );
    }
    Ok((depth, descriptor_heap))
}
