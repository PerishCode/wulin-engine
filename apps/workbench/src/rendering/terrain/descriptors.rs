use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_TYPELESS;

pub(super) unsafe fn create_heap(
    device: &ID3D12Device,
    regions: &[ID3D12Resource],
    stats: &ID3D12Resource,
    seams: &ID3D12Resource,
) -> Result<ID3D12DescriptorHeap> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: 52,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        })
    }
    .context("terrain descriptor heap creation failed")?;
    let increment =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV) }
            as usize;
    let start = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };
    for (index, resource) in regions.iter().enumerate() {
        let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: 0,
                    NumElements: terrain_format::PAYLOAD_BYTES / 4,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_RAW,
                },
            },
        };
        unsafe {
            device.CreateShaderResourceView(
                resource,
                Some(&desc),
                D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: start.ptr + index * increment,
                },
            )
        };
    }
    for (index, resource) in [stats, seams].into_iter().enumerate() {
        let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: 0,
                    NumElements: 8,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_RAW,
                },
            },
        };
        unsafe {
            device.CreateUnorderedAccessView(
                resource,
                None,
                Some(&desc),
                D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: start.ptr + (50 + index) * increment,
                },
            )
        };
    }
    Ok(heap)
}
