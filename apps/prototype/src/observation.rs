use engine_runtime::{CanonicalObjectNearestQuery, TerrainPosition};
use reference_host::HostElapsedSample;

pub(crate) const OBJECT_OBSERVATION_RADIUS_Q9: u32 = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Status {
    pub pending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Completed {
    pub origin: TerrainPosition,
    pub query: CanonicalObjectNearestQuery,
}

pub(crate) struct Policy {
    pending: bool,
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self { pending: false }
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
        query: CanonicalObjectNearestQuery,
    ) -> Option<Completed> {
        if !self.wants_completion(step_count) {
            return None;
        }
        self.pending = false;
        Some(Completed { origin, query })
    }

    pub(crate) const fn status(&self) -> Status {
        Status {
            pending: self.pending,
        }
    }
}
