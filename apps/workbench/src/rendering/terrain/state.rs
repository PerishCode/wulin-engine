use anyhow::{Result, ensure};
use serde_json::{Value, json};
use windows::Win32::Graphics::Direct3D12::ID3D12DescriptorHeap;

use crate::load::LoadConfig;
use crate::terrain::TerrainAssignment;

use super::{PATCH_GROUP_COUNT, TERRAIN_REVISION, TerrainLodSettings, TerrainRenderer, lod};

impl TerrainRenderer {
    pub fn enable(&mut self) -> Result<()> {
        ensure!(
            self.published.is_some(),
            "terrain requires a published snapshot"
        );
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.published.as_ref().map(|value| value.config)
    }

    pub fn status_json(&self) -> Value {
        json!({
            "revision": TERRAIN_REVISION,
            "enabled": self.enabled,
            "published": self.published.as_ref().map(|value| json!({
                "config": value.config,
                "generation": value.generation,
                "active": value.active,
                "transaction": value.report,
            })),
            "transfer": self.transfer.status_json(),
            "lod": {
                "revision": lod::LOD_REVISION,
                "settings": self.lod_settings,
                "submission": {
                    "dispatchCount": u32::from(self.lod_settings.enabled),
                    "dispatchGroups": [PATCH_GROUP_COUNT, 2, 1],
                },
            },
            "submission": {
                "meshDispatchCount": 1,
                "meshDispatchGroups": [PATCH_GROUP_COUNT, 1, 1],
                "seamDispatchCount": 1,
                "seamDispatchGroups": [25, 2, 1],
            },
        })
    }

    pub fn configure_lod(
        &mut self,
        near_patch_radius: u32,
        middle_patch_radius: u32,
        forced_lod: Option<u32>,
    ) -> Result<()> {
        self.lod_settings =
            self.lod_settings
                .configured(near_patch_radius, middle_patch_radius, forced_lod)?;
        Ok(())
    }

    pub fn enable_lod(&mut self) {
        self.lod_settings.enabled = true;
    }

    pub fn disable_lod(&mut self) {
        self.lod_settings.enabled = false;
    }

    pub fn lod_settings(&self) -> TerrainLodSettings {
        self.lod_settings
    }

    pub(in crate::rendering) fn descriptor_heap(&self) -> &ID3D12DescriptorHeap {
        self.transfer.descriptor_heap()
    }

    pub(in crate::rendering) fn active_assignments(&self) -> Option<&[TerrainAssignment]> {
        self.published.as_ref().map(|value| value.active.as_slice())
    }

    pub(in crate::rendering) fn published_tiles(&self) -> Option<&[terrain_format::TerrainTile]> {
        self.published.as_ref().map(|value| value.tiles.as_slice())
    }

    pub fn arm_copy_gate(&mut self) -> Result<u64> {
        self.transfer.arm_gate()
    }

    pub unsafe fn release_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.transfer.release_gate() }
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.transfer.wait_idle() }
    }
}
