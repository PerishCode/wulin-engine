mod control;
mod prefetch;
mod rollover;
#[cfg(test)]
#[path = "../../../tests/private/composition_traversal.rs"]
mod tests;

use anyhow::{Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::address::GlobalRegionConfig;
use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::region::RegionCoord;
use crate::scene::Camera;

use prefetch::PrefetchState;
use rollover::{RolloverPolicy, RolloverState};

const TRAVERSAL_REVISION: &str = "camera-region-traversal-v1";
const WORLD_MIN_METERS: f64 = -1_032.0;
const LOCAL_ORIGIN: u32 = MAX_REGION_SIDE / 2;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraversalBasis {
    world_region_side: u32,
    active_radius: u32,
    global_origin: RegionCoord,
    rollover: RolloverPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TraversalTarget {
    pub(super) config: LoadConfig,
    pub(super) global_config: GlobalRegionConfig,
}

#[derive(Clone, Copy)]
struct PendingTraversal {
    target: TraversalTarget,
    prefetch: bool,
}

enum TraversalAction {
    Schedule {
        target: TraversalTarget,
        prefetch: bool,
    },
    PromotePrefetch(TraversalTarget),
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledTarget {
    token: u64,
    config: LoadConfig,
    global_config: GlobalRegionConfig,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlockedTarget {
    config: LoadConfig,
    global_config: GlobalRegionConfig,
    message: String,
}

#[derive(Default)]
pub(super) struct CameraTraversal {
    enabled: bool,
    basis: Option<TraversalBasis>,
    desired: Option<TraversalTarget>,
    queued: Option<TraversalTarget>,
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
    rollover: RolloverState,
    prefetch: PrefetchState,
}

impl CameraTraversal {
    pub(super) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(super) fn enable(&mut self, published: TraversalTarget) -> Result<()> {
        let basis = TraversalBasis::new(published)?;
        self.enabled = true;
        self.basis = Some(basis);
        self.desired = None;
        self.queued = None;
        self.blocked = None;
        self.rollover.activate();
        self.session_count += 1;
        Ok(())
    }

    pub(super) fn disable(&mut self) {
        self.enabled = false;
        self.basis = None;
        self.desired = None;
        self.queued = None;
        self.blocked = None;
        self.rollover.deactivate();
        self.prefetch.disable();
    }

    fn plan(
        &mut self,
        camera: Camera,
        published: TraversalTarget,
        pending: Option<PendingTraversal>,
    ) -> Result<Option<TraversalAction>> {
        if !self.enabled {
            return Ok(None);
        }
        let basis = *self.basis.as_ref().expect("enabled traversal has no basis");
        ensure!(
            basis.target(published.config)? == published,
            "published composition changed the traversal basis"
        );
        let desired = basis.camera_target(map_camera(camera, &basis)?)?;
        let prefetch = self.prefetch.observe(camera, basis, published, desired)?;
        if self.desired != Some(desired) {
            self.desired = Some(desired);
            self.desired_change_count += 1;
            if self
                .blocked
                .as_ref()
                .is_some_and(|blocked| blocked.target() != desired)
            {
                self.blocked = None;
            }
        }

        if let Some(pending) = pending {
            if pending.prefetch && desired == pending.target {
                self.replace_queued(None);
                return Ok(Some(TraversalAction::PromotePrefetch(desired)));
            }
            self.replace_queued(
                (desired != published && desired != pending.target).then_some(desired),
            );
            return Ok(None);
        }
        self.queued = None;
        if desired == published
            || self
                .blocked
                .as_ref()
                .is_some_and(|blocked| blocked.target() == desired)
        {
            return Ok(prefetch.map(|target| TraversalAction::Schedule {
                target,
                prefetch: true,
            }));
        }
        Ok(Some(TraversalAction::Schedule {
            target: desired,
            prefetch: false,
        }))
    }

    pub(super) fn mark_scheduled(&mut self, token: u64, target: TraversalTarget) {
        self.automatic_schedule_count += 1;
        self.last_scheduled = Some(ScheduledTarget::new(token, target));
        if self.queued == Some(target) {
            self.queued = None;
        }
    }

    pub(super) fn mark_attempted(&mut self) {
        self.automatic_attempt_count += 1;
    }

    pub(super) fn mark_published(
        &mut self,
        token: u64,
        config: LoadConfig,
        global_config: GlobalRegionConfig,
    ) -> Result<()> {
        let target = TraversalTarget {
            config,
            global_config,
        };
        self.commit_rollover(token, target)?;
        self.automatic_publication_count += 1;
        self.last_published = Some(ScheduledTarget::new(token, target));
        self.prefetch.mark_published(target);
        if self
            .blocked
            .as_ref()
            .is_some_and(|blocked| blocked.target() == target)
        {
            self.blocked = None;
        }
        Ok(())
    }

    pub(super) fn mark_prefetch_scheduled(&mut self, token: u64, target: TraversalTarget) {
        self.prefetch.mark_scheduled(token, target);
    }

    pub(super) fn mark_prefetch_completed(
        &mut self,
        token: u64,
        target: TraversalTarget,
        evidence: Value,
    ) {
        self.prefetch.mark_completed(token, target, evidence);
    }

    pub(super) fn mark_prefetch_promoted(&mut self, target: TraversalTarget) {
        self.prefetch.mark_promoted(target);
    }

    pub(super) fn mark_prefetch_failed(&mut self, target: TraversalTarget, message: String) {
        self.prefetch.mark_failed(target, message);
    }

    pub(super) fn mark_failed(
        &mut self,
        config: LoadConfig,
        global_config: GlobalRegionConfig,
        message: String,
    ) {
        let failure = BlockedTarget::new(
            TraversalTarget {
                config,
                global_config,
            },
            message,
        );
        if self.enabled {
            self.blocked = Some(failure.clone());
        }
        self.last_failure = Some(failure);
    }

    pub(super) fn status_json(&self) -> Value {
        let mut value = json!({
            "revision": TRAVERSAL_REVISION,
            "enabled": self.enabled,
            "basis": self.basis,
            "desired": self.desired.map(TraversalTarget::status_json),
            "queued": self.queued.map(TraversalTarget::status_json),
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
        });
        if let Some(rollover) = self.rollover.status_json() {
            value["rollover"] = rollover;
        }
        if let Some(prefetch) = self.prefetch.status_json() {
            value["prefetch"] = prefetch;
        }
        value
    }

    fn replace_queued(&mut self, next: Option<TraversalTarget>) {
        if self.queued == next {
            return;
        }
        if self.queued.is_some() {
            self.coalesced_replacement_count += 1;
        }
        self.queued = next;
        self.max_queued_depth = self.max_queued_depth.max(u32::from(next.is_some()));
    }

    fn commit_rollover(&mut self, token: u64, target: TraversalTarget) -> Result<()> {
        let Some(basis) = self.basis.as_mut() else {
            return Ok(());
        };
        self.rollover.commit(token, basis, target)
    }

    fn take_camera_delta(&mut self) -> Option<[i32; 2]> {
        self.rollover.take_camera_delta()
    }
}

impl TraversalBasis {
    fn new(published: TraversalTarget) -> Result<Self> {
        ensure!(
            published.global_config.local_config()? == published.config,
            "published global composition does not match its local config"
        );
        let basis = Self {
            world_region_side: published.config.world_region_side,
            active_radius: published.config.active_radius,
            global_origin: published.global_config.global_origin,
            rollover: RolloverPolicy::canonical(),
        };
        ensure!(
            basis.world_region_side == MAX_REGION_SIDE,
            "signed traversal requires the canonical world extent"
        );
        Ok(basis)
    }

    fn camera_target(self, config: LoadConfig) -> Result<TraversalTarget> {
        self.rollover.target(self, config)
    }

    fn target(self, config: LoadConfig) -> Result<TraversalTarget> {
        ensure!(
            config.world_region_side == self.world_region_side
                && config.active_radius == self.active_radius,
            "traversal target changed the immutable basis"
        );
        let center = self.global_center(config.active_center_x, config.active_center_z)?;
        let global_config = GlobalRegionConfig::new(
            self.global_origin.x,
            self.global_origin.z,
            center.x,
            center.z,
            config.active_radius,
        )?;
        Ok(TraversalTarget {
            config,
            global_config,
        })
    }

    fn center_bounds(self) -> (u32, u32) {
        let world_start = (MAX_REGION_SIDE - self.world_region_side) / 2;
        (
            world_start + self.active_radius,
            world_start + self.world_region_side - self.active_radius - 1,
        )
    }

    fn global_center(self, local_x: u32, local_z: u32) -> Result<RegionCoord> {
        self.global_origin.checked_offset(
            i64::from(local_x) - i64::from(LOCAL_ORIGIN),
            i64::from(local_z) - i64::from(LOCAL_ORIGIN),
        )
    }
}

impl TraversalTarget {
    fn status_json(self) -> Value {
        json!({"config": self.config, "globalConfig": self.global_config})
    }
}

impl ScheduledTarget {
    fn new(token: u64, target: TraversalTarget) -> Self {
        Self {
            token,
            config: target.config,
            global_config: target.global_config,
        }
    }

    fn target(&self) -> TraversalTarget {
        TraversalTarget {
            config: self.config,
            global_config: self.global_config,
        }
    }
}

impl BlockedTarget {
    fn new(target: TraversalTarget, message: String) -> Self {
        Self {
            config: target.config,
            global_config: target.global_config,
            message,
        }
    }

    fn target(&self) -> TraversalTarget {
        TraversalTarget {
            config: self.config,
            global_config: self.global_config,
        }
    }
}

fn map_camera(camera: Camera, basis: &TraversalBasis) -> Result<LoadConfig> {
    let (minimum, maximum) = basis.center_bounds();
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
