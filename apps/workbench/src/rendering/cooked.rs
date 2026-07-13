use anyhow::Result;

use crate::cooked::{CookCompletion, CookScheduleReport};
use crate::load::LoadConfig;

use super::renderer::Renderer;

impl Renderer {
    pub fn open_cooked_pack(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<region_format::PackMetadata> {
        self.cooked_streamer.open(path)
    }

    pub fn stream_cooked_resident(&mut self, config: LoadConfig) -> Result<CookScheduleReport> {
        self.load_renderer.disable();
        self.resident_renderer.disable();
        let reservation = self.async_resident_renderer.reserve(config)?;
        let transaction_id = reservation.transaction_id;
        match self.cooked_streamer.schedule(reservation) {
            Ok(report) => Ok(report),
            Err(error) => {
                let _ = self
                    .async_resident_renderer
                    .cancel_reservation(transaction_id);
                Err(error)
            }
        }
    }

    pub fn cooked_status(&self) -> serde_json::Value {
        self.cooked_streamer.status_json()
    }

    pub fn arm_cooked_io_gate(&mut self) -> Result<u64> {
        self.cooked_streamer.arm_gate()
    }

    pub fn release_cooked_io_gate(&mut self) -> Result<u64> {
        self.cooked_streamer.release_gate()
    }

    pub(super) unsafe fn poll_cooked_completion(&mut self) -> Result<()> {
        let Some(completion) = self.cooked_streamer.poll_completion() else {
            return Ok(());
        };
        match completion {
            CookCompletion::Ready {
                transaction_id,
                uploads,
                preparation_ms,
            } => {
                let release_fence = self.next_fence_value;
                self.next_fence_value += 1;
                match unsafe {
                    self.async_resident_renderer.submit(
                        transaction_id,
                        uploads,
                        preparation_ms,
                        &self.queue,
                        &self.fence,
                        release_fence,
                    )
                } {
                    Ok(report) => self.cooked_streamer.mark_submitted(report),
                    Err(error) => {
                        self.cooked_streamer.mark_failed(
                            transaction_id,
                            error.to_string(),
                            Default::default(),
                        );
                        Err(error)
                    }
                }
            }
            CookCompletion::Failed {
                transaction_id,
                message,
                metrics,
            } => {
                let cancellation = self
                    .async_resident_renderer
                    .cancel_reservation(transaction_id);
                let message = match cancellation {
                    Ok(()) => message,
                    Err(error) => format!("{message}; reservation cancellation failed: {error}"),
                };
                self.cooked_streamer
                    .mark_failed(transaction_id, message, metrics);
                Ok(())
            }
        }
    }
}
