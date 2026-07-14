use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::ID3D12DescriptorHeap;

use crate::load::LoadConfig;

use super::{AsyncResidentRenderer, PublishedSnapshot};

impl AsyncResidentRenderer {
    pub fn arm_gate(&mut self) -> Result<u64> {
        self.transfer.arm_gate()
    }

    pub unsafe fn release_gate(&mut self) -> Result<u64> {
        unsafe { self.transfer.release_gate() }
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.published.as_ref().map(|snapshot| snapshot.config)
    }

    pub(in crate::rendering) fn snapshot(&self) -> Option<&PublishedSnapshot> {
        self.published.as_ref()
    }

    pub(in crate::rendering) fn descriptor_heap(&self) -> &ID3D12DescriptorHeap {
        self.transfer.descriptor_heap()
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.transfer.wait_idle() }
    }

    pub(super) fn protected_slots(&self) -> std::collections::BTreeSet<u32> {
        self.published
            .iter()
            .flat_map(|snapshot| snapshot.active_slots.iter().copied())
            .chain(
                self.staged
                    .iter()
                    .flat_map(|publication| publication.active_slots.iter().copied()),
            )
            .collect()
    }
}
