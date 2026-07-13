use animation_catalog::Catalog as AnimationCatalog;
use anyhow::Result;
use meshlet_catalog::Catalog as MeshletCatalog;
use serde_json::Value;
use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Device};

use crate::rendering::async_resident::PublishedSnapshot;
use crate::scene::SceneState;

use super::renderer::SkeletalSceneRenderer;
use super::resources::ExecutionResources;
use super::surface::{
    SurfaceProbe, SurfaceProbeContext, SurfaceRenderer, SurfaceRendererInput, SurfaceSettings,
};

pub(super) unsafe fn create_surface(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
    resources: &ExecutionResources,
    mesh: &MeshletCatalog,
    animation: &AnimationCatalog,
    extent: [u32; 2],
) -> Result<SurfaceRenderer> {
    unsafe {
        SurfaceRenderer::new(
            device,
            SurfaceRendererInput {
                queue,
                source_heap: &resources.heap,
                source_visible: &resources.visible,
                source_counters: &resources.counters,
                mesh,
                animation,
                extent,
            },
        )
    }
}

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

    pub fn enable_surface_occlusion(&mut self) {
        self.surface.enable_occlusion();
    }

    pub fn disable_surface_occlusion(&mut self) {
        self.surface.disable_occlusion();
    }

    pub fn reset_surface_occlusion(&mut self) {
        self.surface.reset_occlusion_history();
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
                mesh_catalog: &self.mesh_catalog,
                scene,
                skeletal_settings: self.settings,
                config: snapshot.config,
                background_color,
                timestamp_readback: &self.resources.timestamp_readback,
                timestamp_frequency: self.timestamp_frequency,
            })
        }
    }
}
