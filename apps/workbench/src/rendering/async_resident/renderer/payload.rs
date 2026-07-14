use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use windows::Win32::Graphics::Direct3D12::*;

use crate::async_resident::{AsyncTransactionReport, PayloadPreparation};
use crate::load::INSTANCES_PER_REGION;
use crate::rendering::resident::{create_buffer, read_values};
use crate::resident::{
    ACTIVE_REGION_CAPACITY, InstanceRecord, REGION_IDENTITY_BYTES, REGION_INSTANCE_BYTES,
    RegionUpload,
};

use super::AsyncResidentRenderer;

pub const ACTIVE_PAYLOAD_BYTES: u64 = (ACTIVE_REGION_CAPACITY * REGION_INSTANCE_BYTES) as u64;
pub const ACTIVE_IDENTITY_BYTES: u64 = (ACTIVE_REGION_CAPACITY * REGION_IDENTITY_BYTES) as u64;

pub(in crate::rendering) struct ActivePayloadReadback {
    pub records: Vec<Vec<InstanceRecord>>,
    pub local_ids: Vec<Vec<u32>>,
    pub expected_checksums: Option<Vec<[u8; 32]>>,
    pub readback_bytes: u64,
    pub allocation_bytes: u64,
    pub copy_count: u32,
    pub probe_count: u64,
    pub total_copy_count: u64,
    pub identity_readback_bytes: u64,
    pub identity_allocation_bytes: u64,
    pub identity_copy_count: u32,
    pub identity_probe_count: u64,
    pub identity_total_copy_count: u64,
}

pub(super) unsafe fn create_identity_readback(
    device: &ID3D12Device,
) -> Result<(ID3D12Resource, u64)> {
    let resource = unsafe {
        create_buffer(
            device,
            ACTIVE_IDENTITY_BYTES,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_FLAG_NONE,
        )
    }?;
    let allocation = unsafe { device.GetResourceAllocationInfo(0, &[resource.GetDesc()]) };
    Ok((resource, allocation.SizeInBytes))
}

pub(super) unsafe fn create_readback(device: &ID3D12Device) -> Result<(ID3D12Resource, u64)> {
    let resource = unsafe {
        create_buffer(
            device,
            ACTIVE_PAYLOAD_BYTES,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_FLAG_NONE,
        )
    }?;
    let allocation = unsafe { device.GetResourceAllocationInfo(0, &[resource.GetDesc()]) };
    Ok((resource, allocation.SizeInBytes))
}

impl AsyncResidentRenderer {
    pub(in crate::rendering) unsafe fn submit_canonical_cooked(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        preparation_ms: f64,
        object_page_checksums: Vec<[u8; 32]>,
        direct_sync: (&ID3D12CommandQueue, &ID3D12Fence, u64),
    ) -> Result<AsyncTransactionReport> {
        let (direct_queue, direct_fence, direct_release_fence) = direct_sync;
        let mut report = unsafe {
            self.transfer.submit(
                transaction_id,
                uploads,
                PayloadPreparation::cooked(preparation_ms),
                direct_queue,
                direct_fence,
                direct_release_fence,
            )
        }?;
        self.transfer
            .bind_object_page_checksums(transaction_id, object_page_checksums.clone())?;
        report.object_page_checksums = Some(object_page_checksums);
        Ok(report)
    }

    pub(in crate::rendering) unsafe fn record_active_payload_readback(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> Result<()> {
        let active_slots = self
            .published
            .as_ref()
            .context("active payload readback has no published snapshot")?
            .active_slots
            .clone();
        ensure!(
            active_slots.len() == ACTIVE_REGION_CAPACITY,
            "composition payload readback requires the complete active window"
        );
        unsafe {
            self.transfer.record_active_pages(
                command_list,
                &active_slots,
                &self.active_payload_readback,
            )
        }?;
        unsafe {
            self.transfer.record_active_identities(
                command_list,
                &active_slots,
                &self.active_identity_readback,
            )
        }?;
        self.active_payload_probe_count += 1;
        self.active_payload_copy_count += active_slots.len() as u64;
        self.active_identity_probe_count += 1;
        self.active_identity_copy_count += active_slots.len() as u64;
        Ok(())
    }

    pub(in crate::rendering) unsafe fn read_active_payload(&self) -> Result<ActivePayloadReadback> {
        let snapshot = self
            .published
            .as_ref()
            .context("active payload readback has no published snapshot")?;
        let page_count = snapshot.active_slots.len();
        ensure!(
            page_count == ACTIVE_REGION_CAPACITY,
            "active payload readback page count is not canonical"
        );
        if let Some(expected) = &snapshot.object_page_checksums {
            ensure!(
                expected.len() == page_count,
                "published object checksum count differs from active pages"
            );
        }
        let flat = unsafe {
            read_values::<InstanceRecord>(
                &self.active_payload_readback,
                page_count * INSTANCES_PER_REGION as usize,
            )
        }?;
        let records = flat
            .chunks_exact(INSTANCES_PER_REGION as usize)
            .map(<[InstanceRecord]>::to_vec)
            .collect();
        let flat_local_ids = unsafe {
            read_values::<u32>(
                &self.active_identity_readback,
                page_count * INSTANCES_PER_REGION as usize,
            )
        }?;
        let local_ids = flat_local_ids
            .chunks_exact(INSTANCES_PER_REGION as usize)
            .map(<[u32]>::to_vec)
            .collect::<Vec<_>>();
        for page in &local_ids {
            let mut seen = [false; INSTANCES_PER_REGION as usize];
            for local_id in page {
                ensure!(
                    (*local_id as usize) < seen.len()
                        && !std::mem::replace(&mut seen[*local_id as usize], true),
                    "active object identity page is not a canonical permutation"
                );
            }
        }
        Ok(ActivePayloadReadback {
            records,
            local_ids,
            expected_checksums: snapshot.object_page_checksums.clone(),
            readback_bytes: page_count as u64 * REGION_INSTANCE_BYTES as u64,
            allocation_bytes: self.active_payload_allocation_bytes,
            copy_count: page_count as u32,
            probe_count: self.active_payload_probe_count,
            total_copy_count: self.active_payload_copy_count,
            identity_readback_bytes: page_count as u64 * REGION_IDENTITY_BYTES as u64,
            identity_allocation_bytes: self.active_identity_allocation_bytes,
            identity_copy_count: page_count as u32,
            identity_probe_count: self.active_identity_probe_count,
            identity_total_copy_count: self.active_identity_copy_count,
        })
    }

    pub(in crate::rendering) fn payload_readback_status(&self) -> Value {
        json!({
            "resourceCount": 1,
            "capacityPages": ACTIVE_REGION_CAPACITY,
            "capacityBytes": ACTIVE_PAYLOAD_BYTES,
            "allocationBytes": self.active_payload_allocation_bytes,
            "probeCount": self.active_payload_probe_count,
            "copyCount": self.active_payload_copy_count,
            "identity": {
                "resourceCount": 1,
                "capacityPages": ACTIVE_REGION_CAPACITY,
                "capacityBytes": ACTIVE_IDENTITY_BYTES,
                "allocationBytes": self.active_identity_allocation_bytes,
                "probeCount": self.active_identity_probe_count,
                "copyCount": self.active_identity_copy_count,
            },
        })
    }
}
