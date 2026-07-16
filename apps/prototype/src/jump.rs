use anyhow::{Result, ensure};
use engine_runtime::{ActorSimulationOutcome, ActorStateTransition};
use reference_host::HostElapsedSample;

// Nearest integer Q16 displacement per fixed step to 4 m/s at 60 Hz.
pub(crate) const JUMP_VELOCITY_DELTA_Q16: i32 = 4_369;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Status {
    pub pending: bool,
    pub grounded: bool,
}

pub(crate) struct Policy {
    pending: bool,
    grounded: bool,
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self {
            pending: false,
            grounded: true,
        }
    }

    pub(crate) fn ingest(&mut self, pressed: bool) {
        if pressed && self.grounded {
            self.pending = true;
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

    pub(crate) const fn initial_velocity_delta_q16(&self) -> i32 {
        if self.pending {
            JUMP_VELOCITY_DELTA_Q16
        } else {
            0
        }
    }

    pub(crate) fn observe_outcome(&mut self, outcome: ActorSimulationOutcome) -> Result<()> {
        match outcome {
            ActorSimulationOutcome::Advanced(advance) => self.observe_advance(advance.actor),
            ActorSimulationOutcome::RenderBlocked(_) => Ok(()),
        }
    }

    pub(crate) fn observe_advance(&mut self, advance: ActorStateTransition) -> Result<()> {
        ensure!(
            (advance.step_count == 0) == advance.last_step_grounded.is_none(),
            "prototype jump advance grounded witness shape diverged"
        );
        if let Some(grounded) = advance.last_step_grounded {
            self.pending = false;
            self.grounded = grounded;
        }
        Ok(())
    }

    pub(crate) const fn status(&self) -> Status {
        Status {
            pending: self.pending,
            grounded: self.grounded,
        }
    }
}
