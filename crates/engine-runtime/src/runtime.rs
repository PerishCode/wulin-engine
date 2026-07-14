use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;
use windows::Win32::Foundation::HWND;

use crate::rendering::{RenderOutcome, Renderer};
use crate::scene::SceneState;
use crate::streaming::address::GlobalRegionConfig;
use crate::world::RegionCoord;

#[derive(Clone, Copy)]
pub struct FrameRequest {
    pub clear_color: [f32; 4],
    pub capture: bool,
    pub capture_object_ids: bool,
    pub probe: bool,
}

pub struct Runtime {
    renderer: Renderer,
    scene: SceneState,
}

impl Runtime {
    /// Creates the native D3D12 runtime for one window.
    ///
    /// # Safety
    ///
    /// `hwnd` must identify a live window owned by the calling thread and must remain valid until
    /// this runtime is dropped. The extent must describe that window's renderable client area.
    pub unsafe fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            renderer: unsafe { Renderer::new(hwnd, width, height)? },
            scene: SceneState::new(),
        })
    }

    /// Advances and presents one frame on the runtime's owning thread.
    ///
    /// # Safety
    ///
    /// The window supplied to [`Runtime::new`] must still be live, and this call must execute on
    /// the thread that created the runtime while no external code uses its native GPU objects.
    pub unsafe fn frame(&mut self, request: FrameRequest) -> Result<RenderOutcome> {
        unsafe {
            self.renderer.render(
                request.clear_color,
                request.capture,
                request.capture_object_ids,
                request.probe,
                &mut self.scene,
            )
        }
    }

    /// Waits for all runtime-owned GPU and streaming work to become idle.
    ///
    /// # Safety
    ///
    /// This must execute on the runtime's owning thread while its native window and device remain
    /// valid.
    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.renderer.wait_idle() }
    }

    pub fn adapter_name(&self) -> &str {
        self.renderer.adapter_name()
    }

    pub fn debug_layer(&self) -> bool {
        self.renderer.debug_layer()
    }

    pub fn mesh_shader_tier(&self) -> u32 {
        self.renderer.mesh_shader_tier()
    }

    pub fn shader_model(&self) -> &str {
        self.renderer.shader_model()
    }

    pub fn barycentrics_supported(&self) -> bool {
        self.renderer.barycentrics_supported()
    }

    pub fn rasterizer_ordered_views_supported(&self) -> bool {
        self.renderer.rasterizer_ordered_views_supported()
    }

    pub fn visibility_format_supported(&self) -> bool {
        self.renderer.visibility_format_supported()
    }

    pub fn color_uav_format_supported(&self) -> bool {
        self.renderer.color_uav_format_supported()
    }

    /// Returns the current native device-removal reason, if any.
    ///
    /// # Safety
    ///
    /// The runtime's native device must not be used concurrently outside this facade.
    pub unsafe fn device_removed_reason(&self) -> Option<String> {
        unsafe { self.renderer.device_removed_reason() }
    }

    pub fn calibration_mode_active(&self) -> bool {
        self.renderer.calibration_mode_active()
    }

    pub fn camera_json(&self) -> Value {
        self.scene.camera_json()
    }

    pub fn reset_camera(&mut self) -> Value {
        self.scene.reset_camera();
        self.scene.camera_json()
    }

    pub fn set_camera(
        &mut self,
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<Value> {
        self.scene.set_camera(
            position,
            target,
            vertical_fov_degrees,
            self.renderer.calibration_mode_active(),
        )?;
        Ok(self.scene.camera_json())
    }

    pub fn objects_json(&self) -> Value {
        self.scene.objects_json()
    }

    pub fn spatial_json(&self) -> Value {
        self.scene.spatial_json()
    }

    pub fn world_json(&self) -> Result<Value> {
        self.scene.world_json()
    }

    pub fn relocate_world(&mut self, region: RegionCoord) -> Result<Value> {
        self.scene.relocate_world(region)?;
        self.scene.world_json()
    }

    pub fn rebase_world(&mut self, region: RegionCoord) -> Result<Value> {
        self.scene.rebase_world(region)?;
        self.scene.world_json()
    }

    pub fn reset_world(&mut self) -> Result<Value> {
        self.scene.reset_world()?;
        self.scene.world_json()
    }

    pub fn world_probe_json(&self) -> Result<Value> {
        self.scene.world_probe_json()
    }

    pub fn open_terrain_pack(&mut self, path: PathBuf) -> Result<Value> {
        self.renderer.open_terrain_pack(path)
    }

    pub fn open_cooked_object_pack(&mut self, path: PathBuf) -> Result<Value> {
        self.renderer.open_cooked_object_pack(path)
    }

    pub fn composition_status(&self) -> Value {
        self.renderer.composition_status()
    }

    pub fn composition_enabled(&self) -> bool {
        self.renderer.composition_enabled()
    }

    pub fn presentation_time_status(&self) -> Value {
        self.renderer.presentation_time_status()
    }

    pub fn pause_presentation_time(&mut self) -> Value {
        self.renderer.pause_presentation_time()
    }

    pub fn resume_presentation_time(&mut self) -> Value {
        self.renderer.resume_presentation_time()
    }

    pub fn set_presentation_time(&mut self, tick: u32) -> Result<Value> {
        self.renderer.set_presentation_time(tick)
    }

    pub fn step_presentation_time(&mut self, ticks: u32) -> Result<Value> {
        self.renderer.step_presentation_time(ticks)
    }

    /// Schedules one canonical terrain/object pair publication.
    ///
    /// # Safety
    ///
    /// This must execute on the runtime's owning thread while its native device and source
    /// streamers are live.
    pub unsafe fn schedule_global_composition(
        &mut self,
        config: GlobalRegionConfig,
    ) -> Result<Value> {
        unsafe { self.renderer.schedule_global_composition(config) }
    }

    pub fn enable_composition_traversal(&mut self) -> Result<()> {
        self.renderer.enable_composition_traversal()
    }

    pub fn disable_composition_traversal(&mut self) {
        self.renderer.disable_composition_traversal();
    }

    pub fn enable_composition_prefetch(&mut self) -> Result<()> {
        self.renderer.enable_composition_prefetch()
    }

    pub fn disable_composition_prefetch(&mut self) -> Result<()> {
        self.renderer.disable_composition_prefetch()
    }

    pub fn arm_object_io_gate(&mut self) -> Result<u64> {
        self.renderer.arm_object_io_gate()
    }

    pub fn release_object_io_gate(&mut self) -> Result<u64> {
        self.renderer.release_object_io_gate()
    }

    pub fn arm_object_copy_gate(&mut self) -> Result<u64> {
        self.renderer.arm_async_copy_gate()
    }

    /// Releases the deliberately armed object-copy gate.
    ///
    /// # Safety
    ///
    /// The matching gate must have been armed on this runtime, and release must occur on the
    /// runtime's owning thread.
    pub unsafe fn release_object_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.renderer.release_async_copy_gate() }
    }

    pub fn arm_terrain_io_gate(&mut self) -> Result<u64> {
        self.renderer.arm_terrain_io_gate()
    }

    pub fn release_terrain_io_gate(&mut self) -> Result<u64> {
        self.renderer.release_terrain_io_gate()
    }

    pub fn arm_terrain_copy_gate(&mut self) -> Result<u64> {
        self.renderer.arm_terrain_copy_gate()
    }

    /// Releases the deliberately armed terrain-copy gate.
    ///
    /// # Safety
    ///
    /// The matching gate must have been armed on this runtime, and release must occur on the
    /// runtime's owning thread.
    pub unsafe fn release_terrain_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.renderer.release_terrain_copy_gate() }
    }
}
