use std::collections::BTreeSet;
use std::ptr;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::async_resident::{
    ASYNC_CACHE_CAPACITY, ASYNC_RESIDENT_REVISION, AsyncLayoutPlan, AsyncRegionCache,
    AsyncReservationReport, AsyncTransactionReport, PayloadPreparation,
};
use crate::load::LoadConfig;
use crate::resident::{REGION_INSTANCE_BYTES, RegionUpload, as_bytes};

use super::super::resident::{create_buffer, transition};
use super::resources::create_descriptor_heap;

mod status;

pub struct AsyncTransfer {
    regions: Vec<ID3D12Resource>,
    descriptor_heap: ID3D12DescriptorHeap,
    region_allocation_bytes: u64,
    upload: ID3D12Resource,
    release_allocator: ID3D12CommandAllocator,
    release_list: ID3D12GraphicsCommandList,
    copy_queue: ID3D12CommandQueue,
    copy_allocator: ID3D12CommandAllocator,
    copy_list: ID3D12GraphicsCommandList,
    copy_fence: ID3D12Fence,
    copy_event: HANDLE,
    next_copy_fence: u64,
    gate_fence: ID3D12Fence,
    armed_gate: Option<u64>,
    next_gate_fence: u64,
    cache: AsyncRegionCache,
    reservation: Option<ReservedTransfer>,
    shader_slots: [bool; ASYNC_CACHE_CAPACITY],
    pending: Option<PendingTransfer>,
    last_completed: Option<AsyncTransactionReport>,
    next_transaction_id: u64,
}

struct ReservedTransfer {
    transaction_id: u64,
    layout: AsyncLayoutPlan,
    started_at: Instant,
}

struct PendingTransfer {
    next_cache: AsyncRegionCache,
    active_slots: Vec<u32>,
    uploaded_slots: Vec<u32>,
    report: AsyncTransactionReport,
    started_at: Instant,
}

pub struct Publication {
    pub config: LoadConfig,
    pub active_slots: Vec<u32>,
    pub report: AsyncTransactionReport,
}

impl AsyncTransfer {
    pub unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let mut regions = Vec::with_capacity(ASYNC_CACHE_CAPACITY);
        for _ in 0..ASYNC_CACHE_CAPACITY {
            regions.push(unsafe {
                create_buffer(
                    device,
                    REGION_INSTANCE_BYTES as u64,
                    D3D12_HEAP_TYPE_DEFAULT,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_FLAG_NONE,
                )
            }?);
        }
        let descriptor_heap = unsafe { create_descriptor_heap(device, &regions) }?;
        let region_allocation_bytes =
            unsafe { device.GetResourceAllocationInfo(0, &[regions[0].GetDesc()]) }.SizeInBytes
                * ASYNC_CACHE_CAPACITY as u64;
        let upload = unsafe {
            create_buffer(
                device,
                (ASYNC_CACHE_CAPACITY * REGION_INSTANCE_BYTES) as u64,
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let release_allocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                .context("async release allocator creation failed")?;
        let release_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &release_allocator, None)
        }
        .context("async release command list creation failed")?;
        unsafe { release_list.Close() }.context("async release command list close failed")?;

        let copy_queue = unsafe {
            device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_COPY,
                Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            })
        }
        .context("async copy queue creation failed")?;
        let copy_allocator = unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_COPY) }
            .context("async copy allocator creation failed")?;
        let copy_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_COPY, &copy_allocator, None)
        }
        .context("async copy command list creation failed")?;
        unsafe { copy_list.Close() }.context("async copy command list close failed")?;
        let copy_fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("async copy fence creation failed")?;
        let gate_fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("async gate fence creation failed")?;
        let copy_event = unsafe { CreateEventW(None, false, false, None) }
            .context("async event creation failed")?;

        Ok(Self {
            regions,
            descriptor_heap,
            region_allocation_bytes,
            upload,
            release_allocator,
            release_list,
            copy_queue,
            copy_allocator,
            copy_list,
            copy_fence,
            copy_event,
            next_copy_fence: 1,
            gate_fence,
            armed_gate: None,
            next_gate_fence: 1,
            cache: AsyncRegionCache::default(),
            reservation: None,
            shader_slots: [false; ASYNC_CACHE_CAPACITY],
            pending: None,
            last_completed: None,
            next_transaction_id: 1,
        })
    }

    pub unsafe fn schedule(
        &mut self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<AsyncTransactionReport> {
        let reservation = self.reserve(config, protected_slots)?;
        let generation_start = Instant::now();
        let uploads = reservation
            .assignments
            .iter()
            .map(|assignment| RegionUpload {
                slot: assignment.slot,
                records: crate::resident::generate_region(assignment.region_id),
            })
            .collect();
        let generation_ms = generation_start.elapsed().as_secs_f64() * 1_000.0;
        unsafe {
            self.submit(
                reservation.transaction_id,
                uploads,
                PayloadPreparation::generated(generation_ms),
                direct_queue,
                direct_fence,
                direct_release_fence,
            )
        }
    }

    pub fn reserve(
        &mut self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncReservationReport> {
        if self.reservation.is_some() || self.pending.is_some() {
            bail!("stream_busy");
        }
        let layout = self.cache.plan_layout(config, protected_slots)?;
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id += 1;
        let report = AsyncReservationReport {
            revision: ASYNC_RESIDENT_REVISION,
            transaction_id,
            config,
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

    pub fn cancel_reservation(&mut self, transaction_id: u64) -> Result<()> {
        let reservation = self
            .reservation
            .as_ref()
            .context("async transfer has no cache reservation")?;
        ensure_transaction(reservation.transaction_id, transaction_id)?;
        self.reservation = None;
        Ok(())
    }

    pub unsafe fn submit(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        preparation: PayloadPreparation,
        direct_queue: &ID3D12CommandQueue,
        direct_fence: &ID3D12Fence,
        direct_release_fence: u64,
    ) -> Result<AsyncTransactionReport> {
        let reservation = self
            .reservation
            .as_ref()
            .context("async transfer has no cache reservation")?;
        ensure_transaction(reservation.transaction_id, transaction_id)?;
        let reservation = self
            .reservation
            .take()
            .expect("async reservation disappeared");
        let plan = reservation.layout.materialize(uploads)?;
        unsafe { self.write_uploads(&plan.uploads) }?;

        unsafe { self.release_allocator.Reset() }
            .context("async release allocator reset failed")?;
        unsafe { self.release_list.Reset(&self.release_allocator, None) }
            .context("async release list reset failed")?;
        for slot in &plan.layout.reused_slots {
            let index = *slot as usize;
            if !self.shader_slots[index] {
                bail!("reused async slot {slot} is not in shader-resource state");
            }
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
        unsafe { self.release_list.Close() }.context("async release list close failed")?;
        let release_list: ID3D12CommandList = self.release_list.cast()?;
        unsafe {
            direct_queue.ExecuteCommandLists(&[Some(release_list)]);
            direct_queue.Signal(direct_fence, direct_release_fence)
        }
        .context("async direct release signal failed")?;

        unsafe { self.copy_allocator.Reset() }.context("async copy allocator reset failed")?;
        unsafe { self.copy_list.Reset(&self.copy_allocator, None) }
            .context("async copy list reset failed")?;
        for upload in &plan.uploads {
            let upload_offset = u64::from(upload.slot) * REGION_INSTANCE_BYTES as u64;
            unsafe {
                self.copy_list.CopyBufferRegion(
                    &self.regions[upload.slot as usize],
                    0,
                    &self.upload,
                    upload_offset,
                    REGION_INSTANCE_BYTES as u64,
                )
            };
        }
        unsafe { self.copy_list.Close() }.context("async copy list close failed")?;

        let gate_fence = self.armed_gate;
        if let Some(value) = gate_fence {
            unsafe { self.copy_queue.Wait(&self.gate_fence, value) }
                .context("async copy gate wait failed")?;
        }
        unsafe { self.copy_queue.Wait(direct_fence, direct_release_fence) }
            .context("async copy release wait failed")?;
        let copy_list: ID3D12CommandList = self.copy_list.cast()?;
        unsafe { self.copy_queue.ExecuteCommandLists(&[Some(copy_list)]) };
        let copy_fence = self.next_copy_fence;
        self.next_copy_fence += 1;
        unsafe { self.copy_queue.Signal(&self.copy_fence, copy_fence) }
            .context("async copy signal failed")?;

        let report = AsyncTransactionReport {
            revision: ASYNC_RESIDENT_REVISION,
            transaction_id,
            config: plan.layout.config,
            counts: plan.layout.counts,
            uploaded_sha256: plan.uploaded_sha256,
            direct_release_fence,
            copy_fence,
            gate_fence,
            payload_source: preparation.source,
            payload_preparation_ms: preparation.total_ms,
            generation_ms: preparation.generation_ms,
            schedule_ms: reservation.started_at.elapsed().as_secs_f64() * 1_000.0,
            pending_ms: 0.0,
        };
        self.pending = Some(PendingTransfer {
            next_cache: plan.layout.next_cache,
            active_slots: plan.layout.active_slots,
            uploaded_slots: plan.uploads.iter().map(|upload| upload.slot).collect(),
            report: report.clone(),
            started_at: reservation.started_at,
        });
        Ok(report)
    }

    pub unsafe fn poll_publication(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Option<Publication> {
        let pending = self.pending.as_ref()?;
        if unsafe { self.copy_fence.GetCompletedValue() } < pending.report.copy_fence {
            return None;
        }
        let mut pending = self.pending.take().expect("pending transfer disappeared");
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
        self.cache = pending.next_cache;
        pending.report.pending_ms = pending.started_at.elapsed().as_secs_f64() * 1_000.0;
        self.last_completed = Some(pending.report.clone());
        Some(Publication {
            config: pending.report.config,
            active_slots: pending.active_slots,
            report: pending.report,
        })
    }

    pub fn arm_gate(&mut self) -> Result<u64> {
        if self.reservation.is_some() || self.pending.is_some() || self.armed_gate.is_some() {
            bail!("copy gate or stream transaction is already active");
        }
        let value = self.next_gate_fence;
        self.next_gate_fence += 1;
        self.armed_gate = Some(value);
        Ok(value)
    }

    pub unsafe fn release_gate(&mut self) -> Result<u64> {
        let value = self.armed_gate.context("copy gate is not armed")?;
        unsafe { self.gate_fence.Signal(value) }.context("copy gate signal failed")?;
        self.armed_gate = None;
        Ok(value)
    }

    pub fn descriptor_heap(&self) -> &ID3D12DescriptorHeap {
        &self.descriptor_heap
    }

    pub fn has_pending(&self) -> bool {
        self.reservation.is_some() || self.pending.is_some()
    }

    pub fn has_armed_gate(&self) -> bool {
        self.armed_gate.is_some()
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        if let Some(value) = self.armed_gate.take() {
            unsafe { self.gate_fence.Signal(value) }
                .context("async shutdown gate signal failed")?;
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
            .context("async copy completion event failed")?;
        let wait = unsafe { WaitForSingleObject(self.copy_event, INFINITE) };
        if wait != WAIT_OBJECT_0 {
            bail!("async copy wait returned {wait:?}");
        }
        Ok(())
    }

    unsafe fn write_uploads(&self, uploads: &[crate::resident::RegionUpload]) -> Result<()> {
        let mut mapped = ptr::null_mut();
        unsafe {
            self.upload.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut mapped),
            )
        }
        .context("async upload arena map failed")?;
        for upload in uploads {
            let offset = upload.slot as usize * REGION_INSTANCE_BYTES;
            let bytes = as_bytes(&upload.records);
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    mapped.cast::<u8>().add(offset),
                    bytes.len(),
                )
            };
        }
        unsafe {
            self.upload.Unmap(
                0,
                Some(&D3D12_RANGE {
                    Begin: 0,
                    End: ASYNC_CACHE_CAPACITY * REGION_INSTANCE_BYTES,
                }),
            )
        };
        Ok(())
    }
}

impl Drop for AsyncTransfer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_idle();
            let _ = CloseHandle(self.copy_event);
        }
    }
}

fn ensure_transaction(actual: u64, requested: u64) -> Result<()> {
    if actual != requested {
        bail!("async reservation {actual} does not match transaction {requested}");
    }
    Ok(())
}
