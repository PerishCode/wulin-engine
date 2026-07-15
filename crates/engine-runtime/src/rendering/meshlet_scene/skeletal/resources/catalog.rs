use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::rendering::resident::create_buffer;

pub(super) unsafe fn upload_payloads(
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
