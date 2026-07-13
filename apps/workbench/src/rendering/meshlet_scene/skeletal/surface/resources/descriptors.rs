use std::mem::size_of;

use anyhow::{Context, Result};
use surface_catalog::{MATERIAL_COUNT, MIP_COUNT, Material, SurfacePrimitive, SurfaceVertex};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32_TYPELESS, DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_R32G32_UINT, DXGI_FORMAT_UNKNOWN,
};

use super::super::occlusion::{OCCLUSION_COUNTER_BYTES, OCCLUSION_GROUPS, OcclusionResources};
use super::upload::UploadedSurface;
use super::{CANDIDATE_CAPACITY, SAMPLE_BYTES, STATS_BYTES};

const DESCRIPTOR_COUNT: u32 = 96;
const COPIED_DESCRIPTOR_COUNT: u32 = 61;

pub struct HeapInputs<'a> {
    pub source_heap: &'a ID3D12DescriptorHeap,
    pub catalog: &'a surface_catalog::Catalog,
    pub uploaded: &'a UploadedSurface,
    pub visibility: &'a ID3D12Resource,
    pub winner: &'a ID3D12Resource,
    pub color: &'a ID3D12Resource,
    pub candidate: &'a ID3D12Resource,
    pub stats: &'a ID3D12Resource,
    pub samples: &'a ID3D12Resource,
    pub source_visible: &'a ID3D12Resource,
    pub source_counters: &'a ID3D12Resource,
    pub occlusion: &'a OcclusionResources,
}

pub unsafe fn create_heap(
    device: &ID3D12Device,
    inputs: HeapInputs<'_>,
) -> Result<(ID3D12DescriptorHeap, usize)> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: DESCRIPTOR_COUNT,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        })
    }
    .context("surface descriptor heap creation failed")?;
    let increment =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV) }
            as usize;
    let start = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };
    unsafe {
        device.CopyDescriptorsSimple(
            COPIED_DESCRIPTOR_COUNT,
            start,
            inputs.source_heap.GetCPUDescriptorHandleForHeapStart(),
            D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
        );
        structured_srv(
            device,
            &inputs.occlusion.filtered_visible,
            CANDIDATE_CAPACITY,
            24,
            cpu_handle(start, increment, 50),
        );
        structured_srv(
            device,
            inputs.source_visible,
            CANDIDATE_CAPACITY,
            24,
            cpu_handle(start, increment, 61),
        );
        raw_srv(
            device,
            inputs.source_counters,
            80,
            cpu_handle(start, increment, 62),
        );
        structured_srv(
            device,
            &inputs.uploaded.vertices,
            inputs.catalog.vertices.len() as u32,
            size_of::<SurfaceVertex>() as u32,
            cpu_handle(start, increment, 68),
        );
        structured_srv(
            device,
            &inputs.uploaded.primitives,
            inputs.catalog.primitives.len() as u32,
            size_of::<SurfacePrimitive>() as u32,
            cpu_handle(start, increment, 69),
        );
        structured_srv(
            device,
            &inputs.uploaded.materials,
            inputs.catalog.materials.len() as u32,
            size_of::<Material>() as u32,
            cpu_handle(start, increment, 70),
        );
        texture_srv(
            device,
            inputs.visibility,
            DXGI_FORMAT_R32G32_UINT,
            D3D12_SRV_DIMENSION_TEXTURE2D,
            cpu_handle(start, increment, 71),
        );
        texture_srv(
            device,
            &inputs.uploaded.texture,
            DXGI_FORMAT_R8G8B8A8_UNORM,
            D3D12_SRV_DIMENSION_TEXTURE2DARRAY,
            cpu_handle(start, increment, 72),
        );
        structured_srv(
            device,
            inputs.candidate,
            CANDIDATE_CAPACITY,
            4,
            cpu_handle(start, increment, 73),
        );
        structured_uav(
            device,
            inputs.candidate,
            CANDIDATE_CAPACITY,
            4,
            cpu_handle(start, increment, 74),
        );
        texture_uav(
            device,
            inputs.color,
            DXGI_FORMAT_R8G8B8A8_UNORM,
            cpu_handle(start, increment, 75),
        );
        raw_uav(
            device,
            inputs.stats,
            STATS_BYTES,
            cpu_handle(start, increment, 76),
        );
        raw_uav(
            device,
            inputs.samples,
            SAMPLE_BYTES,
            cpu_handle(start, increment, 77),
        );
        texture_uav(
            device,
            inputs.winner,
            DXGI_FORMAT_R32G32_UINT,
            cpu_handle(start, increment, 78),
        );
        texture_srv(
            device,
            inputs.winner,
            DXGI_FORMAT_R32G32_UINT,
            D3D12_SRV_DIMENSION_TEXTURE2D,
            cpu_handle(start, increment, 79),
        );
        structured_uav(
            device,
            &inputs.occlusion.filtered_visible,
            CANDIDATE_CAPACITY,
            24,
            cpu_handle(start, increment, 80),
        );
        raw_uav(
            device,
            &inputs.occlusion.counters,
            OCCLUSION_COUNTER_BYTES,
            cpu_handle(start, increment, 81),
        );
        for mip in 0..inputs.occlusion.mip_count {
            texture_mip_uav(
                device,
                &inputs.occlusion.hierarchy,
                DXGI_FORMAT_R32_UINT,
                mip,
                cpu_handle(start, increment, 82 + mip as usize),
            );
        }
        structured_uav(
            device,
            &inputs.occlusion.candidate_mask,
            CANDIDATE_CAPACITY,
            4,
            cpu_handle(start, increment, 93),
        );
        structured_uav(
            device,
            &inputs.occlusion.group_offsets,
            OCCLUSION_GROUPS,
            4,
            cpu_handle(start, increment, 94),
        );
        texture_srv_mips(
            device,
            &inputs.occlusion.hierarchy,
            DXGI_FORMAT_R32_UINT,
            inputs.occlusion.mip_count,
            cpu_handle(start, increment, 95),
        );
    }
    Ok((heap, increment))
}

unsafe fn raw_srv(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    bytes: u64,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: DXGI_FORMAT_R32_TYPELESS,
        ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Buffer: D3D12_BUFFER_SRV {
                NumElements: (bytes / 4) as u32,
                Flags: D3D12_BUFFER_SRV_FLAG_RAW,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateShaderResourceView(resource, Some(&desc), handle) };
}

unsafe fn structured_srv(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    count: u32,
    stride: u32,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Buffer: D3D12_BUFFER_SRV {
                NumElements: count,
                StructureByteStride: stride,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateShaderResourceView(resource, Some(&desc), handle) };
}

unsafe fn structured_uav(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    count: u32,
    stride: u32,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: DXGI_FORMAT_UNKNOWN,
        ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Buffer: D3D12_BUFFER_UAV {
                NumElements: count,
                StructureByteStride: stride,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

unsafe fn texture_srv(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    format: DXGI_FORMAT,
    dimension: D3D12_SRV_DIMENSION,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let anonymous = if dimension == D3D12_SRV_DIMENSION_TEXTURE2DARRAY {
        D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2DArray: D3D12_TEX2D_ARRAY_SRV {
                MipLevels: MIP_COUNT,
                ArraySize: MATERIAL_COUNT,
                ..Default::default()
            },
        }
    } else {
        D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_SRV {
                MipLevels: 1,
                ..Default::default()
            },
        }
    };
    let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: format,
        ViewDimension: dimension,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: anonymous,
    };
    unsafe { device.CreateShaderResourceView(resource, Some(&desc), handle) };
}

unsafe fn texture_uav(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    format: DXGI_FORMAT,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
        ..Default::default()
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

unsafe fn texture_srv_mips(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    format: DXGI_FORMAT,
    mip_count: u32,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_SRV {
                MipLevels: mip_count,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateShaderResourceView(resource, Some(&desc), handle) };
}

unsafe fn texture_mip_uav(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    format: DXGI_FORMAT,
    mip: u32,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_UAV {
                MipSlice: mip,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

unsafe fn raw_uav(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    bytes: u64,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: DXGI_FORMAT_R32_TYPELESS,
        ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Buffer: D3D12_BUFFER_UAV {
                NumElements: (bytes / 4) as u32,
                Flags: D3D12_BUFFER_UAV_FLAG_RAW,
                ..Default::default()
            },
        },
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

pub fn cpu_handle(
    start: D3D12_CPU_DESCRIPTOR_HANDLE,
    increment: usize,
    index: usize,
) -> D3D12_CPU_DESCRIPTOR_HANDLE {
    D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: start.ptr + increment * index,
    }
}
