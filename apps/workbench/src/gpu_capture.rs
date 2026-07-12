use std::mem::ManuallyDrop;
use std::ptr;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};

pub struct Readback {
    resource: ID3D12Resource,
    layout: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    rows: u32,
    row_size: u64,
    total_bytes: u64,
    width: u32,
    height: u32,
}

pub struct CapturedPixels {
    pub width: u32,
    pub height: u32,
    pub row_pitch: u32,
    pub allocation_bytes: u64,
    pub row_copy_ms: f64,
    pub rgba: Vec<u8>,
}

impl Readback {
    pub unsafe fn new(device: &ID3D12Device, source: &ID3D12Resource) -> Result<Self> {
        let desc = unsafe { source.GetDesc() };
        let mut layout = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
        let mut rows = 0;
        let mut row_size = 0;
        let mut total_bytes = 0;
        unsafe {
            device.GetCopyableFootprints(
                &desc,
                0,
                1,
                0,
                Some(&mut layout),
                Some(&mut rows),
                Some(&mut row_size),
                Some(&mut total_bytes),
            );
        }
        if rows != desc.Height || row_size != desc.Width * 4 || total_bytes == 0 {
            bail!("unexpected R8G8B8A8 capture footprint");
        }

        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_READBACK,
            ..Default::default()
        };
        let buffer_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: total_bytes,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: D3D12_RESOURCE_FLAG_NONE,
        };
        let mut resource = None;
        unsafe {
            device.CreateCommittedResource(
                &heap,
                D3D12_HEAP_FLAG_NONE,
                &buffer_desc,
                D3D12_RESOURCE_STATE_COPY_DEST,
                None,
                &mut resource,
            )
        }
        .context("capture readback allocation failed")?;
        Ok(Self {
            resource: resource.context("capture readback allocation returned no resource")?,
            layout,
            rows,
            row_size,
            total_bytes,
            width: u32::try_from(desc.Width).context("capture width exceeds u32")?,
            height: desc.Height,
        })
    }

    pub unsafe fn record(&self, command_list: &ID3D12GraphicsCommandList, source: &ID3D12Resource) {
        let mut source_location = D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(source.clone())),
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: 0,
            },
        };
        let mut destination_location = D3D12_TEXTURE_COPY_LOCATION {
            pResource: ManuallyDrop::new(Some(self.resource.clone())),
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: self.layout,
            },
        };
        unsafe {
            command_list.CopyTextureRegion(&destination_location, 0, 0, 0, &source_location, None);
            ManuallyDrop::drop(&mut source_location.pResource);
            ManuallyDrop::drop(&mut destination_location.pResource);
        }
    }

    pub unsafe fn read(&self) -> Result<CapturedPixels> {
        let allocation_size = usize::try_from(self.total_bytes)
            .context("capture allocation is too large for this process")?;
        let tight_row_bytes =
            usize::try_from(self.row_size).context("capture row is too large for this process")?;
        let row_pitch = self.layout.Footprint.RowPitch as usize;
        let row_count = self.rows as usize;
        let tight_size = tight_row_bytes
            .checked_mul(row_count)
            .context("capture pixel size overflow")?;
        let mut mapped = ptr::null_mut();
        let read_range = D3D12_RANGE {
            Begin: 0,
            End: allocation_size,
        };
        unsafe { self.resource.Map(0, Some(&read_range), Some(&mut mapped)) }
            .context("capture readback map failed")?;

        let copy_start = Instant::now();
        let mut rgba = vec![0u8; tight_size];
        for row in 0..row_count {
            unsafe {
                ptr::copy_nonoverlapping(
                    mapped.cast::<u8>().add(row * row_pitch),
                    rgba.as_mut_ptr().add(row * tight_row_bytes),
                    tight_row_bytes,
                );
            }
        }
        let row_copy_ms = copy_start.elapsed().as_secs_f64() * 1_000.0;
        let no_write = D3D12_RANGE { Begin: 0, End: 0 };
        unsafe { self.resource.Unmap(0, Some(&no_write)) };

        Ok(CapturedPixels {
            width: self.width,
            height: self.height,
            row_pitch: self.layout.Footprint.RowPitch,
            allocation_bytes: self.total_bytes,
            row_copy_ms,
            rgba,
        })
    }
}
