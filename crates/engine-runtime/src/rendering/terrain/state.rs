use anyhow::{Result, ensure};
use windows::Win32::Graphics::Direct3D12::ID3D12DescriptorHeap;

use crate::load::LoadConfig;
use crate::terrain::{GlobalTerrainConfig, TerrainAssignment, TerrainSourceNamespace};
use crate::terrain_query::{TerrainHeight, TerrainQueryPosition, query_published_height};

use super::{TerrainProjection, TerrainRenderer};

impl TerrainRenderer {
    pub fn enable(&mut self) -> Result<()> {
        ensure!(
            self.published.is_some(),
            "terrain requires a published snapshot"
        );
        Ok(())
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.published.as_ref().map(|value| value.config)
    }

    pub(in crate::rendering) fn global_config(&self) -> Option<GlobalTerrainConfig> {
        self.published.as_ref().map(|value| value.global_config)
    }

    pub(in crate::rendering) fn source_namespace(&self) -> Option<TerrainSourceNamespace> {
        self.published
            .as_ref()
            .map(|value| value.report.source_namespace)
    }

    pub(in crate::rendering) fn projection(&self) -> Result<TerrainProjection> {
        let published = self
            .published
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("terrain projection requires a published snapshot"))?;
        TerrainProjection::for_terrain(published.config)
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

    pub(in crate::rendering) fn published_generation(&self) -> Option<u64> {
        self.published.as_ref().map(|value| value.generation)
    }

    pub(in crate::rendering) fn query_height(
        &self,
        position: TerrainQueryPosition,
    ) -> Result<TerrainHeight> {
        let published = self
            .published
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("terrain query requires a published snapshot"))?;
        query_published_height(
            published.global_config,
            &published.active,
            &published.tiles,
            position,
        )
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
