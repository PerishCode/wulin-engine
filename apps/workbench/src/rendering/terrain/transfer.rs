use std::collections::BTreeSet;
use std::ptr;
use std::time::Instant;

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::load::LoadConfig;
use crate::terrain::{
    GlobalTerrainConfig, TERRAIN_STREAM_REVISION, TerrainAssignment, TerrainIoMetrics,
    TerrainReservationReport, TerrainTransactionReport, TerrainUpload,
};

use super::super::resident::{create_buffer, transition};
use super::cache::{LayoutPlan, TerrainCache};
use super::copy_timing::CopyTimer;
use super::descriptors::create_heap;

pub(super) use super::cache::{TERRAIN_ACTIVE_CAPACITY, TERRAIN_CACHE_CAPACITY};

struct ReservedTransfer {
    transaction_id: u64,
    layout: LayoutPlan,
    started_at: Instant,
}

struct PendingTransfer {
    next_cache: TerrainCache,
    next_tiles: Vec<Option<terrain_format::TerrainTile>>,
    active: Vec<TerrainAssignment>,
    uploaded_slots: Vec<u32>,
    report: TerrainTransactionReport,
    started_at: Instant,
}

pub struct TerrainPublication {
    pub active: Vec<TerrainAssignment>,
    pub tiles: Vec<terrain_format::TerrainTile>,
    pub report: TerrainTransactionReport,
}

pub struct TerrainTransfer {
    regions: Vec<ID3D12Resource>,
    heap: ID3D12DescriptorHeap,
    region_allocation_bytes: u64,
    upload: ID3D12Resource,
    release_allocator: ID3D12CommandAllocator,
    release_list: ID3D12GraphicsCommandList,
    copy_queue: ID3D12CommandQueue,
    copy_allocator: ID3D12CommandAllocator,
    copy_list: ID3D12GraphicsCommandList,
    copy_timer: CopyTimer,
    copy_fence: ID3D12Fence,
    copy_event: HANDLE,
    next_copy_fence: u64,
    gate_fence: ID3D12Fence,
    armed_gate: Option<u64>,
    next_gate_fence: u64,
    cache: TerrainCache,
    tiles: Vec<Option<terrain_format::TerrainTile>>,
    shader_slots: [bool; TERRAIN_CACHE_CAPACITY],
    reservation: Option<ReservedTransfer>,
    pending: Option<PendingTransfer>,
    last_completed: Option<TerrainTransactionReport>,
    next_transaction_id: u64,
}

impl TerrainTransfer {
    pub unsafe fn new(
        device: &ID3D12Device,
        stats: &ID3D12Resource,
        seams: &ID3D12Resource,
        lod_stats: &ID3D12Resource,
    ) -> Result<Self> {
        let mut regions = Vec::with_capacity(TERRAIN_CACHE_CAPACITY);
        for _ in 0..TERRAIN_CACHE_CAPACITY {
            regions.push(unsafe {
                create_buffer(
                    device,
                    u64::from(terrain_format::PAYLOAD_BYTES),
                    D3D12_HEAP_TYPE_DEFAULT,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_FLAG_NONE,
                )
            }?);
        }
        let heap = unsafe { create_heap(device, &regions, stats, seams, lod_stats) }?;
        let region_allocation_bytes =
            unsafe { device.GetResourceAllocationInfo(0, &[regions[0].GetDesc()]) }.SizeInBytes
                * TERRAIN_CACHE_CAPACITY as u64;
        let upload = unsafe {
            create_buffer(
                device,
                TERRAIN_CACHE_CAPACITY as u64 * u64::from(terrain_format::PAYLOAD_BYTES),
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let release_allocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .context("terrain release allocator creation failed")?;
        let release_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &release_allocator, None)
        }
        .context("terrain release list creation failed")?;
        unsafe { release_list.Close() }.context("terrain release list close failed")?;
        let copy_queue: ID3D12CommandQueue = unsafe {
            device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_COPY,
                Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            })
        }
        .context("terrain copy queue creation failed")?;
        let copy_allocator = unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COPY) }
            .context("terrain copy allocator creation failed")?;
        let copy_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_COPY, &copy_allocator, None)
        }
        .context("terrain copy list creation failed")?;
        unsafe { copy_list.Close() }.context("terrain copy list close failed")?;
        let copy_timer = unsafe { CopyTimer::new(device, &copy_queue) }?;
        let copy_fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("terrain copy fence creation failed")?;
        let gate_fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("terrain gate fence creation failed")?;
        let copy_event = unsafe { CreateEventW(None, false, false, None) }
            .context("terrain copy event creation failed")?;
        Ok(Self {
            regions,
            heap,
            region_allocation_bytes,
            upload,
            release_allocator,
            release_list,
            copy_queue,
            copy_allocator,
            copy_list,
            copy_timer,
            copy_fence,
            copy_event,
            next_copy_fence: 1,
            gate_fence,
            armed_gate: None,
            next_gate_fence: 1,
            cache: TerrainCache::default(),
            tiles: std::iter::repeat_with(|| None)
                .take(TERRAIN_CACHE_CAPACITY)
                .collect(),
            shader_slots: [false; TERRAIN_CACHE_CAPACITY],
            reservation: None,
            pending: None,
            last_completed: None,
            next_transaction_id: 1,
        })
    }

    pub fn reserve(
        &mut self,
        config: LoadConfig,
        protected: &BTreeSet<u32>,
    ) -> Result<TerrainReservationReport> {
        if self.reservation.is_some() || self.pending.is_some() {
            bail!("terrain_stream_busy");
        }
        let layout = self.cache.plan(config, protected)?;
        self.reserve_layout(layout)
    }

    pub fn reserve_global(
        &mut self,
        config: GlobalTerrainConfig,
        protected: &BTreeSet<u32>,
    ) -> Result<TerrainReservationReport> {
        if self.reservation.is_some() || self.pending.is_some() {
            bail!("terrain_stream_busy");
        }
        let layout = self.cache.plan_global(config, protected)?;
        self.reserve_layout(layout)
    }

    fn reserve_layout(&mut self, layout: LayoutPlan) -> Result<TerrainReservationReport> {
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id += 1;
        let report = TerrainReservationReport {
            revision: TERRAIN_STREAM_REVISION,
            transaction_id,
            config: layout.config,
            global_config: layout.global_config,
            counts: layout.counts,
            assignments: layout.assignments.clone(),
        };
        self.reservation = Some(ReservedTransfer {
            transaction_id,
            layout,
            started_at: Instant::now(),
        });
        Ok(report)
    }

    pub fn cancel(&mut self, transaction_id: u64) -> Result<()> {
        let reservation = self
            .reservation
            .as_ref()
            .context("terrain transfer has no reservation")?;
        ensure!(
            reservation.transaction_id == transaction_id,
            "terrain reservation mismatch"
        );
        self.reservation = None;
        Ok(())
    }

    pub unsafe fn submit(
        &mut self,
        transaction_id: u64,
        uploads: Vec<TerrainUpload>,
        io: TerrainIoMetrics,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<TerrainTransactionReport> {
        let reservation = self
            .reservation
            .take()
            .context("terrain transfer has no reservation")?;
        ensure!(
            reservation.transaction_id == transaction_id,
            "terrain reservation mismatch"
        );
        ensure!(
            uploads.len() == reservation.layout.assignments.len(),
            "terrain upload count mismatch"
        );
        for (assignment, upload) in reservation.layout.assignments.iter().zip(&uploads) {
            ensure!(
                assignment.slot == upload.slot
                    && assignment.region_id == upload.region_id
                    && assignment.global_region == upload.global_region,
                "terrain upload assignment mismatch"
            );
            ensure!(
                upload.tile.region_id == upload.region_id,
                "terrain upload tile mismatch"
            );
        }
        unsafe { self.write_uploads(&uploads) }?;

        unsafe { self.release_allocator.Reset() }
            .context("terrain release allocator reset failed")?;
        unsafe { self.release_list.Reset(&self.release_allocator, None) }
            .context("terrain release list reset failed")?;
        for slot in &reservation.layout.reused_slots {
            let index = *slot as usize;
            ensure!(
                self.shader_slots[index],
                "reused terrain slot is not shader-readable"
            );
            unsafe {
                transition(
                    &self.release_list,
                    &self.regions[index],
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                )
            };
            self.shader_slots[index] = false;
        }
        unsafe { self.release_list.Close() }.context("terrain release list close failed")?;
        let release_list: ID3D12CommandList = self.release_list.cast()?;
        unsafe {
            direct_queue.ExecuteCommandLists(&[Some(release_list)]);
            direct_queue.Signal(direct_fence, direct_release_fence)
        }
        .context("terrain release signal failed")?;

        unsafe { self.copy_allocator.Reset() }.context("terrain copy allocator reset failed")?;
        unsafe { self.copy_list.Reset(&self.copy_allocator, None) }
            .context("terrain copy list reset failed")?;
        unsafe { self.copy_timer.begin(&self.copy_list) };
        for upload in &uploads {
            let offset = u64::from(upload.slot) * u64::from(terrain_format::PAYLOAD_BYTES);
            unsafe {
                self.copy_list.CopyBufferRegion(
                    &self.regions[upload.slot as usize],
                    0,
                    &self.upload,
                    offset,
                    u64::from(terrain_format::PAYLOAD_BYTES),
                )
            };
        }
        unsafe { self.copy_timer.end_and_resolve(&self.copy_list) };
        unsafe { self.copy_list.Close() }.context("terrain copy list close failed")?;
        let copy_gate_fence = self.armed_gate;
        if let Some(value) = copy_gate_fence {
            unsafe { self.copy_queue.Wait(&self.gate_fence, value) }
                .context("terrain copy gate wait failed")?;
        }
        unsafe { self.copy_queue.Wait(direct_fence, direct_release_fence) }
            .context("terrain copy release wait failed")?;
        let copy_list: ID3D12CommandList = self.copy_list.cast()?;
        unsafe { self.copy_queue.ExecuteCommandLists(&[Some(copy_list)]) };
        let copy_fence = self.next_copy_fence;
        self.next_copy_fence += 1;
        unsafe { self.copy_queue.Signal(&self.copy_fence, copy_fence) }
            .context("terrain copy signal failed")?;

        let mut hash = Sha256::new();
        let mut next_tiles = self.tiles.clone();
        for upload in &uploads {
            hash.update(upload.slot.to_le_bytes());
            hash.update(upload.region_id.to_le_bytes());
            if let Some(global) = upload.global_region {
                hash.update(global.x.to_le_bytes());
                hash.update(global.z.to_le_bytes());
            }
            hash.update(upload.payload.as_slice());
            hash.update(upload.sha256.as_bytes());
            next_tiles[upload.slot as usize] = Some(upload.tile.clone());
        }
        let report = TerrainTransactionReport {
            revision: TERRAIN_STREAM_REVISION,
            transaction_id,
            config: reservation.layout.config,
            global_config: reservation.layout.global_config,
            counts: reservation.layout.counts,
            uploaded_sha256: format!("{:x}", hash.finalize()),
            direct_release_fence,
            copy_fence,
            copy_gate_fence,
            io,
            schedule_ms: reservation.started_at.elapsed().as_secs_f64() * 1_000.0,
            copy_gpu_ms: 0.0,
            copy_to_publication_ms: 0.0,
            pending_ms: 0.0,
        };
        self.pending = Some(PendingTransfer {
            next_cache: reservation.layout.next_cache,
            next_tiles,
            active: reservation.layout.active,
            uploaded_slots: uploads.iter().map(|upload| upload.slot).collect(),
            report: report.clone(),
            started_at: reservation.started_at,
        });
        Ok(report)
    }

    pub unsafe fn poll(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Result<Option<TerrainPublication>> {
        let Some(pending) = self.pending.as_ref() else {
            return Ok(None);
        };
        if unsafe { self.copy_fence.GetCompletedValue() } < pending.report.copy_fence {
            return Ok(None);
        }
        let copy_gpu_ms = unsafe { self.copy_timer.read_ms() }?;
        let mut pending = self.pending.take().expect("terrain pending disappeared");
        pending.report.copy_gpu_ms = copy_gpu_ms;
        for slot in &pending.uploaded_slots {
            let index = *slot as usize;
            unsafe {
                transition(
                    command_list,
                    &self.regions[index],
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                )
            };
            self.shader_slots[index] = true;
        }
        let tiles = pending
            .active
            .iter()
            .map(|entry| {
                pending.next_tiles[entry.slot as usize]
                    .clone()
                    .expect("active terrain tile missing")
            })
            .collect();
        self.cache = pending.next_cache;
        self.tiles = pending.next_tiles;
        pending.report.pending_ms = pending.started_at.elapsed().as_secs_f64() * 1_000.0;
        pending.report.copy_to_publication_ms =
            (pending.report.pending_ms - pending.report.schedule_ms).max(0.0);
        self.last_completed = Some(pending.report.clone());
        Ok(Some(TerrainPublication {
            active: pending.active,
            tiles,
            report: pending.report,
        }))
    }

    pub fn arm_gate(&mut self) -> Result<u64> {
        if self.reservation.is_some() || self.pending.is_some() || self.armed_gate.is_some() {
            bail!("terrain copy gate or transaction is already active");
        }
        let value = self.next_gate_fence;
        self.next_gate_fence += 1;
        self.armed_gate = Some(value);
        Ok(value)
    }

    pub fn descriptor_heap(&self) -> &ID3D12DescriptorHeap {
        &self.heap
    }

    pub unsafe fn release_gate(&mut self) -> Result<u64> {
        let value = self.armed_gate.context("terrain copy gate is not armed")?;
        unsafe { self.gate_fence.Signal(value) }.context("terrain copy gate signal failed")?;
        self.armed_gate = None;
        Ok(value)
    }

    pub fn status_json(&self) -> serde_json::Value {
        serde_json::json!({
            "cacheCapacity": TERRAIN_CACHE_CAPACITY,
            "activeCapacity": TERRAIN_ACTIVE_CAPACITY,
            "payloadBytesPerRegion": terrain_format::PAYLOAD_BYTES,
            "payloadArenaBytes": TERRAIN_CACHE_CAPACITY as u32 * terrain_format::PAYLOAD_BYTES,
            "defaultHeapAllocationBytes": self.region_allocation_bytes,
            "copyTimestampBytes": CopyTimer::READBACK_BYTES,
            "reservation": self.reservation.as_ref().map(|value| value.transaction_id),
            "copyPending": self.pending.as_ref().map(|value| value.report.transaction_id),
            "copyGate": self.armed_gate,
            "lastCompleted": self.last_completed,
        })
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        if let Some(value) = self.armed_gate.take() {
            unsafe { self.gate_fence.Signal(value) }
                .context("terrain shutdown gate signal failed")?;
        }
        let Some(value) = self
            .pending
            .as_ref()
            .map(|pending| pending.report.copy_fence)
        else {
            return Ok(());
        };
        if unsafe { self.copy_fence.GetCompletedValue() } >= value {
            return Ok(());
        }
        unsafe { self.copy_fence.SetEventOnCompletion(value, self.copy_event) }
            .context("terrain copy completion event failed")?;
        let wait = unsafe { WaitForSingleObject(self.copy_event, INFINITE) };
        ensure!(wait == WAIT_OBJECT_0, "terrain copy wait returned {wait:?}");
        Ok(())
    }

    unsafe fn write_uploads(&self, uploads: &[TerrainUpload]) -> Result<()> {
        let mut mapped = ptr::null_mut();
        unsafe {
            self.upload.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut mapped),
            )
        }
        .context("terrain upload arena map failed")?;
        for upload in uploads {
            let offset = upload.slot as usize * terrain_format::PAYLOAD_BYTES as usize;
            unsafe {
                ptr::copy_nonoverlapping(
                    upload.payload.as_ptr(),
                    mapped.cast::<u8>().add(offset),
                    upload.payload.len(),
                )
            };
        }
        unsafe {
            self.upload.Unmap(
                0,
                Some(&D3D12_RANGE {
                    Begin: 0,
                    End: TERRAIN_CACHE_CAPACITY * terrain_format::PAYLOAD_BYTES as usize,
                }),
            )
        };
        Ok(())
    }
}

impl Drop for TerrainTransfer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_idle();
            let _ = CloseHandle(self.copy_event);
        }
    }
}
