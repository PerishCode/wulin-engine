use anyhow::{Result, ensure};
use engine_runtime::{
    CanonicalObjectIdentity, CanonicalObjectNearestQuery, CanonicalObjectResolution,
    CanonicalObjectSnapshot, TerrainPosition,
};
use reference_host::HostElapsedSample;

pub(crate) const OBJECT_OBSERVATION_RADIUS_Q9: u32 = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Availability {
    Resolved,
    OutsidePublishedWindow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Target {
    pub identity: CanonicalObjectIdentity,
    pub snapshot: CanonicalObjectSnapshot,
    pub availability: Availability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Status {
    pub pending: bool,
    pub target: Option<Target>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Completed {
    pub origin: TerrainPosition,
    pub snapshot: CanonicalObjectSnapshot,
    pub query: CanonicalObjectNearestQuery,
}

pub(crate) struct Policy {
    pending: bool,
    target: Option<Target>,
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self {
            pending: false,
            target: None,
        }
    }

    pub(crate) fn observe_sample(&mut self, sample: HostElapsedSample) {
        if matches!(
            sample,
            HostElapsedSample::Reset | HostElapsedSample::Suspended
        ) {
            self.pending = false;
        }
    }

    pub(crate) fn ingest(&mut self, pressed: bool) {
        self.pending |= pressed;
    }

    pub(crate) const fn wants_completion(&self, step_count: u32) -> bool {
        self.pending && step_count != 0
    }

    pub(crate) fn complete_after_advance(
        &mut self,
        step_count: u32,
        origin: TerrainPosition,
        snapshot: CanonicalObjectSnapshot,
        query: CanonicalObjectNearestQuery,
    ) -> Result<Option<Completed>> {
        if !self.wants_completion(step_count) {
            return Ok(None);
        }
        let target = query
            .nearest
            .map(|nearest| {
                ensure!(
                    nearest.object.identity.source_namespace == snapshot.source_namespace,
                    "observed object identity does not belong to the observed snapshot"
                );
                Ok(Target {
                    identity: nearest.object.identity,
                    snapshot,
                    availability: Availability::Resolved,
                })
            })
            .transpose()?;
        self.target = target;
        self.pending = false;
        Ok(Some(Completed {
            origin,
            snapshot,
            query,
        }))
    }

    pub(crate) const fn has_target(&self) -> bool {
        self.target.is_some()
    }

    pub(crate) const fn target_identity(&self) -> Option<CanonicalObjectIdentity> {
        match self.target {
            Some(target) => Some(target.identity),
            None => None,
        }
    }

    pub(crate) fn validation_request(
        &self,
        snapshot: CanonicalObjectSnapshot,
    ) -> Option<CanonicalObjectIdentity> {
        self.target
            .filter(|target| target.snapshot != snapshot)
            .map(|target| target.identity)
    }

    pub(crate) fn complete_validation(
        &mut self,
        snapshot: CanonicalObjectSnapshot,
        resolution: CanonicalObjectResolution,
    ) -> Result<()> {
        let target = self
            .target
            .ok_or_else(|| anyhow::anyhow!("object target disappeared before validation"))?;
        ensure!(
            target.snapshot != snapshot,
            "unchanged object snapshot must not trigger target resolution"
        );
        let next = match resolution {
            CanonicalObjectResolution::Resolved(object) => {
                ensure!(
                    object.identity == target.identity
                        && object.identity.source_namespace == snapshot.source_namespace,
                    "resolved object does not match the retained target and snapshot"
                );
                Some(Target {
                    snapshot,
                    availability: Availability::Resolved,
                    ..target
                })
            }
            CanonicalObjectResolution::OutsidePublishedWindow => {
                ensure!(
                    target.identity.source_namespace == snapshot.source_namespace,
                    "window departure reported across an object source replacement"
                );
                Some(Target {
                    snapshot,
                    availability: Availability::OutsidePublishedWindow,
                    ..target
                })
            }
            CanonicalObjectResolution::SourceReplaced => {
                ensure!(
                    target.identity.source_namespace != snapshot.source_namespace,
                    "object source replacement retained the same namespace"
                );
                None
            }
        };
        self.target = next;
        Ok(())
    }

    pub(crate) const fn status(&self) -> Status {
        Status {
            pending: self.pending,
            target: self.target,
        }
    }
}
