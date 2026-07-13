use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::scene::Camera;

use super::super::renderer::Renderer;

const TRAVERSAL_REVISION: &str = "camera-region-traversal-v1";
const WORLD_MIN_METERS: f64 = -1_032.0;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraversalBasis {
    world_region_side: u32,
    active_radius: u32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledTarget {
    token: u64,
    config: LoadConfig,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlockedTarget {
    config: LoadConfig,
    message: String,
}

#[derive(Default)]
pub(super) struct CameraTraversal {
    enabled: bool,
    basis: Option<TraversalBasis>,
    desired: Option<LoadConfig>,
    queued: Option<LoadConfig>,
    blocked: Option<BlockedTarget>,
    session_count: u64,
    desired_change_count: u64,
    automatic_attempt_count: u64,
    automatic_schedule_count: u64,
    automatic_publication_count: u64,
    coalesced_replacement_count: u64,
    max_queued_depth: u32,
    last_scheduled: Option<ScheduledTarget>,
    last_published: Option<ScheduledTarget>,
    last_failure: Option<BlockedTarget>,
}

impl CameraTraversal {
    pub(super) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn enable(&mut self, published: LoadConfig) {
        self.enabled = true;
        self.basis = Some(TraversalBasis {
            world_region_side: published.world_region_side,
            active_radius: published.active_radius,
        });
        self.desired = None;
        self.queued = None;
        self.blocked = None;
        self.session_count += 1;
    }

    pub(super) fn disable(&mut self) {
        self.enabled = false;
        self.basis = None;
        self.desired = None;
        self.queued = None;
        self.blocked = None;
    }

    pub(super) fn plan(
        &mut self,
        camera: Camera,
        published: LoadConfig,
        pending: Option<LoadConfig>,
    ) -> Result<Option<LoadConfig>> {
        if !self.enabled {
            return Ok(None);
        }
        let basis = self
            .basis
            .as_ref()
            .expect("enabled traversal has no immutable basis");
        ensure!(
            published.world_region_side == basis.world_region_side
                && published.active_radius == basis.active_radius,
            "published composition changed the traversal basis"
        );
        let desired = map_camera(camera, basis)?;
        if self.desired != Some(desired) {
            self.desired = Some(desired);
            self.desired_change_count += 1;
            if self
                .blocked
                .as_ref()
                .is_some_and(|blocked| blocked.config != desired)
            {
                self.blocked = None;
            }
        }

        if let Some(pending) = pending {
            self.replace_queued((desired != pending).then_some(desired));
            return Ok(None);
        }
        self.queued = None;
        if desired == published
            || self
                .blocked
                .as_ref()
                .is_some_and(|blocked| blocked.config == desired)
        {
            return Ok(None);
        }
        Ok(Some(desired))
    }

    pub(super) fn mark_scheduled(&mut self, token: u64, config: LoadConfig) {
        self.automatic_schedule_count += 1;
        self.last_scheduled = Some(ScheduledTarget { token, config });
        if self.queued == Some(config) {
            self.queued = None;
        }
    }

    pub(super) fn mark_attempted(&mut self) {
        self.automatic_attempt_count += 1;
    }

    pub(super) fn mark_published(&mut self, token: u64, config: LoadConfig) {
        self.automatic_publication_count += 1;
        self.last_published = Some(ScheduledTarget { token, config });
        if self
            .blocked
            .as_ref()
            .is_some_and(|blocked| blocked.config == config)
        {
            self.blocked = None;
        }
    }

    pub(super) fn mark_failed(&mut self, config: LoadConfig, message: String) {
        let failure = BlockedTarget { config, message };
        if self.enabled {
            self.blocked = Some(failure.clone());
        }
        self.last_failure = Some(failure);
    }

    pub(super) fn status_json(&self) -> Value {
        json!({
            "revision": TRAVERSAL_REVISION,
            "enabled": self.enabled,
            "basis": self.basis,
            "desired": self.desired,
            "queued": self.queued,
            "blocked": self.blocked,
            "sessionCount": self.session_count,
            "desiredChangeCount": self.desired_change_count,
            "automaticAttemptCount": self.automatic_attempt_count,
            "automaticScheduleCount": self.automatic_schedule_count,
            "automaticPublicationCount": self.automatic_publication_count,
            "coalescedReplacementCount": self.coalesced_replacement_count,
            "maxQueuedDepth": self.max_queued_depth,
            "lastScheduled": self.last_scheduled,
            "lastPublished": self.last_published,
            "lastFailure": self.last_failure,
        })
    }

    fn replace_queued(&mut self, next: Option<LoadConfig>) {
        if self.queued == next {
            return;
        }
        if self.queued.is_some() {
            self.coalesced_replacement_count += 1;
        }
        self.queued = next;
        self.max_queued_depth = self.max_queued_depth.max(u32::from(next.is_some()));
    }
}

impl Renderer {
    pub fn enable_composition_traversal(&mut self) -> Result<()> {
        ensure!(
            self.composition.enabled,
            "camera traversal requires composition mode"
        );
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        let config = self
            .composition
            .published
            .as_ref()
            .context("camera traversal requires a published pair")?
            .config;
        self.composition.traversal.enable(config);
        Ok(())
    }

    pub fn disable_composition_traversal(&mut self) {
        self.composition.traversal.disable();
    }

    pub(in crate::rendering) unsafe fn drive_composition_traversal(
        &mut self,
        camera: Camera,
    ) -> Result<()> {
        if !self.composition.traversal.is_enabled() {
            return Ok(());
        }
        let published = self
            .composition
            .published
            .as_ref()
            .context("enabled camera traversal has no published pair")?
            .config;
        let pending = self.composition.pending.as_ref().map(|value| value.config);
        let Some(config) = self
            .composition
            .traversal
            .plan(camera, published, pending)?
        else {
            return Ok(());
        };
        self.composition.traversal.mark_attempted();
        match unsafe { self.schedule_composition_pair(config, true) } {
            Ok(value) => {
                let token = value["token"]
                    .as_u64()
                    .expect("composition schedule response omitted token");
                self.composition.traversal.mark_scheduled(token, config);
            }
            Err(error) => {
                self.composition
                    .traversal
                    .mark_failed(config, format!("{error:#}"));
            }
        }
        Ok(())
    }
}

fn map_camera(camera: Camera, basis: &TraversalBasis) -> Result<LoadConfig> {
    let world_start = (MAX_REGION_SIDE - basis.world_region_side) / 2;
    let minimum = world_start + basis.active_radius;
    let maximum = world_start + basis.world_region_side - basis.active_radius - 1;
    let map_axis = |position: f32| {
        (((f64::from(position) - WORLD_MIN_METERS) / f64::from(terrain_format::REGION_SIDE_METERS))
            .floor() as i64)
            .clamp(i64::from(minimum), i64::from(maximum)) as u32
    };
    LoadConfig::new(
        basis.world_region_side,
        map_axis(camera.position[0]),
        map_axis(camera.position[2]),
        basis.active_radius,
    )
}
