use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{Context, Result};
use surface_catalog::{Catalog, MATERIAL_COUNT, MIP_COUNT, TEXTURE_SIDE};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::rendering::resident::create_buffer;

pub struct UploadedSurface {
    pub vertices: ID3D12Resource,
    pub primitives: ID3D12Resource,
    pub materials: ID3D12Resource,
    pub texture: ID3D12Resource,
    pub total_bytes: usize,
}

impl UploadedSurface {
    pub unsafe fn new(
        device: &ID3D12Device,
        queue: &ID3D12CommandQueue,
        catalog: &Catalog,
    ) -> Result<Self> {
        let payloads = [
            catalog.vertex_bytes(),
            catalog.primitive_bytes(),
            catalog.material_bytes(),
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
        let texture = unsafe { create_texture(device) }?;
        let (texture_upload, layouts) =
            unsafe { prepare_texture_upload(device, &texture, catalog) }?;

        let allocator: ID3D12CommandAllocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .context("surface upload allocator creation failed")?;
        let list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, None)
        }
        .context("surface upload command list creation failed")?;
        for ((default, upload), bytes) in defaults.iter().zip(&uploads).zip(&payloads) {
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
        unsafe { copy_texture(&list, &texture, &texture_upload, &layouts) };
        unsafe {
            crate::rendering::device::transition(
                &list,
                &texture,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            list.Close()
        }
        .context("surface upload command list close failed")?;
        unsafe { execute_and_wait(device, queue, &list) }?;

        let mut defaults = defaults.into_iter();
        Ok(Self {
            vertices: defaults.next().unwrap(),
            primitives: defaults.next().unwrap(),
            materials: defaults.next().unwrap(),
            texture,
            total_bytes: catalog.gpu_bytes(),
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
    .context("surface buffer upload map failed")?;
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast(), bytes.len()) };
    unsafe { resource.Unmap(0, None) };
    Ok(())
}

unsafe fn create_texture(device: &ID3D12Device) -> Result<ID3D12Resource> {
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(TEXTURE_SIDE),
        Height: TEXTURE_SIDE,
        DepthOrArraySize: MATERIAL_COUNT as u16,
        MipLevels: MIP_COUNT as u16,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_NONE,
    };
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
            &mut resource,
        )
    }
    .context("surface texture allocation failed")?;
    resource.context("surface texture allocation returned no resource")
}

unsafe fn prepare_texture_upload(
    device: &ID3D12Device,
    texture: &ID3D12Resource,
    catalog: &Catalog,
) -> Result<(ID3D12Resource, Vec<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>)> {
    let count = MATERIAL_COUNT * MIP_COUNT;
    let desc = unsafe { texture.GetDesc() };
    let mut layouts = vec![D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default(); count as usize];
    let mut rows = vec![0u32; count as usize];
    let mut row_sizes = vec![0u64; count as usize];
    let mut total_bytes = 0;
    unsafe {
        device.GetCopyableFootprints(
            &desc,
            0,
            count,
            0,
            Some(layouts.as_mut_ptr()),
            Some(rows.as_mut_ptr()),
            Some(row_sizes.as_mut_ptr()),
            Some(&mut total_bytes),
        )
    };
    let upload = unsafe {
        create_buffer(
            device,
            total_bytes,
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
    .context("surface texture upload map failed")?;
    for layer in 0..MATERIAL_COUNT {
        for mip in 0..MIP_COUNT {
            let index = (layer * MIP_COUNT + mip) as usize;
            let side = (TEXTURE_SIDE >> mip).max(1) as usize;
            let source = &catalog.texture_mips[mip as usize];
            let layer_start = layer as usize * side * side * 4;
            for row in 0..side {
                let source_start = layer_start + row * side * 4;
                let destination = unsafe {
                    mapped.cast::<u8>().add(
                        layouts[index].Offset as usize
                            + row * layouts[index].Footprint.RowPitch as usize,
                    )
                };
                unsafe {
                    ptr::copy_nonoverlapping(
                        source.as_ptr().add(source_start),
                        destination,
                        side * 4,
                    )
                };
            }
            debug_assert_eq!(rows[index] as usize, side);
            debug_assert_eq!(row_sizes[index] as usize, side * 4);
        }
    }
    unsafe { upload.Unmap(0, None) };
    Ok((upload, layouts))
}

unsafe fn copy_texture(
    list: &ID3D12GraphicsCommandList,
    texture: &ID3D12Resource,
    upload: &ID3D12Resource,
    layouts: &[D3D12_PLACED_SUBRESOURCE_FOOTPRINT],
) {
    for (index, layout) in layouts.iter().enumerate() {
        let mut source = D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(upload.clone())),
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: *layout,
            },
        };
        let mut destination = D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(texture.clone())),
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: index as u32,
            },
        };
        unsafe {
            list.CopyTextureRegion(&destination, 0, 0, 0, &source, None);
            ManuallyDrop::drop(&mut source.pResource);
            ManuallyDrop::drop(&mut destination.pResource);
        }
    }
}

unsafe fn execute_and_wait(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
    list: &ID3D12GraphicsCommandList,
) -> Result<()> {
    let command: ID3D12CommandList = list.cast()?;
    unsafe { queue.ExecuteCommandLists(&[Some(command)]) };
    let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
        .context("surface upload fence creation failed")?;
    let event = unsafe { CreateEventW(None, false, false, None) }
        .context("surface upload event creation failed")?;
    unsafe { queue.Signal(&fence, 1) }.context("surface upload signal failed")?;
    unsafe { fence.SetEventOnCompletion(1, event) }.context("surface upload wait setup failed")?;
    let wait = unsafe { WaitForSingleObject(event, INFINITE) };
    unsafe { CloseHandle(event) }.context("surface upload event close failed")?;
    anyhow::ensure!(
        wait == WAIT_OBJECT_0,
        "surface upload wait returned {wait:?}"
    );
    Ok(())
}
