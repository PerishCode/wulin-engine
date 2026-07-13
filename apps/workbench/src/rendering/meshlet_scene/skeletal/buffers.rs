use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::*;

use crate::rendering::resident::create_buffer;

pub(super) unsafe fn uav_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
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

pub(super) unsafe fn readback_buffer(device: &ID3D12Device, bytes: u64) -> Result<ID3D12Resource> {
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
