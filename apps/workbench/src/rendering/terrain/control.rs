use anyhow::Result;

use crate::load::LoadConfig;
use crate::terrain::{TerrainCompletion, TerrainScheduleReport};

use super::super::renderer::Renderer;

impl Renderer {
    pub fn open_terrain_pack(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<terrain_format::PackMetadata> {
        self.terrain_streamer.open(path)
    }

    pub fn schedule_terrain(&mut self, config: LoadConfig) -> Result<TerrainScheduleReport> {
        let reservation = self.terrain_renderer.reserve(config)?;
        let transaction_id = reservation.transaction_id;
        match self.terrain_streamer.schedule(reservation) {
            Ok(report) => Ok(report),
            Err(error) => {
                let _ = self.terrain_renderer.cancel(transaction_id);
                Err(error)
            }
        }
    }

    pub fn terrain_status(&self) -> serde_json::Value {
        serde_json::json!({
            "stream": self.terrain_streamer.status_json(),
            "renderer": self.terrain_renderer.status_json(),
        })
    }

    pub fn enable_terrain(&mut self) -> Result<()> {
        self.disable_meshlet_scene();
        self.disable_skeletal_scene();
        self.terrain_renderer.enable()
    }

    pub fn disable_terrain(&mut self) {
        self.terrain_renderer.disable();
    }

    pub fn terrain_mode_enabled(&self) -> bool {
        self.terrain_renderer.is_enabled()
    }

    pub fn terrain_config(&self) -> Option<LoadConfig> {
        self.terrain_renderer.config()
    }

    pub fn terrain_lod_status(&self) -> serde_json::Value {
        serde_json::to_value(self.terrain_renderer.lod_settings())
            .expect("terrain LOD settings should serialize")
    }

    pub fn configure_terrain_lod(
        &mut self,
        near_patch_radius: u32,
        middle_patch_radius: u32,
        forced_lod: Option<u32>,
    ) -> Result<()> {
        self.terrain_renderer
            .configure_lod(near_patch_radius, middle_patch_radius, forced_lod)
    }

    pub fn enable_terrain_lod(&mut self) {
        self.terrain_renderer.enable_lod();
    }

    pub fn disable_terrain_lod(&mut self) {
        self.terrain_renderer.disable_lod();
    }

    pub fn arm_terrain_io_gate(&mut self) -> Result<u64> {
        self.terrain_streamer.arm_gate()
    }

    pub fn release_terrain_io_gate(&mut self) -> Result<u64> {
        self.terrain_streamer.release_gate()
    }

    pub fn arm_terrain_copy_gate(&mut self) -> Result<u64> {
        self.terrain_renderer.arm_copy_gate()
    }

    pub unsafe fn release_terrain_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.terrain_renderer.release_copy_gate() }
    }

    pub(in crate::rendering) unsafe fn poll_terrain_completion(&mut self) -> Result<()> {
        let Some(completion) = self.terrain_streamer.poll_completion() else {
            return Ok(());
        };
        match completion {
            TerrainCompletion::Ready {
                transaction_id,
                uploads,
                metrics,
            } => {
                let release_fence = self.next_fence_value;
                self.next_fence_value += 1;
                match unsafe {
                    self.terrain_renderer.submit(
                        transaction_id,
                        uploads,
                        metrics,
                        &self.queue,
                        &self.fence,
                        release_fence,
                    )
                } {
                    Ok(mut report) => self.terrain_streamer.mark_submitted(&mut report),
                    Err(error) => {
                        self.terrain_streamer.mark_failed(
                            transaction_id,
                            error.to_string(),
                            Default::default(),
                        );
                        Err(error)
                    }
                }
            }
            TerrainCompletion::Failed {
                transaction_id,
                message,
                metrics,
            } => {
                let cancellation = self.terrain_renderer.cancel(transaction_id);
                let message = match cancellation {
                    Ok(()) => message,
                    Err(error) => format!("{message}; terrain cancellation failed: {error}"),
                };
                self.terrain_streamer
                    .mark_failed(transaction_id, message, metrics);
                Ok(())
            }
        }
    }
}
