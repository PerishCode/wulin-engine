use anyhow::Result;

use super::Renderer;

impl Renderer {
    pub fn calibration_mode_active(&self) -> bool {
        !self.composition_enabled()
    }

    pub fn arm_async_copy_gate(&mut self) -> Result<u64> {
        self.async_resident_renderer.arm_gate()
    }

    pub unsafe fn release_async_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.async_resident_renderer.release_gate() }
    }
}
