use anyhow::{Context, Result, ensure};

use crate::scene::Camera;

use super::{PendingTraversal, TraversalAction, TraversalTarget};
use crate::rendering::composition::{PairPurpose, Renderer};

impl Renderer {
    pub fn enable_composition_traversal(&mut self) -> Result<()> {
        ensure!(
            self.composition.enabled,
            "camera traversal requires composition mode"
        );
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        let published = self
            .composition
            .published
            .as_ref()
            .context("camera traversal requires a published pair")?;
        self.composition.traversal.enable(TraversalTarget {
            config: published.config,
            global_config: published.global_config,
        })
    }

    pub fn disable_composition_traversal(&mut self) {
        self.composition.traversal.disable();
    }

    pub fn enable_composition_prefetch(&mut self) -> Result<()> {
        ensure!(
            self.composition.traversal.is_enabled(),
            "composition prefetch requires camera traversal"
        );
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        self.composition.traversal.prefetch.enable()
    }

    pub fn disable_composition_prefetch(&mut self) -> Result<()> {
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        self.composition.traversal.prefetch.disable();
        Ok(())
    }

    pub(in crate::rendering) fn take_composition_camera_shift(&mut self) -> Option<[i32; 2]> {
        self.composition.traversal.take_camera_delta()
    }

    pub(in crate::rendering) unsafe fn drive_composition_traversal(
        &mut self,
        camera: Camera,
    ) -> Result<()> {
        if !self.composition.traversal.is_enabled() {
            return Ok(());
        }
        let published_pair = self
            .composition
            .published
            .as_ref()
            .context("enabled camera traversal has no published pair")?;
        let published = TraversalTarget {
            config: published_pair.config,
            global_config: published_pair.global_config,
        };
        let pending = self
            .composition
            .pending
            .as_ref()
            .map(|value| PendingTraversal {
                target: TraversalTarget {
                    config: value.config,
                    global_config: value.global_config,
                },
                prefetch: value.purpose.prefetch(),
            });
        let Some(action) = self
            .composition
            .traversal
            .plan(camera, published, pending)?
        else {
            return Ok(());
        };
        if let TraversalAction::PromotePrefetch(target) = action {
            let pending = self
                .composition
                .pending
                .as_mut()
                .context("prefetch promotion has no pending pair")?;
            ensure!(pending.purpose.prefetch(), "pending pair is not a prefetch");
            pending.purpose = PairPurpose::Traversal;
            self.composition.traversal.mark_prefetch_promoted(target);
            return Ok(());
        }
        let TraversalAction::Schedule { target, prefetch } = action else {
            unreachable!();
        };
        let purpose = if prefetch {
            PairPurpose::Prefetch
        } else {
            PairPurpose::Traversal
        };
        if !prefetch {
            self.composition.traversal.mark_attempted();
        }
        match unsafe {
            self.schedule_composition_pair(target.config, target.global_config, purpose)
        } {
            Ok(value) => {
                let token = value["token"]
                    .as_u64()
                    .expect("composition schedule response omitted token");
                if prefetch {
                    self.composition
                        .traversal
                        .mark_prefetch_scheduled(token, target);
                } else {
                    self.composition.traversal.mark_scheduled(token, target);
                }
            }
            Err(error) if prefetch => {
                self.composition
                    .traversal
                    .mark_prefetch_failed(target, format!("{error:#}"));
            }
            Err(error) => {
                self.composition.traversal.mark_failed(
                    target.config,
                    target.global_config,
                    format!("{error:#}"),
                );
            }
        }
        Ok(())
    }
}
