use anyhow::Result;
use serde_json::{Value, json};

use crate::load::LoadConfig;
use crate::scene::Camera;

use super::{ScheduledTarget, TraversalBasis, TraversalTarget, WORLD_MIN_METERS};

const TRIGGER_DISTANCE_METERS: f64 = 4.0;
const MOTION_EPSILON_METERS: f64 = 0.001;

#[derive(Default)]
pub(super) struct PrefetchState {
    configured: bool,
    enabled: bool,
    previous_camera: Option<[f32; 2]>,
    candidate: Option<TraversalTarget>,
    prepared: Option<ScheduledTarget>,
    failed: Option<TraversalTarget>,
    schedule_count: u64,
    completion_count: u64,
    promotion_count: u64,
    failure_count: u64,
    last_scheduled: Option<ScheduledTarget>,
    last_completed: Option<Value>,
    last_failure: Option<Value>,
}

impl PrefetchState {
    pub(super) fn enable(&mut self) -> Result<()> {
        self.configured = true;
        self.enabled = true;
        self.previous_camera = None;
        self.candidate = None;
        self.failed = None;
        Ok(())
    }

    pub(super) fn disable(&mut self) {
        if !self.configured {
            return;
        }
        self.enabled = false;
        self.previous_camera = None;
        self.candidate = None;
        self.failed = None;
    }

    pub(super) fn observe(
        &mut self,
        camera: Camera,
        basis: TraversalBasis,
        published: TraversalTarget,
        desired: TraversalTarget,
    ) -> Result<Option<TraversalTarget>> {
        let current = [camera.position[0], camera.position[2]];
        let previous = self.previous_camera.replace(current);
        if !self.enabled || desired != published {
            self.candidate = None;
            return Ok(None);
        }
        let Some(previous) = previous else {
            self.candidate = None;
            return Ok(None);
        };
        let motion = [
            f64::from(current[0] - previous[0]),
            f64::from(current[1] - previous[1]),
        ];
        let offset_x = predicted_offset(
            f64::from(current[0]),
            published.config.active_center_x,
            motion[0],
        );
        let offset_z = predicted_offset(
            f64::from(current[1]),
            published.config.active_center_z,
            motion[1],
        );
        if offset_x == 0 && offset_z == 0 {
            self.candidate = None;
            return Ok(None);
        }
        let center_x = checked_offset(published.config.active_center_x, offset_x)?;
        let center_z = checked_offset(published.config.active_center_z, offset_z)?;
        let local = LoadConfig::new(
            published.config.world_region_side,
            center_x,
            center_z,
            published.config.active_radius,
        )?;
        let candidate = basis.camera_target(local)?;
        self.candidate = Some(candidate);
        if self
            .prepared
            .as_ref()
            .is_some_and(|prepared| prepared.target() == candidate)
            || self.failed == Some(candidate)
        {
            return Ok(None);
        }
        Ok(Some(candidate))
    }

    pub(super) fn mark_scheduled(&mut self, token: u64, target: TraversalTarget) {
        let scheduled = ScheduledTarget::new(token, target);
        self.schedule_count += 1;
        self.last_scheduled = Some(scheduled);
    }

    pub(super) fn mark_completed(
        &mut self,
        token: u64,
        target: TraversalTarget,
        mut evidence: Value,
    ) {
        let scheduled = ScheduledTarget::new(token, target);
        self.completion_count += 1;
        self.prepared = Some(scheduled.clone());
        evidence["token"] = json!(token);
        evidence["config"] = json!(target.config);
        evidence["globalConfig"] = json!(target.global_config);
        self.last_completed = Some(evidence);
        self.failed = None;
        self.last_failure = None;
    }

    pub(super) fn mark_promoted(&mut self, target: TraversalTarget) {
        self.promotion_count += 1;
        self.candidate = Some(target);
    }

    pub(super) fn mark_failed(&mut self, target: TraversalTarget, message: String) {
        self.failure_count += 1;
        self.failed = Some(target);
        self.last_failure = Some(json!({
            "target": target.status_json(),
            "message": message,
        }));
    }

    pub(super) fn mark_published(&mut self, target: TraversalTarget) {
        if self
            .prepared
            .as_ref()
            .is_some_and(|prepared| prepared.target() == target)
        {
            self.prepared = None;
        }
        self.previous_camera = None;
        self.candidate = None;
        self.failed = None;
    }

    pub(super) fn status_json(&self) -> Option<Value> {
        self.configured.then(|| {
            json!({
                "revision": "canonical-traversal-prefetch-v1",
                "enabled": self.enabled,
                "triggerDistanceMeters": TRIGGER_DISTANCE_METERS,
                "motionEpsilonMeters": MOTION_EPSILON_METERS,
                "candidate": self.candidate.map(TraversalTarget::status_json),
                "prepared": self.prepared,
                "scheduleCount": self.schedule_count,
                "completionCount": self.completion_count,
                "promotionCount": self.promotion_count,
                "failureCount": self.failure_count,
                "lastScheduled": self.last_scheduled,
                "lastCompleted": self.last_completed,
                "lastFailure": self.last_failure,
            })
        })
    }
}

fn predicted_offset(position: f64, center: u32, motion: f64) -> i32 {
    let region_side = f64::from(terrain_format::REGION_SIDE_METERS);
    let start = WORLD_MIN_METERS + f64::from(center) * region_side;
    let end = start + region_side;
    if motion > MOTION_EPSILON_METERS && (end - position) <= TRIGGER_DISTANCE_METERS {
        1
    } else if motion < -MOTION_EPSILON_METERS && (position - start) <= TRIGGER_DISTANCE_METERS {
        -1
    } else {
        0
    }
}

fn checked_offset(value: u32, delta: i32) -> Result<u32> {
    let offset = i64::from(value) + i64::from(delta);
    u32::try_from(offset).map_err(Into::into)
}
