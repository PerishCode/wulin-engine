use anyhow::{Result, bail};

use crate::async_resident::AsyncTransactionReport;
use crate::load::LoadConfig;

use super::{Renderer, ensure_no_composition_pair};
use crate::rendering::meshlet_scene::{SkeletalSettings, SurfaceSettings};

impl Renderer {
    pub fn calibration_mode_active(&self) -> bool {
        !self.composition_enabled()
            && !self.terrain_mode_enabled()
            && !self.async_resident_enabled()
            && self.resident_config().is_none()
            && self.load_renderer.config().is_none()
    }

    pub fn configure_load(&mut self, config: LoadConfig) -> Result<()> {
        ensure_no_composition_pair(&self.composition)?;
        self.disable_composition();
        self.terrain_renderer.disable();
        self.meshlet_scene_renderer.disable();
        self.skeletal_scene_renderer.disable();
        self.async_resident_renderer.disable()?;
        self.resident_renderer.disable();
        self.load_renderer.configure(config);
        Ok(())
    }

    pub unsafe fn stream_resident(&mut self, config: LoadConfig) -> Result<()> {
        ensure_no_composition_pair(&self.composition)?;
        self.disable_composition();
        self.terrain_renderer.disable();
        self.meshlet_scene_renderer.disable();
        self.skeletal_scene_renderer.disable();
        self.async_resident_renderer.disable()?;
        self.load_renderer.disable();
        unsafe { self.resident_renderer.prepare_stream(config) }
    }

    pub fn disable_load(&mut self) -> Result<()> {
        ensure_no_composition_pair(&self.composition)?;
        self.disable_composition();
        self.terrain_renderer.disable();
        self.meshlet_scene_renderer.disable();
        self.skeletal_scene_renderer.disable();
        self.async_resident_renderer.disable()?;
        self.load_renderer.disable();
        self.resident_renderer.disable();
        Ok(())
    }

    pub fn load_config(&self) -> Option<LoadConfig> {
        self.terrain_renderer.config().or_else(|| {
            self.async_resident_renderer
                .config()
                .or_else(|| self.resident_renderer.config())
                .or_else(|| self.load_renderer.config())
        })
    }

    pub unsafe fn stream_async_resident(
        &mut self,
        config: LoadConfig,
    ) -> Result<AsyncTransactionReport> {
        ensure_no_composition_pair(&self.composition)?;
        self.disable_composition();
        self.load_renderer.disable();
        self.resident_renderer.disable();
        let release_fence = self.next_fence_value;
        self.next_fence_value += 1;
        unsafe {
            self.async_resident_renderer
                .schedule(config, &self.queue, &self.fence, release_fence)
        }
    }

    pub fn async_resident_status(&self) -> serde_json::Value {
        self.async_resident_renderer.status_json()
    }

    pub fn async_resident_config(&self) -> Option<LoadConfig> {
        self.async_resident_renderer.config()
    }

    pub fn async_resident_enabled(&self) -> bool {
        self.async_resident_renderer.is_enabled()
    }

    pub fn configure_meshlet_scene(
        &mut self,
        archetype_mask: u32,
        forced_lod: Option<u32>,
    ) -> Result<()> {
        self.meshlet_scene_renderer
            .configure(archetype_mask, forced_lod)
    }

    pub fn enable_meshlet_scene(&mut self) -> Result<()> {
        if self.async_resident_renderer.config().is_none() {
            bail!("meshlet scene requires a published async resident snapshot");
        }
        if self
            .async_resident_renderer
            .object_source_namespace()
            .is_some()
        {
            bail!("canonical generated objects require atomic composition mode");
        }
        self.disable_composition();
        self.skeletal_scene_renderer.disable();
        self.terrain_renderer.disable();
        self.meshlet_scene_renderer.enable();
        Ok(())
    }

    pub fn disable_meshlet_scene(&mut self) {
        self.meshlet_scene_renderer.disable();
    }

    pub fn meshlet_scene_status(&self) -> serde_json::Value {
        self.meshlet_scene_renderer.status_json()
    }

    pub fn configure_skeletal_scene(
        &mut self,
        animated_percent: u32,
        bone_count: u32,
        phase_count: u32,
        time_tick: u32,
        unique_poses: bool,
        forced_lod: Option<u32>,
    ) -> Result<()> {
        self.skeletal_scene_renderer.configure(SkeletalSettings {
            animated_percent,
            bone_count,
            phase_count,
            time_tick,
            unique_poses,
            forced_lod,
        })
    }

    pub fn enable_skeletal_scene(&mut self) -> Result<()> {
        if self.async_resident_renderer.config().is_none() {
            bail!("skeletal scene requires a published async resident snapshot");
        }
        if self
            .async_resident_renderer
            .object_source_namespace()
            .is_some()
        {
            bail!("canonical generated objects require atomic composition mode");
        }
        self.disable_composition();
        self.meshlet_scene_renderer.disable();
        self.terrain_renderer.disable();
        self.skeletal_scene_renderer.enable();
        Ok(())
    }

    pub fn disable_skeletal_scene(&mut self) {
        self.skeletal_scene_renderer.disable();
    }

    pub fn skeletal_scene_status(&self) -> serde_json::Value {
        self.skeletal_scene_renderer.status_json()
    }

    pub fn configure_surface(&mut self, material_count: u32, mip_level: u32) -> Result<()> {
        self.skeletal_scene_renderer
            .configure_surface(SurfaceSettings {
                material_count,
                mip_level,
            })
    }

    pub fn enable_surface(&mut self) -> Result<()> {
        if self.async_resident_renderer.config().is_none() {
            bail!("surface resolve requires a published async resident snapshot");
        }
        if self
            .async_resident_renderer
            .object_source_namespace()
            .is_some()
        {
            bail!("canonical generated objects require atomic composition mode");
        }
        self.disable_composition();
        self.meshlet_scene_renderer.disable();
        self.terrain_renderer.disable();
        self.skeletal_scene_renderer.enable_surface();
        Ok(())
    }

    pub fn disable_surface(&mut self) {
        self.skeletal_scene_renderer.disable_surface();
    }

    pub fn surface_status(&self) -> serde_json::Value {
        self.skeletal_scene_renderer.surface_status_json()
    }

    pub fn enable_surface_occlusion(&mut self) {
        self.skeletal_scene_renderer.enable_surface_occlusion();
    }

    pub fn disable_surface_occlusion(&mut self) {
        self.skeletal_scene_renderer.disable_surface_occlusion();
    }

    pub fn reset_surface_occlusion(&mut self) {
        self.skeletal_scene_renderer.reset_surface_occlusion();
    }

    pub fn arm_async_copy_gate(&mut self) -> Result<u64> {
        self.async_resident_renderer.arm_gate()
    }

    pub unsafe fn release_async_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.async_resident_renderer.release_gate() }
    }

    pub fn resident_config(&self) -> Option<LoadConfig> {
        self.resident_renderer.config()
    }
}
