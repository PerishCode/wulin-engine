mod oracle;
mod probe;

pub(super) use oracle::{BoundProof, OcclusionOracle, validate_fixture_bound};
pub(super) use probe::read as read_probe;

use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R32_UINT, DXGI_SAMPLE_DESC};

use crate::rendering::resident::create_buffer;

use super::resources::CANDIDATE_CAPACITY;

pub const FILTERED_VISIBLE_BYTES: u64 = CANDIDATE_CAPACITY as u64 * 24;
pub const OCCLUSION_COUNTER_BYTES: u64 = 80;
pub const OCCLUSION_MASK_BYTES: u64 = CANDIDATE_CAPACITY as u64 * 4;
pub const OCCLUSION_GROUPS: u32 = CANDIDATE_CAPACITY.div_ceil(256);
pub const GROUP_OFFSETS_BYTES: u64 = OCCLUSION_GROUPS as u64 * 4;
pub const BOUND_RADIAL_SCALE: f32 = 0.35;
pub const BOUND_RADIAL_BIAS: f32 = 0.25;
pub const BOUND_VERTICAL_PAD: f32 = 0.25;
pub const PIXEL_EXPANSION: f32 = 2.0;
pub const DEPTH_BIAS: f32 = 0.000001;

pub struct OcclusionResources {
    pub filtered_visible: ID3D12Resource,
    pub counters: ID3D12Resource,
    pub candidate_mask: ID3D12Resource,
    pub group_offsets: ID3D12Resource,
    pub hierarchy: ID3D12Resource,
    pub counter_readback: ID3D12Resource,
    pub mask_readback: ID3D12Resource,
    pub order_readback: ID3D12Resource,
    pub hierarchy_readback: HierarchyReadback,
    pub mip_count: u32,
}

impl OcclusionResources {
    pub unsafe fn new(device: &ID3D12Device, width: u32, height: u32) -> Result<Self> {
        let mip_count = mip_count(width, height);
        let filtered_visible = unsafe { uav_buffer(device, FILTERED_VISIBLE_BYTES) }?;
        let counters = unsafe { uav_buffer(device, OCCLUSION_COUNTER_BYTES) }?;
        let candidate_mask = unsafe { uav_buffer(device, OCCLUSION_MASK_BYTES) }?;
        let group_offsets = unsafe { uav_buffer(device, GROUP_OFFSETS_BYTES) }?;
        let hierarchy = unsafe { hierarchy_texture(device, width, height, mip_count) }?;
        let counter_readback = unsafe { readback_buffer(device, OCCLUSION_COUNTER_BYTES) }?;
        let mask_readback = unsafe { readback_buffer(device, OCCLUSION_MASK_BYTES) }?;
        let order_readback = unsafe { readback_buffer(device, FILTERED_VISIBLE_BYTES * 2) }?;
        let hierarchy_readback = unsafe { HierarchyReadback::new(device, &hierarchy) }?;
        Ok(Self {
            filtered_visible,
            counters,
            candidate_mask,
            group_offsets,
            hierarchy,
            counter_readback,
            mask_readback,
            order_readback,
            hierarchy_readback,
            mip_count,
        })
    }
}

pub struct HierarchyReadback {
    resource: ID3D12Resource,
    layouts: Vec<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>,
    rows: Vec<u32>,
    row_sizes: Vec<u64>,
    total_bytes: u64,
}

pub struct HierarchyMip {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

impl HierarchyReadback {
    unsafe fn new(device: &ID3D12Device, source: &ID3D12Resource) -> Result<Self> {
        let desc = unsafe { source.GetDesc() };
        let mip_count = usize::from(desc.MipLevels);
        let mut layouts = vec![D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default(); mip_count];
        let mut rows = vec![0; mip_count];
        let mut row_sizes = vec![0; mip_count];
        let mut total_bytes = 0;
        unsafe {
            device.GetCopyableFootprints(
                &desc,
                0,
                mip_count as u32,
                0,
                Some(layouts.as_mut_ptr()),
                Some(rows.as_mut_ptr()),
                Some(row_sizes.as_mut_ptr()),
                Some(&mut total_bytes),
            );
        }
        if total_bytes == 0 {
            bail!("occlusion hierarchy readback footprint is empty");
        }
        for (mip, ((layout, rows), row_size)) in
            layouts.iter().zip(&rows).zip(&row_sizes).enumerate()
        {
            let width = (desc.Width >> mip).max(1);
            let height = (u64::from(desc.Height) >> mip).max(1);
            if u64::from(*rows) != height || *row_size != width * 4 {
                bail!("occlusion hierarchy mip {mip} has an unexpected footprint");
            }
            if layout.Footprint.Format != DXGI_FORMAT_R32_UINT {
                bail!("occlusion hierarchy mip {mip} readback format differs");
            }
        }
        let resource = unsafe { readback_buffer(device, total_bytes) }?;
        Ok(Self {
            resource,
            layouts,
            rows,
            row_sizes,
            total_bytes,
        })
    }

    pub unsafe fn record(&self, command_list: &ID3D12GraphicsCommandList, source: &ID3D12Resource) {
        for (mip, layout) in self.layouts.iter().copied().enumerate() {
            let mut source_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(source.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: mip as u32,
                },
            };
            let mut destination_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(self.resource.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: layout,
                },
            };
            unsafe {
                command_list.CopyTextureRegion(
                    &destination_location,
                    0,
                    0,
                    0,
                    &source_location,
                    None,
                );
                ManuallyDrop::drop(&mut source_location.pResource);
                ManuallyDrop::drop(&mut destination_location.pResource);
            }
        }
    }

    pub unsafe fn read(&self) -> Result<Vec<HierarchyMip>> {
        let allocation_size = usize::try_from(self.total_bytes)
            .context("occlusion hierarchy readback exceeds process size")?;
        let mut mapped = ptr::null_mut();
        let read_range = D3D12_RANGE {
            Begin: 0,
            End: allocation_size,
        };
        unsafe { self.resource.Map(0, Some(&read_range), Some(&mut mapped)) }
            .context("occlusion hierarchy readback map failed")?;
        let mut mips = Vec::with_capacity(self.layouts.len());
        for (mip, ((layout, rows), row_size)) in self
            .layouts
            .iter()
            .zip(&self.rows)
            .zip(&self.row_sizes)
            .enumerate()
        {
            let tight_row = usize::try_from(*row_size).context("hierarchy row is too large")?;
            let row_pitch = layout.Footprint.RowPitch as usize;
            let mut bytes = vec![0; tight_row * *rows as usize];
            for row in 0..*rows as usize {
                unsafe {
                    ptr::copy_nonoverlapping(
                        mapped
                            .cast::<u8>()
                            .add(layout.Offset as usize + row * row_pitch),
                        bytes.as_mut_ptr().add(row * tight_row),
                        tight_row,
                    );
                }
            }
            mips.push(HierarchyMip {
                width: layout.Footprint.Width,
                height: layout.Footprint.Height,
                bytes,
            });
            debug_assert_eq!(mips[mip].width, layout.Footprint.Width);
        }
        let no_write = D3D12_RANGE { Begin: 0, End: 0 };
        unsafe { self.resource.Unmap(0, Some(&no_write)) };
        Ok(mips)
    }
}

pub fn mip_count(width: u32, height: u32) -> u32 {
    u32::BITS - width.max(height).leading_zeros()
}

unsafe fn hierarchy_texture(
    device: &ID3D12Device,
    width: u32,
    height: u32,
    mip_count: u32,
) -> Result<ID3D12Resource> {
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(width),
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: mip_count as u16,
        Format: DXGI_FORMAT_R32_UINT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
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
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            None,
            &mut resource,
        )
    }
    .context("occlusion hierarchy allocation failed")?;
    resource.context("occlusion hierarchy allocation returned no resource")
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
