use std::mem::size_of;

use anyhow::{Context, Result};
use meshlet_catalog::Catalog as MeshletCatalog;
use surface_catalog::{
    Catalog, MATERIAL_COUNT, MIP_COUNT, Material, SurfacePrimitive, SurfaceVertex,
};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32_TYPELESS, DXGI_FORMAT_R32G32_UINT,
    DXGI_FORMAT_UNKNOWN,
};

use crate::rendering::gpu_capture::Readback;

use super::targets::{
    color_target, create_visibility_rtv, readback_buffer, uav_buffer, visibility_target,
    winner_target,
};
use super::upload::UploadedSurface;

pub const CANDIDATE_CAPACITY: u32 = 25_600;
pub const STATS_BYTES: u64 = 32;
pub const SAMPLE_COUNT: u32 = 6;
pub const SAMPLE_STRIDE: u64 = 32;
pub const SAMPLE_BYTES: u64 = SAMPLE_COUNT as u64 * SAMPLE_STRIDE;
const DESCRIPTOR_COUNT: u32 = 79;
const COPIED_DESCRIPTOR_COUNT: u32 = 61;

pub struct SurfaceResources {
    pub catalog: Catalog,
    pub catalog_sha256: String,
    pub uploaded: UploadedSurface,
    pub visibility: ID3D12Resource,
    pub visibility_winner: ID3D12Resource,
    pub color: ID3D12Resource,
    pub candidate_to_visible: ID3D12Resource,
    pub stats: ID3D12Resource,
    pub samples: ID3D12Resource,
    pub heap: ID3D12DescriptorHeap,
    pub visibility_rtv: ID3D12DescriptorHeap,
    pub visibility_readback: Readback,
    pub stats_readback: ID3D12Resource,
    pub sample_readback: ID3D12Resource,
    pub execution_bytes: u64,
    descriptor_increment: usize,
}

impl SurfaceResources {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        source_heap: &ID3D12DescriptorHeap,
        mesh: &MeshletCatalog,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let catalog = Catalog::build(mesh);
        let catalog_sha256 = catalog.sha256();
        let uploaded = unsafe { UploadedSurface::new(device, queue, &catalog) }?;
        let visibility = unsafe { visibility_target(device, width, height) }?;
        let visibility_winner = unsafe { winner_target(device, width, height) }?;
        let color = unsafe { color_target(device, width, height) }?;
        let candidate_to_visible = unsafe { uav_buffer(device, CANDIDATE_CAPACITY as u64 * 4) }?;
        let stats = unsafe { uav_buffer(device, STATS_BYTES) }?;
        let samples = unsafe { uav_buffer(device, SAMPLE_BYTES) }?;
        let (heap, descriptor_increment) = unsafe {
            create_heap(
                device,
                HeapInputs {
                    source_heap,
                    catalog: &catalog,
                    uploaded: &uploaded,
                    visibility: &visibility,
                    winner: &visibility_winner,
                    color: &color,
                    candidate: &candidate_to_visible,
                    stats: &stats,
                    samples: &samples,
                },
            )
        }?;
        let visibility_rtv = unsafe { create_visibility_rtv(device, &visibility) }?;
        let visibility_readback =
            unsafe { Readback::new_with_pixel_bytes(device, &visibility, 8) }?;
        let stats_readback = unsafe { readback_buffer(device, STATS_BYTES) }?;
        let sample_readback = unsafe { readback_buffer(device, SAMPLE_BYTES) }?;
        let execution_bytes = u64::from(width) * u64::from(height) * 20
            + CANDIDATE_CAPACITY as u64 * 4
            + STATS_BYTES
            + SAMPLE_BYTES
            + uploaded.total_bytes as u64;
        Ok(Self {
            catalog,
            catalog_sha256,
            uploaded,
            visibility,
            visibility_winner,
            color,
            candidate_to_visible,
            stats,
            samples,
            heap,
            visibility_rtv,
            visibility_readback,
            stats_readback,
            sample_readback,
            execution_bytes,
            descriptor_increment,
        })
    }

    pub unsafe fn visibility_handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.visibility_rtv.GetCPUDescriptorHandleForHeapStart() }
    }

    pub unsafe fn gpu_start(&self) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() }
    }

    pub unsafe fn stats_uav_handles(
        &self,
    ) -> (D3D12_GPU_DESCRIPTOR_HANDLE, D3D12_CPU_DESCRIPTOR_HANDLE) {
        let gpu = unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() };
        let cpu = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        (
            D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: gpu.ptr + (self.descriptor_increment * 76) as u64,
            },
            cpu_handle(cpu, self.descriptor_increment, 76),
        )
    }

    pub unsafe fn winner_uav_handles(
        &self,
    ) -> (D3D12_GPU_DESCRIPTOR_HANDLE, D3D12_CPU_DESCRIPTOR_HANDLE) {
        let gpu = unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() };
        let cpu = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        (
            D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: gpu.ptr + (self.descriptor_increment * 78) as u64,
            },
            cpu_handle(cpu, self.descriptor_increment, 78),
        )
    }
}

struct HeapInputs<'a> {
    source_heap: &'a ID3D12DescriptorHeap,
    catalog: &'a Catalog,
    uploaded: &'a UploadedSurface,
    visibility: &'a ID3D12Resource,
    winner: &'a ID3D12Resource,
    color: &'a ID3D12Resource,
    candidate: &'a ID3D12Resource,
    stats: &'a ID3D12Resource,
    samples: &'a ID3D12Resource,
}

unsafe fn create_heap(
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
    }
    Ok((heap, increment))
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
                FirstElement: 0,
                NumElements: count,
                StructureByteStride: stride,
                Flags: D3D12_BUFFER_SRV_FLAG_NONE,
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
                FirstElement: 0,
                NumElements: count,
                StructureByteStride: stride,
                CounterOffsetInBytes: 0,
                Flags: D3D12_BUFFER_UAV_FLAG_NONE,
            },
        },
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

unsafe fn texture_srv(
    device: &ID3D12Device,
    resource: &ID3D12Resource,
    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
    dimension: D3D12_SRV_DIMENSION,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let anonymous = if dimension == D3D12_SRV_DIMENSION_TEXTURE2DARRAY {
        D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2DArray: D3D12_TEX2D_ARRAY_SRV {
                MostDetailedMip: 0,
                MipLevels: MIP_COUNT,
                FirstArraySlice: 0,
                ArraySize: MATERIAL_COUNT,
                PlaneSlice: 0,
                ResourceMinLODClamp: 0.0,
            },
        }
    } else {
        D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_SRV {
                MostDetailedMip: 0,
                MipLevels: 1,
                PlaneSlice: 0,
                ResourceMinLODClamp: 0.0,
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
    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
) {
    let desc = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_UAV {
                MipSlice: 0,
                PlaneSlice: 0,
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
                FirstElement: 0,
                NumElements: (bytes / 4) as u32,
                StructureByteStride: 0,
                CounterOffsetInBytes: 0,
                Flags: D3D12_BUFFER_UAV_FLAG_RAW,
            },
        },
    };
    unsafe { device.CreateUnorderedAccessView(resource, None, Some(&desc), handle) };
}

fn cpu_handle(
    start: D3D12_CPU_DESCRIPTOR_HANDLE,
    increment: usize,
    index: usize,
) -> D3D12_CPU_DESCRIPTOR_HANDLE {
    D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: start.ptr + increment * index,
    }
}
