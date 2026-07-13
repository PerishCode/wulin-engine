use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::address::GlobalRegionConfig;
use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::scene::Camera;
use crate::world::RegionCoord;

use super::super::renderer::Renderer;

const TRAVERSAL_REVISION: &str = "camera-region-traversal-v1";
const WORLD_MIN_METERS: f64 = -1_032.0;
const LOCAL_ORIGIN: u32 = MAX_REGION_SIDE / 2;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct TraversalBasis {
    world_region_side: u32,
    active_radius: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_origin: Option<RegionCoord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TraversalTarget {
    config: LoadConfig,
    global_config: Option<GlobalRegionConfig>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledTarget {
    token: u64,
    config: LoadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_config: Option<GlobalRegionConfig>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlockedTarget {
    config: LoadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_config: Option<GlobalRegionConfig>,
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
        self.session_count += 1;
        Ok(())
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
        published: TraversalTarget,
        pending: Option<TraversalTarget>,
    ) -> Result<Option<TraversalTarget>> {
        if !self.enabled {
            return Ok(None);
        }
        let basis = self
            .basis
            .as_ref()
            .expect("enabled traversal has no immutable basis");
        ensure!(
            basis.target(published.config)? == published,
            "published composition changed the traversal basis"
        );
        let desired = basis.target(map_camera(camera, basis)?)?;
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
            self.replace_queued((desired != pending).then_some(desired));
            return Ok(None);
        }
        self.queued = None;
        if desired == published
            || self
                .blocked
                .as_ref()
                .is_some_and(|blocked| blocked.target() == desired)
        {
            return Ok(None);
        }
        Ok(Some(desired))
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
        global_config: Option<GlobalRegionConfig>,
    ) {
        let target = TraversalTarget {
            config,
            global_config,
        };
        self.automatic_publication_count += 1;
        self.last_published = Some(ScheduledTarget::new(token, target));
        if self
            .blocked
            .as_ref()
            .is_some_and(|blocked| blocked.target() == target)
        {
            self.blocked = None;
        }
    }

    pub(super) fn mark_failed(
        &mut self,
        config: LoadConfig,
        global_config: Option<GlobalRegionConfig>,
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
        json!({
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
        })
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
}

impl TraversalBasis {
    fn new(published: TraversalTarget) -> Result<Self> {
        if let Some(global) = published.global_config {
            ensure!(
                global.local_config()? == published.config,
                "published global composition does not match its local config"
            );
        }
        let basis = Self {
            world_region_side: published.config.world_region_side,
            active_radius: published.config.active_radius,
            global_origin: published.global_config.map(|value| value.global_origin),
        };
        if basis.global_origin.is_some() {
            ensure!(
                basis.world_region_side == MAX_REGION_SIDE,
                "signed traversal requires the format-V1 world extent"
            );
            let (minimum, maximum) = basis.center_bounds();
            basis.global_center(minimum, minimum)?;
            basis.global_center(maximum, maximum)?;
        }
        Ok(basis)
    }

    fn target(self, config: LoadConfig) -> Result<TraversalTarget> {
        ensure!(
            config.world_region_side == self.world_region_side
                && config.active_radius == self.active_radius,
            "traversal target changed the immutable basis"
        );
        let global_config = self
            .global_origin
            .map(|origin| {
                let center = self.global_center(config.active_center_x, config.active_center_z)?;
                GlobalRegionConfig::new(
                    origin.x,
                    origin.z,
                    center.x,
                    center.z,
                    config.active_radius,
                )
            })
            .transpose()?;
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
        let origin = self
            .global_origin
            .context("local traversal basis has no global origin")?;
        origin.checked_offset(
            i64::from(local_x) - i64::from(LOCAL_ORIGIN),
            i64::from(local_z) - i64::from(LOCAL_ORIGIN),
        )
    }
}

impl TraversalTarget {
    fn status_json(self) -> Value {
        match self.global_config {
            Some(global) => json!({"config": self.config, "globalConfig": global}),
            None => json!(self.config),
        }
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
            .map(|value| TraversalTarget {
                config: value.config,
                global_config: value.global_config,
            });
        let Some(target) = self
            .composition
            .traversal
            .plan(camera, published, pending)?
        else {
            return Ok(());
        };
        self.composition.traversal.mark_attempted();
        match unsafe { self.schedule_composition_pair(target.config, target.global_config, true) } {
            Ok(value) => {
                let token = value["token"]
                    .as_u64()
                    .expect("composition schedule response omitted token");
                self.composition.traversal.mark_scheduled(token, target);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(x: f32, z: f32) -> Camera {
        Camera {
            position: [x, 5.0, z],
            target: [x, 0.0, z - 1.0],
            vertical_fov_degrees: 60.0,
            near_plane_meters: 0.1,
        }
    }

    #[test]
    fn far_target_is_exact() {
        let far = 1_i64 << 40;
        let global = GlobalRegionConfig::new(far, -far, far, -far, 2).unwrap();
        let published = TraversalTarget {
            config: global.local_config().unwrap(),
            global_config: Some(global),
        };
        let basis = TraversalBasis::new(published).unwrap();
        let local = map_camera(camera(16.0, -16.0), &basis).unwrap();
        let target = basis.target(local).unwrap();
        assert_eq!((local.active_center_x, local.active_center_z), (65, 63));
        let mapped = target.global_config.unwrap();
        assert_eq!(mapped.global_origin, RegionCoord::new(far, -far));
        assert_eq!(mapped.global_center, RegionCoord::new(far + 1, -far - 1));
    }

    #[test]
    fn legacy_status_is_unchanged() {
        let config = LoadConfig::new(128, 64, 64, 2).unwrap();
        let target = TraversalTarget {
            config,
            global_config: None,
        };
        assert_eq!(target.status_json(), json!(config));
        let basis = TraversalBasis::new(target).unwrap();
        assert_eq!(json!(basis).get("globalOrigin"), None);
    }

    #[test]
    fn basis_rejects_extent_overflow() {
        let global = GlobalRegionConfig::new(i64::MAX, 0, i64::MAX, 0, 2).unwrap();
        let published = TraversalTarget {
            config: global.local_config().unwrap(),
            global_config: Some(global),
        };
        assert!(TraversalBasis::new(published).is_err());
    }
}
