use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::{ID3D12Device, ID3D12GraphicsCommandList};

use crate::address::GlobalRegionConfig;
use crate::async_resident::{AsyncTransactionReport, ObjectSourceNamespace};
use crate::load::LoadConfig;
use crate::rendering::terrain::TerrainProjection;

use super::transfer::{AsyncTransfer, Publication};

mod global;
mod payload;
mod status;

pub(in crate::rendering) use payload::ActivePayloadReadback;

pub struct AsyncResidentRenderer {
    transfer: AsyncTransfer,
    active_payload_readback: windows::Win32::Graphics::Direct3D12::ID3D12Resource,
    active_payload_allocation_bytes: u64,
    active_payload_probe_count: u64,
    active_payload_copy_count: u64,
    active_identity_readback: windows::Win32::Graphics::Direct3D12::ID3D12Resource,
    active_identity_allocation_bytes: u64,
    active_identity_probe_count: u64,
    active_identity_copy_count: u64,
    active_presentation_readback: windows::Win32::Graphics::Direct3D12::ID3D12Resource,
    active_presentation_allocation_bytes: u64,
    active_presentation_probe_count: u64,
    active_presentation_copy_count: u64,
    published: Option<PublishedSnapshot>,
    staged: Option<Publication>,
}

pub(in crate::rendering) struct PublishedSnapshot {
    pub config: LoadConfig,
    pub global_config: GlobalRegionConfig,
    pub object_source_namespace: ObjectSourceNamespace,
    pub object_stable_seed_namespace: ObjectSourceNamespace,
    pub object_page_checksums: Vec<[u8; 32]>,
    pub active_slots: Vec<u32>,
}

impl PublishedSnapshot {
    pub(in crate::rendering) fn projection(&self) -> Result<TerrainProjection> {
        TerrainProjection::for_objects(self.config)
    }
}

impl AsyncResidentRenderer {
    pub unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let transfer = unsafe { AsyncTransfer::new(device) }?;
        let (active_payload_readback, active_payload_allocation_bytes) =
            unsafe { payload::create_readback(device) }?;
        let (active_identity_readback, active_identity_allocation_bytes) =
            unsafe { payload::create_identity_readback(device) }?;
        let (active_presentation_readback, active_presentation_allocation_bytes) =
            unsafe { payload::create_presentation_readback(device) }?;
        Ok(Self {
            transfer,
            active_payload_readback,
            active_payload_allocation_bytes,
            active_payload_probe_count: 0,
            active_payload_copy_count: 0,
            active_identity_readback,
            active_identity_allocation_bytes,
            active_identity_probe_count: 0,
            active_identity_copy_count: 0,
            active_presentation_readback,
            active_presentation_allocation_bytes,
            active_presentation_probe_count: 0,
            active_presentation_copy_count: 0,
            published: None,
            staged: None,
        })
    }

    pub fn cancel_reservation(&mut self, transaction_id: u64) -> Result<()> {
        self.transfer.cancel_reservation(transaction_id)
    }

    pub(in crate::rendering) unsafe fn stage_frame(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
    ) -> bool {
        if self.staged.is_some() {
            return false;
        }
        let Some(publication) = (unsafe { self.transfer.poll_publication(command_list) }) else {
            return false;
        };
        self.staged = Some(publication);
        true
    }

    pub(in crate::rendering) fn commit_staged(&mut self) -> Option<AsyncTransactionReport> {
        let Publication {
            config,
            active_slots,
            report,
        } = self.staged.take()?;
        self.published = Some(PublishedSnapshot {
            config,
            global_config: report.global_config,
            object_source_namespace: report.object_source_namespace,
            object_stable_seed_namespace: report.object_stable_seed_namespace,
            object_page_checksums: report.object_page_checksums.clone(),
            active_slots,
        });
        Some(report)
    }

    pub(in crate::rendering) fn discard_staged(&mut self) -> Option<AsyncTransactionReport> {
        self.staged.take().map(|publication| publication.report)
    }

    pub(in crate::rendering) fn staged_report(&self) -> Option<&AsyncTransactionReport> {
        self.staged.as_ref().map(|publication| &publication.report)
    }

    pub(in crate::rendering) fn staged_active_slots(&self) -> Option<&[u32]> {
        self.staged
            .as_ref()
            .map(|publication| publication.active_slots.as_slice())
    }
}
