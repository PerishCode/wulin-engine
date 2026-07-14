use std::ptr;

use anyhow::{Context, Result, bail};
use windows::Win32::Graphics::Direct3D12::*;
use windows::core::Interface;

use crate::async_resident::{
    ASYNC_CACHE_CAPACITY, ASYNC_RESIDENT_REVISION, AsyncTransactionReport,
};
use crate::rendering::resident::transition;
use crate::resident::{REGION_IDENTITY_BYTES, REGION_INSTANCE_BYTES, RegionUpload, as_bytes};

use super::{AsyncTransfer, PendingTransfer, ensure_transaction};

impl AsyncTransfer {
    pub unsafe fn submit(
        &mut self,
        transaction_id: u64,
        uploads: Vec<RegionUpload>,
        payload_preparation_ms: f64,
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
        let identity_copy_slots = plan
            .uploads
            .iter()
            .map(|upload| upload.slot)
            .collect::<Vec<_>>();
        unsafe { self.write_uploads(&plan.uploads, &identity_copy_slots) }?;

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
        for slot in &identity_copy_slots {
            let index = *slot as usize;
            if self.identity_shader_slots[index] {
                unsafe {
                    transition(
                        &self.release_list,
                        &self.identities[index],
                        D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                        D3D12_RESOURCE_STATE_COPY_DEST,
                    )
                };
                self.identity_shader_slots[index] = false;
            }
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
        for slot in &identity_copy_slots {
            let upload_offset = u64::from(*slot) * REGION_IDENTITY_BYTES as u64;
            unsafe {
                self.copy_list.CopyBufferRegion(
                    &self.identities[*slot as usize],
                    0,
                    &self.identity_upload,
                    upload_offset,
                    REGION_IDENTITY_BYTES as u64,
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
            global_config: plan.layout.global_config,
            object_source_namespace: plan.layout.object_source_namespace,
            object_stable_seed_namespace: plan.layout.object_stable_seed_namespace,
            object_page_checksums: Vec::new(),
            counts: plan.layout.counts,
            uploaded_sha256: plan.uploaded_sha256,
            identity_copy_count: identity_copy_slots.len(),
            identity_copy_bytes: identity_copy_slots.len() * REGION_IDENTITY_BYTES,
            direct_release_fence,
            copy_fence,
            gate_fence,
            payload_preparation_ms,
            schedule_ms: reservation.started_at.elapsed().as_secs_f64() * 1_000.0,
            pending_ms: 0.0,
        };
        self.pending = Some(PendingTransfer {
            next_cache: plan.layout.next_cache,
            active_slots: plan.layout.active_slots,
            uploaded_slots: plan.uploads.iter().map(|upload| upload.slot).collect(),
            identity_transition_slots: plan
                .uploads
                .iter()
                .map(|upload| upload.slot)
                .filter(|slot| !self.identity_shader_slots[*slot as usize])
                .collect(),
            report: report.clone(),
            started_at: reservation.started_at,
        });
        Ok(report)
    }

    unsafe fn write_uploads(
        &self,
        uploads: &[RegionUpload],
        identity_copy_slots: &[u32],
    ) -> Result<()> {
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
        if identity_copy_slots.is_empty() {
            return Ok(());
        }
        let mut identity_mapped = ptr::null_mut();
        unsafe {
            self.identity_upload.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut identity_mapped),
            )
        }
        .context("async identity upload arena map failed")?;
        for upload in uploads {
            if !identity_copy_slots.contains(&upload.slot) {
                continue;
            }
            let offset = upload.slot as usize * REGION_IDENTITY_BYTES;
            let destination = unsafe { identity_mapped.cast::<u8>().add(offset) };
            let bytes = as_bytes(&upload.local_ids);
            unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len()) };
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
        Ok(())
    }
}
