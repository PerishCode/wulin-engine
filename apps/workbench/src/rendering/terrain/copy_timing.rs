use anyhow::{Context, Result, ensure};
use windows::Win32::Graphics::Direct3D12::*;

use super::super::resident::{create_buffer, read_values};

pub(super) struct CopyTimer {
    heap: ID3D12QueryHeap,
    readback: ID3D12Resource,
    frequency: u64,
}

impl CopyTimer {
    pub const READBACK_BYTES: u64 = 2 * size_of::<u64>() as u64;

    pub unsafe fn new(device: &ID3D12Device, queue: &ID3D12CommandQueue) -> Result<Self> {
        let mut heap = None;
        unsafe {
            device.CreateQueryHeap(
                &D3D12_QUERY_HEAP_DESC {
                    Type: D3D12_QUERY_HEAP_TYPE_COPY_QUEUE_TIMESTAMP,
                    Count: 2,
                    NodeMask: 0,
                },
                &mut heap,
            )
        }
        .context("terrain copy timestamp heap creation failed")?;
        let heap = heap.context("terrain copy timestamp heap creation returned no heap")?;
        let readback = unsafe {
            create_buffer(
                device,
                Self::READBACK_BYTES,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let frequency = unsafe { queue.GetTimestampFrequency() }
            .context("terrain copy timestamp frequency query failed")?;
        ensure!(frequency > 0, "terrain copy timestamp frequency is zero");
        Ok(Self {
            heap,
            readback,
            frequency,
        })
    }

    pub unsafe fn begin(&self, command_list: &ID3D12GraphicsCommandList) {
        unsafe { command_list.EndQuery(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, 0) };
    }

    pub unsafe fn end_and_resolve(&self, command_list: &ID3D12GraphicsCommandList) {
        unsafe {
            command_list.EndQuery(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, 1);
            command_list.ResolveQueryData(
                &self.heap,
                D3D12_QUERY_TYPE_TIMESTAMP,
                0,
                2,
                &self.readback,
                0,
            );
        }
    }

    pub unsafe fn read_ms(&self) -> Result<f64> {
        let timestamps = unsafe { read_values::<u64>(&self.readback, 2) }?;
        Ok(timestamps[1].saturating_sub(timestamps[0]) as f64 * 1_000.0 / self.frequency as f64)
    }
}
