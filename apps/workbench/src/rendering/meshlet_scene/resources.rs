use std::ptr;

use anyhow::{Context, Result};
use meshlet_catalog::Catalog;
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::rendering::resident::{create_buffer, transition};

pub struct CatalogBuffers {
    pub vertices: ID3D12Resource,
    pub meshlets: ID3D12Resource,
    pub meshlet_vertices: ID3D12Resource,
    pub primitives: ID3D12Resource,
    pub lods: ID3D12Resource,
    pub total_bytes: usize,
}

impl CatalogBuffers {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        catalog: &Catalog,
    ) -> Result<Self> {
        let payloads = [
            catalog.vertex_bytes(),
            catalog.meshlet_bytes(),
            catalog.meshlet_vertex_bytes(),
            catalog.primitive_bytes(),
            catalog.lod_bytes(),
        ];
        let mut defaults = Vec::with_capacity(payloads.len());
        let mut uploads = Vec::with_capacity(payloads.len());
        for bytes in &payloads {
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
            unsafe { write_upload(&upload, bytes) }?;
            uploads.push(upload);
        }

        let allocator: ID3D12CommandAllocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .context("meshlet catalog allocator creation failed")?;
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)
        }
        .context("meshlet catalog command list creation failed")?;
        for ((default, upload), bytes) in defaults.iter().zip(&uploads).zip(&payloads) {
            unsafe {
                command_list.CopyBufferRegion(default, 0, upload, 0, bytes.len() as u64);
                transition(
                    &command_list,
                    default,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                );
            }
        }
        unsafe { command_list.Close() }.context("meshlet catalog command list close failed")?;
        let list: ID3D12CommandList = command_list.cast()?;
        unsafe { queue.ExecuteCommandLists(&[Some(list)]) };

        let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("meshlet catalog fence creation failed")?;
        let event = unsafe { CreateEventW(None, false, false, None) }
            .context("meshlet catalog event creation failed")?;
        let result = (|| -> Result<()> {
            unsafe { queue.Signal(&fence, 1) }.context("meshlet catalog signal failed")?;
            unsafe { fence.SetEventOnCompletion(1, event) }
                .context("meshlet catalog fence wait setup failed")?;
            let wait = unsafe { WaitForSingleObject(event, INFINITE) };
            anyhow::ensure!(
                wait == WAIT_OBJECT_0,
                "meshlet catalog wait returned {wait:?}"
            );
            Ok(())
        })();
        unsafe { CloseHandle(event) }.context("meshlet catalog event close failed")?;
        result?;

        let mut resources = defaults.into_iter();
        Ok(Self {
            vertices: resources.next().unwrap(),
            meshlets: resources.next().unwrap(),
            meshlet_vertices: resources.next().unwrap(),
            primitives: resources.next().unwrap(),
            lods: resources.next().unwrap(),
            total_bytes: payloads.iter().map(Vec::len).sum(),
        })
    }
}

unsafe fn write_upload(resource: &ID3D12Resource, bytes: &[u8]) -> Result<()> {
    let mut mapped = ptr::null_mut();
    unsafe {
        resource.Map(
            0,
            Some(&D3D12_RANGE { Begin: 0, End: 0 }),
            Some(&mut mapped),
        )
    }
    .context("meshlet catalog upload map failed")?;
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast(), bytes.len()) };
    unsafe {
        resource.Unmap(
            0,
            Some(&D3D12_RANGE {
                Begin: 0,
                End: bytes.len(),
            }),
        )
    };
    Ok(())
}
