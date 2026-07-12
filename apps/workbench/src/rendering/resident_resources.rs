use std::mem::{ManuallyDrop, size_of};
use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};

use crate::resident::{
    ACTIVE_MAPPING_BYTES, CACHE_REGION_CAPACITY, REGION_INSTANCE_BYTES, StreamPlan, as_bytes,
};

pub(super) const QUERY_COUNT: u32 = 4;

pub(super) unsafe fn write_staging(
    instance_upload: &ID3D12Resource,
    active_upload: &ID3D12Resource,
    plan: &StreamPlan,
) -> Result<()> {
    let mut mapped = ptr::null_mut();
    unsafe {
        instance_upload.Map(
            0,
            Some(&D3D12_RANGE { Begin: 0, End: 0 }),
            Some(&mut mapped),
        )
    }
    .context("resident instance upload map failed")?;
    for upload in &plan.uploads {
        let offset = upload.slot as usize * REGION_INSTANCE_BYTES;
        let bytes = as_bytes(&upload.records);
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast::<u8>().add(offset), bytes.len());
        }
    }
    unsafe {
        instance_upload.Unmap(
            0,
            Some(&D3D12_RANGE {
                Begin: 0,
                End: CACHE_REGION_CAPACITY * REGION_INSTANCE_BYTES,
            }),
        )
    };

    let mut active_mapped = ptr::null_mut();
    unsafe {
        active_upload.Map(
            0,
            Some(&D3D12_RANGE { Begin: 0, End: 0 }),
            Some(&mut active_mapped),
        )
    }
    .context("resident active upload map failed")?;
    let active_bytes = as_bytes(&plan.active_regions);
    unsafe {
        ptr::copy_nonoverlapping(
            active_bytes.as_ptr(),
            active_mapped.cast::<u8>(),
            active_bytes.len(),
        );
        active_upload.Unmap(
            0,
            Some(&D3D12_RANGE {
                Begin: 0,
                End: active_bytes.len(),
            }),
        );
    }
    Ok(())
}

pub(super) unsafe fn record_stream_copies(
    command_list: &ID3D12GraphicsCommandList,
    instances: &ID3D12Resource,
    active_regions: &ID3D12Resource,
    instance_upload: &ID3D12Resource,
    active_upload: &ID3D12Resource,
    copy_slots: &[u32],
) {
    unsafe {
        transition(
            command_list,
            instances,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_COPY_DEST,
        );
        transition(
            command_list,
            active_regions,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_COPY_DEST,
        );
        for slot in copy_slots {
            let offset = u64::from(*slot) * REGION_INSTANCE_BYTES as u64;
            command_list.CopyBufferRegion(
                instances,
                offset,
                instance_upload,
                offset,
                REGION_INSTANCE_BYTES as u64,
            );
        }
        command_list.CopyBufferRegion(
            active_regions,
            0,
            active_upload,
            0,
            ACTIVE_MAPPING_BYTES as u64,
        );
        transition(
            command_list,
            instances,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        );
        transition(
            command_list,
            active_regions,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        );
    }
}

pub(super) unsafe fn create_query_heap(device: &ID3D12Device) -> Result<ID3D12QueryHeap> {
    let desc = D3D12_QUERY_HEAP_DESC {
        Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
        Count: QUERY_COUNT,
        NodeMask: 0,
    };
    let mut heap = None;
    unsafe { device.CreateQueryHeap(&desc, &mut heap) }
        .context("resident query heap creation failed")?;
    heap.context("resident query heap creation returned no heap")
}

pub(super) unsafe fn create_buffer(
    device: &ID3D12Device,
    size: u64,
    heap_type: D3D12_HEAP_TYPE,
    initial_state: D3D12_RESOURCE_STATES,
    flags: D3D12_RESOURCE_FLAGS,
) -> Result<ID3D12Resource> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: heap_type,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width: size,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: flags,
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            initial_state,
            None,
            &mut resource,
        )
    }
    .context("resident buffer allocation failed")?;
    resource.context("resident buffer allocation returned no resource")
}

pub(super) unsafe fn read_values<T: Copy>(
    resource: &ID3D12Resource,
    count: usize,
) -> Result<Vec<T>> {
    let byte_count = count * size_of::<T>();
    let mut mapped = ptr::null_mut();
    let range = D3D12_RANGE {
        Begin: 0,
        End: byte_count,
    };
    unsafe { resource.Map(0, Some(&range), Some(&mut mapped)) }
        .context("resident readback map failed")?;
    let mut values = Vec::<T>::with_capacity(count);
    unsafe {
        ptr::copy_nonoverlapping(mapped.cast::<T>(), values.as_mut_ptr(), count);
        values.set_len(count);
    }
    unsafe { resource.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 })) };
    Ok(values)
}

pub(super) unsafe fn transition(
    command_list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    };
    unsafe { command_list.ResourceBarrier(std::slice::from_ref(&barrier)) };
    unsafe {
        let transition = &mut *barrier.Anonymous.Transition;
        ManuallyDrop::drop(&mut transition.pResource);
    }
}

pub(super) unsafe fn uav_barrier(
    command_list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            UAV: ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
            }),
        },
    };
    unsafe { command_list.ResourceBarrier(std::slice::from_ref(&barrier)) };
    unsafe {
        let uav = &mut *barrier.Anonymous.UAV;
        ManuallyDrop::drop(&mut uav.pResource);
    }
}

pub(super) unsafe fn set_viewport(
    command_list: &ID3D12GraphicsCommandList,
    width: u32,
    height: u32,
) {
    unsafe {
        command_list.RSSetViewports(&[D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        }]);
        command_list.RSSetScissorRects(&[RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        }]);
    }
}
