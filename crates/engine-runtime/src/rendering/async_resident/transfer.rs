use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::System::Threading::CreateEventW;

use crate::address::GlobalRegionConfig;
use crate::async_resident::{
    ASYNC_CACHE_CAPACITY, ASYNC_RESIDENT_REVISION, AsyncLayoutPlan, AsyncRegionCache,
    AsyncReservationReport, AsyncTransactionReport, ObjectSourceNamespace,
};
use crate::load::LoadConfig;
use crate::region::RegionCoord;
use crate::resident::{REGION_IDENTITY_BYTES, REGION_INSTANCE_BYTES, REGION_PRESENTATION_BYTES};

use super::super::resident::{create_buffer, transition};
use descriptors::create_descriptor_heap;

mod descriptors;
mod lifecycle;
mod payload;
mod submit;

pub struct AsyncTransfer {
    regions: Vec<ID3D12Resource>,
    identities: Vec<ID3D12Resource>,
    presentations: Vec<ID3D12Resource>,
    descriptor_heap: ID3D12DescriptorHeap,
    upload: ID3D12Resource,
    identity_upload: ID3D12Resource,
    presentation_upload: ID3D12Resource,
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
    cpu_pages: Vec<Option<Arc<CpuObjectPage>>>,
    reservation: Option<ReservedTransfer>,
    shader_slots: [bool; ASYNC_CACHE_CAPACITY],
    identity_shader_slots: [bool; ASYNC_CACHE_CAPACITY],
    presentation_shader_slots: [bool; ASYNC_CACHE_CAPACITY],
    pending: Option<PendingTransfer>,
    next_transaction_id: u64,
}

struct ReservedTransfer {
    transaction_id: u64,
    layout: AsyncLayoutPlan,
    started_at: Instant,
}

struct PendingTransfer {
    next_cache: AsyncRegionCache,
    next_cpu_pages: Vec<Option<Arc<CpuObjectPage>>>,
    active_slots: Vec<u32>,
    uploaded_slots: Vec<u32>,
    identity_transition_slots: Vec<u32>,
    presentation_transition_slots: Vec<u32>,
    report: AsyncTransactionReport,
    started_at: Instant,
}

pub struct Publication {
    pub config: LoadConfig,
    pub active_slots: Vec<u32>,
    pub(super) active_cpu_pages: Vec<Arc<CpuObjectPage>>,
    pub report: AsyncTransactionReport,
}

pub(super) struct CpuObjectPage {
    pub global_region: RegionCoord,
    pub records: Vec<crate::resident::InstanceRecord>,
    pub local_ids: Vec<u32>,
    pub presentations: Vec<crate::resident::PresentationRecord>,
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
        let mut presentations = Vec::with_capacity(ASYNC_CACHE_CAPACITY);
        for _ in 0..ASYNC_CACHE_CAPACITY {
            presentations.push(unsafe {
                create_buffer(
                    device,
                    REGION_PRESENTATION_BYTES as u64,
                    D3D12_HEAP_TYPE_DEFAULT,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_FLAG_NONE,
                )
            }?);
        }
        let descriptor_heap =
            unsafe { create_descriptor_heap(device, &regions, &identities, &presentations) }?;
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
        let presentation_upload = unsafe {
            create_buffer(
                device,
                (ASYNC_CACHE_CAPACITY * REGION_PRESENTATION_BYTES) as u64,
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
            identities,
            presentations,
            descriptor_heap,
            upload,
            identity_upload,
            presentation_upload,
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
            cpu_pages: vec![None; ASYNC_CACHE_CAPACITY],
            reservation: None,
            shader_slots: [false; ASYNC_CACHE_CAPACITY],
            identity_shader_slots: [false; ASYNC_CACHE_CAPACITY],
            presentation_shader_slots: [false; ASYNC_CACHE_CAPACITY],
            pending: None,
            next_transaction_id: 1,
        })
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
        pending.report.object_page_checksums = checksums;
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
        for slot in &pending.presentation_transition_slots {
            let index = *slot as usize;
            unsafe {
                transition(
                    command_list,
                    &self.presentations[index],
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                )
            };
            self.presentation_shader_slots[index] = true;
        }
        self.cache = pending.next_cache;
        self.cpu_pages = pending.next_cpu_pages;
        pending.report.pending_ms = pending.started_at.elapsed().as_secs_f64() * 1_000.0;
        let active_cpu_pages = pending
            .active_slots
            .iter()
            .map(|slot| {
                self.cpu_pages[*slot as usize]
                    .as_ref()
                    .expect("published object slot has no CPU page")
                    .clone()
            })
            .collect();
        Some(Publication {
            config: pending.report.config,
            active_slots: pending.active_slots,
            active_cpu_pages,
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
}

fn ensure_transaction(actual: u64, requested: u64) -> Result<()> {
    if actual != requested {
        bail!("async reservation {actual} does not match transaction {requested}");
    }
    Ok(())
}
