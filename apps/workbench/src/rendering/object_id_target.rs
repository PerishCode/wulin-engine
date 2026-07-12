use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R32_UINT, DXGI_SAMPLE_DESC};

pub struct ObjectIdTarget {
    resource: ID3D12Resource,
    rtv_heap: ID3D12DescriptorHeap,
}

impl ObjectIdTarget {
    pub unsafe fn new(device: &ID3D12Device, width: u32, height: u32) -> Result<Self> {
        let resource = unsafe { create_resource(device, width, height) }?;
        let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 1,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc) }
            .context("object-ID descriptor heap creation failed")?;
        unsafe {
            device.CreateRenderTargetView(
                &resource,
                None,
                rtv_heap.GetCPUDescriptorHandleForHeapStart(),
            );
        }
        Ok(Self { resource, rtv_heap })
    }

    pub fn resource(&self) -> &ID3D12Resource {
        &self.resource
    }

    pub unsafe fn handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.rtv_heap.GetCPUDescriptorHandleForHeapStart() }
    }
}

unsafe fn create_resource(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<ID3D12Resource> {
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
    .context("object-ID render target allocation failed")?;
    resource.context("object-ID render target allocation returned no resource")
}
