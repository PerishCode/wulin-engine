use anyhow::Result;

use crate::terrain::TerrainCompletion;

use super::super::renderer::Renderer;

pub(in crate::rendering) enum TerrainPollOutcome {
    Submitted,
    Failed {
        transaction_id: u64,
        message: String,
    },
}

impl Renderer {
    pub fn open_terrain_pack(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<serde_json::Value> {
        self.terrain_streamer.open(path)
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

    pub(in crate::rendering) unsafe fn poll_terrain_completion(
        &mut self,
    ) -> Result<Option<TerrainPollOutcome>> {
        let Some(completion) = self.terrain_streamer.poll_completion() else {
            return Ok(None);
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
                    Ok(mut report) => {
                        self.terrain_streamer.mark_submitted(&mut report)?;
                        Ok(Some(TerrainPollOutcome::Submitted))
                    }
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
                    .mark_failed(transaction_id, message.clone(), metrics);
                Ok(Some(TerrainPollOutcome::Failed {
                    transaction_id,
                    message,
                }))
            }
        }
    }
}
