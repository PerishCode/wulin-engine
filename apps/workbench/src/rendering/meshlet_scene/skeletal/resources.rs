use std::mem::size_of;
use std::ptr;

use animation_catalog::{Affine, BONE_COUNT, Bone, Catalog as AnimationCatalog, SkinBinding};
use anyhow::{Context, Result};
use meshlet_catalog::{Catalog as MeshletCatalog, Lod, Meshlet, Vertex};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R32_TYPELESS, DXGI_FORMAT_UNKNOWN};
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::async_resident::ASYNC_CACHE_CAPACITY;
use crate::load::INSTANCES_PER_REGION;
use crate::rendering::meshlet_scene::CatalogBuffers;
use crate::rendering::resident::create_buffer;
use crate::resident::ACTIVE_REGION_CAPACITY;

pub const COUNTER_BYTES: u64 = 80;
pub const SAMPLE_BYTES: u64 = 224;
pub const QUERY_COUNT: u32 = 5;
pub const MAX_SHARED_POSES: u32 = 512;
pub const MAX_SKELETAL_VISIBLE: u32 = ACTIVE_REGION_CAPACITY as u32 * INSTANCES_PER_REGION;
pub const PALETTE_BYTES: u64 = MAX_SKELETAL_VISIBLE as u64 * BONE_COUNT as u64 * 48;
const DESCRIPTOR_COUNT: u32 = 68;

pub struct AnimationBuffers {
    pub bones: ID3D12Resource,
    pub inverse_bind: ID3D12Resource,
    pub samples: ID3D12Resource,
    pub skin: ID3D12Resource,
    pub total_bytes: usize,
}

impl AnimationBuffers {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        catalog: &AnimationCatalog,
    ) -> Result<Self> {
        let payloads = [
            catalog.bone_bytes(),
            catalog.inverse_bind_bytes(),
            catalog.sample_bytes(),
            catalog.skin_binding_bytes(),
        ];
        let resources = unsafe { upload_payloads(device, queue, &payloads) }?;
        let mut resources = resources.into_iter();
        Ok(Self {
            bones: resources.next().unwrap(),
            inverse_bind: resources.next().unwrap(),
            samples: resources.next().unwrap(),
            skin: resources.next().unwrap(),
            total_bytes: payloads.iter().map(Vec::len).sum(),
        })
    }
}

pub struct ExecutionResources {
    pub visible: ID3D12Resource,
    pub counters: ID3D12Resource,
    pub animated_indices: ID3D12Resource,
    pub pose_bitset: ID3D12Resource,
    pub active_pose_keys: ID3D12Resource,
    pub palette: ID3D12Resource,
    pub sample: ID3D12Resource,
    pub heap: ID3D12DescriptorHeap,
    pub query_heap: ID3D12QueryHeap,
    pub timestamp_readback: ID3D12Resource,
    pub counter_readback: ID3D12Resource,
    pub sample_readback: ID3D12Resource,
    pub execution_bytes: u64,
}

impl ExecutionResources {
    pub unsafe fn new(
        device: &ID3D12Device,
        region_heap: &ID3D12DescriptorHeap,
        mesh_catalog: &MeshletCatalog,
        animation_catalog: &AnimationCatalog,
        mesh: &CatalogBuffers,
        animation: &AnimationBuffers,
    ) -> Result<Self> {
        let visible = unsafe { uav_buffer(device, MAX_SKELETAL_VISIBLE as u64 * 24) }?;
        let counters = unsafe { uav_buffer(device, COUNTER_BYTES) }?;
        let animated_indices = unsafe { uav_buffer(device, MAX_SKELETAL_VISIBLE as u64 * 4) }?;
        let pose_bitset = unsafe { uav_buffer(device, MAX_SHARED_POSES as u64 / 8) }?;
        let active_pose_keys = unsafe { uav_buffer(device, MAX_SHARED_POSES as u64 * 4) }?;
        let palette = unsafe { uav_buffer(device, PALETTE_BYTES) }?;
        let sample = unsafe { uav_buffer(device, SAMPLE_BYTES) }?;
        let heap = unsafe {
            create_heap(
                device,
                region_heap,
                mesh_catalog,
                animation_catalog,
                mesh,
                animation,
                [
                    &visible,
                    &counters,
                    &animated_indices,
                    &pose_bitset,
                    &active_pose_keys,
                    &palette,
                    &sample,
                ],
            )
        }?;
        let query_desc = D3D12_QUERY_HEAP_DESC {
            Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
            Count: QUERY_COUNT,
            NodeMask: 0,
        };
        let mut query_heap = None;
        unsafe { device.CreateQueryHeap(&query_desc, &mut query_heap) }
            .context("skeletal timestamp query heap creation failed")?;
        let query_heap = query_heap.context("skeletal timestamp query returned no heap")?;
        let timestamp_readback = unsafe { readback_buffer(device, QUERY_COUNT as u64 * 8) }?;
        let counter_readback = unsafe { readback_buffer(device, COUNTER_BYTES) }?;
        let sample_readback = unsafe { readback_buffer(device, SAMPLE_BYTES) }?;
        Ok(Self {
            visible,
            counters,
            animated_indices,
            pose_bitset,
            active_pose_keys,
            palette,
            sample,
            heap,
            query_heap,
            timestamp_readback,
            counter_readback,
            sample_readback,
            execution_bytes: MAX_SKELETAL_VISIBLE as u64 * (24 + 4)
                + COUNTER_BYTES
                + MAX_SHARED_POSES as u64 / 8
                + MAX_SHARED_POSES as u64 * 4
                + PALETTE_BYTES
                + SAMPLE_BYTES,
        })
    }
}

unsafe fn create_heap(
    device: &ID3D12Device,
    region_heap: &ID3D12DescriptorHeap,
    mesh_catalog: &MeshletCatalog,
    animation_catalog: &AnimationCatalog,
    mesh: &CatalogBuffers,
    animation: &AnimationBuffers,
    uavs: [&ID3D12Resource; 7],
) -> Result<ID3D12DescriptorHeap> {
    let heap: ID3D12DescriptorHeap = unsafe {
        device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: DESCRIPTOR_COUNT,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        })
    }
    .context("skeletal descriptor heap creation failed")?;
    let increment =
        unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV) }
            as usize;
    let start = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };
    unsafe {
        device.CopyDescriptorsSimple(
            ASYNC_CACHE_CAPACITY as u32,
            start,
            region_heap.GetCPUDescriptorHandleForHeapStart(),
            D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
        );
    }
    for (offset, resource, count, stride) in [
        (50, uavs[0], MAX_SKELETAL_VISIBLE, 24),
        (
            51,
            &mesh.vertices,
            mesh_catalog.vertices.len() as u32,
            size_of::<Vertex>() as u32,
        ),
        (
            52,
            &mesh.meshlets,
            mesh_catalog.meshlets.len() as u32,
            size_of::<Meshlet>() as u32,
        ),
        (
            53,
            &mesh.meshlet_vertices,
            mesh_catalog.meshlet_vertices.len() as u32,
            4,
        ),
        (
            54,
            &mesh.primitives,
            mesh_catalog.primitives.len() as u32,
            4,
        ),
        (
            55,
            &mesh.lods,
            mesh_catalog.lods.len() as u32,
            size_of::<Lod>() as u32,
        ),
        (
            56,
            &animation.bones,
            animation_catalog.bones.len() as u32,
            size_of::<Bone>() as u32,
        ),
        (
            57,
            &animation.inverse_bind,
            animation_catalog.inverse_bind.len() as u32,
            size_of::<Affine>() as u32,
        ),
        (
            58,
            &animation.samples,
            animation_catalog.samples.len() as u32,
            size_of::<Affine>() as u32,
        ),
        (
            59,
            &animation.skin,
            animation_catalog.skin_bindings.len() as u32,
            size_of::<SkinBinding>() as u32,
        ),
        (
            60,
            uavs[5],
            (PALETTE_BYTES / 48) as u32,
            size_of::<Affine>() as u32,
        ),
    ] {
        unsafe {
            structured_srv(
                device,
                resource,
                count,
                stride,
                cpu_handle(start, increment, offset),
            )
        };
    }
    unsafe {
        structured_uav(
            device,
            uavs[0],
            MAX_SKELETAL_VISIBLE,
            24,
            cpu_handle(start, increment, 61),
        );
        raw_uav(
            device,
            uavs[1],
            COUNTER_BYTES,
            cpu_handle(start, increment, 62),
        );
        structured_uav(
            device,
            uavs[2],
            MAX_SKELETAL_VISIBLE,
            4,
            cpu_handle(start, increment, 63),
        );
        raw_uav(
            device,
            uavs[3],
            MAX_SHARED_POSES as u64 / 8,
            cpu_handle(start, increment, 64),
        );
        structured_uav(
            device,
            uavs[4],
            MAX_SHARED_POSES,
            4,
            cpu_handle(start, increment, 65),
        );
        structured_uav(
            device,
            uavs[5],
            (PALETTE_BYTES / 48) as u32,
            48,
            cpu_handle(start, increment, 66),
        );
        raw_uav(
            device,
            uavs[6],
            SAMPLE_BYTES,
            cpu_handle(start, increment, 67),
        );
    }
    Ok(heap)
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

unsafe fn uav_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
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

unsafe fn readback_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
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

unsafe fn upload_payloads(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
    payloads: &[Vec<u8>],
) -> Result<Vec<ID3D12Resource>> {
    let mut defaults = Vec::with_capacity(payloads.len());
    let mut uploads = Vec::with_capacity(payloads.len());
    for bytes in payloads {
        defaults.push(unsafe {
            create_buffer(
                device,
                bytes.len() as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?);
        let upload = unsafe {
            create_buffer(
                device,
                bytes.len() as u64,
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let mut mapped = ptr::null_mut();
        unsafe {
            upload.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut mapped),
            )
        }
        .context("animation upload map failed")?;
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast(), bytes.len()) };
        unsafe { upload.Unmap(0, None) };
        uploads.push(upload);
    }
    let allocator: ID3D12CommandAllocator =
        unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
            .context("animation catalog allocator creation failed")?;
    let list: ID3D12GraphicsCommandList =
        unsafe { device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None) }
            .context("animation catalog command list creation failed")?;
    for ((default, upload), bytes) in defaults.iter().zip(&uploads).zip(payloads) {
        unsafe {
            list.CopyBufferRegion(default, 0, upload, 0, bytes.len() as u64);
            crate::rendering::device::transition(
                &list,
                default,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
        }
    }
    unsafe { list.Close() }.context("animation catalog command list close failed")?;
    let command: ID3D12CommandList = list.cast()?;
    unsafe { queue.ExecuteCommandLists(&[Some(command)]) };
    let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
        .context("animation catalog fence creation failed")?;
    let event = unsafe { CreateEventW(None, false, false, None) }
        .context("animation catalog event creation failed")?;
    unsafe { queue.Signal(&fence, 1) }.context("animation catalog signal failed")?;
    unsafe { fence.SetEventOnCompletion(1, event) }
        .context("animation catalog wait setup failed")?;
    let wait = unsafe { WaitForSingleObject(event, INFINITE) };
    unsafe { CloseHandle(event) }.context("animation catalog event close failed")?;
    anyhow::ensure!(
        wait == WAIT_OBJECT_0,
        "animation catalog wait returned {wait:?}"
    );
    Ok(defaults)
}
