use anyhow::Result;
use serde_json::Value;

use crate::rendering::async_resident::PublishedSnapshot;
use crate::scene::SceneState;

use super::renderer::SkeletalSceneRenderer;
use super::surface::{SurfaceProbe, SurfaceProbeContext, SurfaceSettings};

impl SkeletalSceneRenderer {
    pub fn configure_surface(&mut self, settings: SurfaceSettings) -> Result<()> {
        self.surface.configure(settings)
    }

    pub fn enable_surface(&mut self) {
        self.enabled = true;
        self.surface.enable();
    }

    pub fn disable_surface(&mut self) {
        self.surface.disable();
    }

    pub fn surface_status_json(&self) -> Value {
        self.surface.status_json()
    }

    pub fn surface_is_enabled(&self) -> bool {
        self.surface.is_enabled()
    }

    pub unsafe fn read_surface_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
        background_color: [f32; 4],
    ) -> Result<SurfaceProbe> {
        let skeletal = unsafe { self.read_probe(snapshot, scene) }?;
        unsafe {
            self.surface.read_probe(SurfaceProbeContext {
                skeletal,
                animation_catalog: &self.animation_catalog,
                skeletal_settings: self.settings,
                config: snapshot.config,
                background_color,
                timestamp_readback: &self.resources.timestamp_readback,
                timestamp_frequency: self.timestamp_frequency,
            })
        }
    }
}
