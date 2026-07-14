mod descriptors;
mod targets;
mod upload;

use anyhow::Result;
use meshlet_catalog::Catalog as MeshletCatalog;
use surface_catalog::Catalog;
use windows::Win32::Graphics::Direct3D12::*;

use crate::rendering::gpu_capture::Readback;

use self::descriptors::{HeapInputs, cpu_handle, create_heap};
use self::targets::{
    color_target, create_shadow_dsv, create_visibility_rtv, readback_buffer, shadow_target,
    uav_buffer, visibility_target, winner_target,
};
use self::upload::UploadedSurface;
use super::occlusion::OcclusionResources;

pub const CANDIDATE_CAPACITY: u32 = 25_600;
pub const STATS_BYTES: u64 = 32;
pub const SAMPLE_COUNT: u32 = 6;
pub const SAMPLE_STRIDE: u64 = 48;
pub const SAMPLE_BYTES: u64 = SAMPLE_COUNT as u64 * SAMPLE_STRIDE;

pub struct SurfaceResources {
    pub catalog: Catalog,
    pub catalog_sha256: String,
    pub _uploaded: UploadedSurface,
    pub visibility: ID3D12Resource,
    pub visibility_winner: ID3D12Resource,
    pub color: ID3D12Resource,
    pub candidate_to_visible: ID3D12Resource,
    pub stats: ID3D12Resource,
    pub samples: ID3D12Resource,
    pub shadow: ID3D12Resource,
    pub occlusion: OcclusionResources,
    pub heap: ID3D12DescriptorHeap,
    pub visibility_rtv: ID3D12DescriptorHeap,
    pub shadow_dsv: ID3D12DescriptorHeap,
    pub visibility_readback: Readback,
    pub winner_readback: Readback,
    pub stats_readback: ID3D12Resource,
    pub sample_readback: ID3D12Resource,
    pub shadow_readback: Readback,
    descriptor_increment: usize,
}

pub struct SurfaceResourceInput<'a> {
    pub queue: &'a ID3D12CommandQueue,
    pub source_heap: &'a ID3D12DescriptorHeap,
    pub source_visible: &'a ID3D12Resource,
    pub source_counters: &'a ID3D12Resource,
    pub mesh: &'a MeshletCatalog,
    pub extent: [u32; 2],
}

impl SurfaceResources {
    pub unsafe fn new(device: &ID3D12Device, input: SurfaceResourceInput<'_>) -> Result<Self> {
        let [width, height] = input.extent;
        let catalog = Catalog::build(input.mesh);
        let catalog_sha256 = catalog.sha256();
        let uploaded = unsafe { UploadedSurface::new(device, input.queue, &catalog) }?;
        let visibility = unsafe { visibility_target(device, width, height) }?;
        let visibility_winner = unsafe { winner_target(device, width, height) }?;
        let color = unsafe { color_target(device, width, height) }?;
        let candidate_to_visible = unsafe { uav_buffer(device, CANDIDATE_CAPACITY as u64 * 4) }?;
        let stats = unsafe { uav_buffer(device, STATS_BYTES) }?;
        let samples = unsafe { uav_buffer(device, SAMPLE_BYTES) }?;
        let shadow = unsafe { shadow_target(device, super::shadow::MAP_SIDE) }?;
        let occlusion = unsafe { OcclusionResources::new(device, width, height) }?;
        let (heap, descriptor_increment) = unsafe {
            create_heap(
                device,
                HeapInputs {
                    source_heap: input.source_heap,
                    catalog: &catalog,
                    uploaded: &uploaded,
                    visibility: &visibility,
                    winner: &visibility_winner,
                    color: &color,
                    candidate: &candidate_to_visible,
                    stats: &stats,
                    samples: &samples,
                    shadow: &shadow,
                    source_visible: input.source_visible,
                    source_counters: input.source_counters,
                    occlusion: &occlusion,
                },
            )
        }?;
        let visibility_rtv = unsafe { create_visibility_rtv(device, &visibility) }?;
        let shadow_dsv = unsafe { create_shadow_dsv(device, &shadow) }?;
        let visibility_readback =
            unsafe { Readback::new_with_pixel_bytes(device, &visibility, 8) }?;
        let winner_readback =
            unsafe { Readback::new_with_pixel_bytes(device, &visibility_winner, 8) }?;
        let stats_readback = unsafe { readback_buffer(device, STATS_BYTES) }?;
        let sample_readback = unsafe { readback_buffer(device, SAMPLE_BYTES) }?;
        let shadow_readback = unsafe { Readback::new_with_pixel_bytes(device, &shadow, 4) }?;
        Ok(Self {
            catalog,
            catalog_sha256,
            _uploaded: uploaded,
            visibility,
            visibility_winner,
            color,
            candidate_to_visible,
            stats,
            samples,
            shadow,
            occlusion,
            heap,
            visibility_rtv,
            shadow_dsv,
            visibility_readback,
            winner_readback,
            stats_readback,
            sample_readback,
            shadow_readback,
            descriptor_increment,
        })
    }

    pub unsafe fn visibility_handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.visibility_rtv.GetCPUDescriptorHandleForHeapStart() }
    }

    pub unsafe fn shadow_handle(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.shadow_dsv.GetCPUDescriptorHandleForHeapStart() }
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

    pub unsafe fn occlusion_counter_uav_handles(
        &self,
    ) -> (D3D12_GPU_DESCRIPTOR_HANDLE, D3D12_CPU_DESCRIPTOR_HANDLE) {
        unsafe { self.uav_handles(81) }
    }

    pub unsafe fn occlusion_mask_uav_handles(
        &self,
    ) -> (D3D12_GPU_DESCRIPTOR_HANDLE, D3D12_CPU_DESCRIPTOR_HANDLE) {
        unsafe { self.uav_handles(93) }
    }

    unsafe fn uav_handles(
        &self,
        index: usize,
    ) -> (D3D12_GPU_DESCRIPTOR_HANDLE, D3D12_CPU_DESCRIPTOR_HANDLE) {
        let gpu = unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() };
        let cpu = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        (
            D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: gpu.ptr + (self.descriptor_increment * index) as u64,
            },
            cpu_handle(cpu, self.descriptor_increment, index),
        )
    }
}
