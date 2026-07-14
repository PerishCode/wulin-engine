use std::collections::BTreeSet;
use std::ptr;
use std::time::Instant;

use anyhow::{Context, Result, bail, ensure};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::address::GlobalRegionConfig;
use crate::async_resident::{
    ASYNC_CACHE_CAPACITY, ASYNC_RESIDENT_REVISION, AsyncLayoutPlan, AsyncRegionCache,
    AsyncReservationReport, AsyncTransactionReport, ObjectSourceNamespace, PayloadPreparation,
};
use crate::load::LoadConfig;
use crate::resident::{REGION_IDENTITY_BYTES, REGION_INSTANCE_BYTES, RegionUpload};

use super::super::resident::{create_buffer, transition};
use super::resources::create_descriptor_heap;

mod lifecycle;
mod payload;
mod status;
mod submit;

pub struct AsyncTransfer {
    regions: Vec<ID3D12Resource>,
    identities: Vec<ID3D12Resource>,
    descriptor_heap: ID3D12DescriptorHeap,
    region_allocation_bytes: u64,
    identity_allocation_bytes: u64,
    upload: ID3D12Resource,
    identity_upload: ID3D12Resource,
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
    identity_shader_slots: [bool; ASYNC_CACHE_CAPACITY],
    identity_kinds: [IdentityKind; ASYNC_CACHE_CAPACITY],
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
    identity_transition_slots: Vec<u32>,
    identity_updates: Vec<(u32, IdentityKind)>,
    report: AsyncTransactionReport,
    started_at: Instant,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum IdentityKind {
    Ordinal,
    Explicit,
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
        let mut identities = Vec::with_capacity(ASYNC_CACHE_CAPACITY);
        for _ in 0..ASYNC_CACHE_CAPACITY {
            identities.push(unsafe {
                create_buffer(
                    device,
                    REGION_IDENTITY_BYTES as u64,
                    D3D12_HEAP_TYPE_DEFAULT,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_FLAG_NONE,
                )
            }?);
        }
        let descriptor_heap = unsafe { create_descriptor_heap(device, &regions, &identities) }?;
        let region_allocation_bytes =
            unsafe { device.GetResourceAllocationInfo(0, &[regions[0].GetDesc()]) }.SizeInBytes
                * ASYNC_CACHE_CAPACITY as u64;
        let identity_allocation_bytes =
            unsafe { device.GetResourceAllocationInfo(0, &[identities[0].GetDesc()]) }.SizeInBytes
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
        let identity_upload = unsafe {
            create_buffer(
                device,
                (ASYNC_CACHE_CAPACITY * REGION_IDENTITY_BYTES) as u64,
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

        let mut transfer = Self {
            regions,
            identities,
            descriptor_heap,
            region_allocation_bytes,
            identity_allocation_bytes,
            upload,
            identity_upload,
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
            identity_shader_slots: [false; ASYNC_CACHE_CAPACITY],
            identity_kinds: [IdentityKind::Ordinal; ASYNC_CACHE_CAPACITY],
            pending: None,
            last_completed: None,
            next_transaction_id: 1,
        };
        unsafe { transfer.initialize_identity_pages() }?;
        Ok(transfer)
    }

    unsafe fn initialize_identity_pages(&mut self) -> Result<()> {
        let mut mapped = ptr::null_mut();
        unsafe {
            self.identity_upload.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut mapped),
            )
        }
        .context("async identity upload arena map failed")?;
        for slot in 0..ASYNC_CACHE_CAPACITY {
            let destination = unsafe {
                mapped
                    .cast::<u8>()
                    .add(slot * REGION_IDENTITY_BYTES)
                    .cast::<u32>()
            };
            for local_id in 0..crate::load::INSTANCES_PER_REGION {
                unsafe { destination.add(local_id as usize).write(local_id) };
            }
        }
        unsafe {
            self.identity_upload.Unmap(
                0,
                Some(&D3D12_RANGE {
                    Begin: 0,
                    End: ASYNC_CACHE_CAPACITY * REGION_IDENTITY_BYTES,
                }),
            )
        };

        unsafe { self.copy_allocator.Reset() }
            .context("async identity initialization allocator reset failed")?;
        unsafe { self.copy_list.Reset(&self.copy_allocator, None) }
            .context("async identity initialization list reset failed")?;
        for slot in 0..ASYNC_CACHE_CAPACITY {
            unsafe {
                self.copy_list.CopyBufferRegion(
                    &self.identities[slot],
                    0,
                    &self.identity_upload,
                    (slot * REGION_IDENTITY_BYTES) as u64,
                    REGION_IDENTITY_BYTES as u64,
                )
            };
        }
        unsafe { self.copy_list.Close() }
            .context("async identity initialization list close failed")?;
        let list: ID3D12CommandList = self.copy_list.cast()?;
        unsafe { self.copy_queue.ExecuteCommandLists(&[Some(list)]) };
        let fence = self.next_copy_fence;
        self.next_copy_fence += 1;
        unsafe { self.copy_queue.Signal(&self.copy_fence, fence) }
            .context("async identity initialization signal failed")?;
        unsafe { self.copy_fence.SetEventOnCompletion(fence, self.copy_event) }
            .context("async identity initialization event failed")?;
        let wait = unsafe { WaitForSingleObject(self.copy_event, INFINITE) };
        ensure!(
            wait == WAIT_OBJECT_0,
            "async identity initialization wait returned {wait:?}"
        );
        Ok(())
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
                local_ids: None,
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
        self.ensure_available()?;
        let layout = self.cache.plan_layout(config, protected_slots)?;
        self.reserve_layout(layout)
    }

    pub fn reserve_composition(
        &mut self,
        config: LoadConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncReservationReport> {
        self.ensure_available()?;
        let layout = self
            .cache
            .plan_composition_layout(config, protected_slots)?;
        self.reserve_layout(layout)
    }

    pub fn reserve_global_composition(
        &mut self,
        config: GlobalRegionConfig,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncReservationReport> {
        self.ensure_available()?;
        let layout = self
            .cache
            .plan_global_composition_layout(config, protected_slots)?;
        self.reserve_layout(layout)
    }

    pub fn reserve_canonical_global_composition(
        &mut self,
        config: GlobalRegionConfig,
        source_namespace: ObjectSourceNamespace,
        stable_seed_namespace: ObjectSourceNamespace,
        protected_slots: &BTreeSet<u32>,
    ) -> Result<AsyncReservationReport> {
        self.ensure_available()?;
        let layout = self.cache.plan_canonical_layout(
            config,
            source_namespace,
            stable_seed_namespace,
            protected_slots,
        )?;
        self.reserve_layout(layout)
    }

    fn reserve_layout(&mut self, layout: AsyncLayoutPlan) -> Result<AsyncReservationReport> {
        debug_assert!(self.reservation.is_none() && self.pending.is_none());
        let transaction_id = self.next_transaction_id;
        self.next_transaction_id += 1;
        let report = AsyncReservationReport {
            revision: ASYNC_RESIDENT_REVISION,
            transaction_id,
            config: layout.config,
            global_config: layout.global_config,
            object_source_namespace: layout.object_source_namespace,
            object_stable_seed_namespace: layout.object_stable_seed_namespace,
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

    fn ensure_available(&self) -> Result<()> {
        if self.reservation.is_some() || self.pending.is_some() {
            bail!("stream_busy");
        }
        Ok(())
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

    pub(super) fn bind_object_page_checksums(
        &mut self,
        transaction_id: u64,
        checksums: Vec<[u8; 32]>,
    ) -> Result<()> {
        let pending = self
            .pending
            .as_mut()
            .context("async transfer has no pending object copy")?;
        ensure_transaction(pending.report.transaction_id, transaction_id)?;
        anyhow::ensure!(
            checksums.len() == pending.active_slots.len(),
            "object page checksum count does not match the active mapping"
        );
        pending.report.object_page_checksums = Some(checksums);
        Ok(())
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
        for slot in &pending.identity_transition_slots {
            let index = *slot as usize;
            unsafe {
                transition(
                    command_list,
                    &self.identities[index],
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                )
            };
            self.identity_shader_slots[index] = true;
        }
        for (slot, kind) in &pending.identity_updates {
            self.identity_kinds[*slot as usize] = *kind;
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
}

fn ensure_transaction(actual: u64, requested: u64) -> Result<()> {
    if actual != requested {
        bail!("async reservation {actual} does not match transaction {requested}");
    }
    Ok(())
}
