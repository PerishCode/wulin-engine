use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use windows::Win32::Graphics::Direct3D12::*;

use crate::rendering::resident::transition;
use crate::resident::{ACTIVE_REGION_CAPACITY, REGION_INSTANCE_BYTES};

use super::AsyncTransfer;

impl AsyncTransfer {
    pub(in crate::rendering::async_resident) unsafe fn record_active_pages(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        active_slots: &[u32],
        destination: &ID3D12Resource,
    ) -> Result<()> {
        ensure!(
            active_slots.len() <= ACTIVE_REGION_CAPACITY,
            "active object page readback exceeds fixed capacity"
        );
        let unique = active_slots.iter().copied().collect::<BTreeSet<_>>();
        ensure!(
            unique.len() == active_slots.len(),
            "active object page readback contains duplicate slots"
        );
        for (active_index, slot) in active_slots.iter().copied().enumerate() {
            let index = slot as usize;
            ensure!(
                index < self.regions.len() && self.shader_slots[index],
                "active object page slot {slot} is not shader-readable"
            );
            unsafe {
                transition(
                    command_list,
                    &self.regions[index],
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    destination,
                    (active_index * REGION_INSTANCE_BYTES) as u64,
                    &self.regions[index],
                    0,
                    REGION_INSTANCE_BYTES as u64,
                );
                transition(
                    command_list,
                    &self.regions[index],
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                );
            }
        }
        Ok(())
    }
}
