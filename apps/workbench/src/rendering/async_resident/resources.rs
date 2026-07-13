use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN;

use crate::async_resident::ASYNC_CACHE_CAPACITY;
use crate::load::INSTANCES_PER_REGION;
use crate::resident::INSTANCE_RECORD_BYTES;

pub(super) unsafe fn create_descriptor_heap(
    device: &ID3D12Device,
    regions: &[ID3D12Resource],
) -> Result<ID3D12DescriptorHeap> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: ASYNC_CACHE_CAPACITY as u32,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        })
    }
    .context("async resident descriptor heap creation failed")?;
    let increment =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV) }
            as usize;
    let start = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };
    for (index, resource) in regions.iter().enumerate() {
        let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: 0,
                    NumElements: INSTANCES_PER_REGION,
                    StructureByteStride: INSTANCE_RECORD_BYTES as u32,
                    Flags: D3D12_BUFFER_SRV_FLAG_NONE,
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
    Ok(heap)
}
