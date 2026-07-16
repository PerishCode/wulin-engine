use anyhow::{Result, ensure};
use engine_runtime::{
    CanonicalObjectProximity, CanonicalObjectResolution, ObjectSourceNamespace,
    ObjectTargetFeedback, ObjectTargetFeedbackKind, TerrainPosition,
};
use reference_host::HostElapsedSample;

pub(crate) const OBJECT_ACTION_RADIUS_Q9: u32 = 512;
pub(crate) const ACKNOWLEDGEMENT_FRAME_COUNT: u32 = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Acknowledgement {
    pub identity: engine_runtime::CanonicalObjectIdentity,
    pub remaining_frames: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Target {
    pub identity: engine_runtime::CanonicalObjectIdentity,
    pub available: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Status {
    pub pending: bool,
    pub acknowledgement: Option<Acknowledgement>,
    pub committed_count: u64,
    pub ineligible_count: u64,
    pub consumed: Option<engine_runtime::CanonicalObjectIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ineligible {
    MissingTarget,
    UnavailableTarget,
    SourceReplaced,
    OutsidePublishedWindow,
    OutsideRadius,
    OutsideFacing,
    CapacityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Facing {
    pub yaw_q16: u32,
    pub direction_x: i64,
    pub direction_z: i64,
    pub dot_q9: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Eligible {
    pub feedback: ObjectTargetFeedback,
    pub proximity: CanonicalObjectProximity,
    pub facing: Facing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Attempt {
    Eligible(Eligible),
    Ineligible(Ineligible),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameCompletion {
    pub applied: bool,
    pub feedback: ObjectTargetFeedback,
}

pub(crate) struct Policy {
    pending: bool,
    acknowledgement: Option<Acknowledgement>,
    committed_count: u64,
    ineligible_count: u64,
    consumed: Option<engine_runtime::CanonicalObjectIdentity>,
}

fn resolved_attempt(
    target: Target,
    origin: TerrainPosition,
    yaw_q16: u32,
    resolution: Option<CanonicalObjectResolution>,
) -> Result<Attempt> {
    match resolution
        .ok_or_else(|| anyhow::anyhow!("resolved target action has no object resolution"))?
    {
        CanonicalObjectResolution::Resolved(object) => {
            ensure!(
                object.identity == target.identity,
                "object action resolution diverged from the retained identity"
            );
            proximity_attempt(target, origin, yaw_q16, object)
        }
        CanonicalObjectResolution::SourceReplaced => {
            Ok(Attempt::Ineligible(Ineligible::SourceReplaced))
        }
        CanonicalObjectResolution::OutsidePublishedWindow => {
            Ok(Attempt::Ineligible(Ineligible::OutsidePublishedWindow))
        }
    }
}

fn proximity_attempt(
    target: Target,
    origin: TerrainPosition,
    yaw_q16: u32,
    object: engine_runtime::CanonicalObject,
) -> Result<Attempt> {
    let Some(proximity) = object.proximity_from(origin, OBJECT_ACTION_RADIUS_Q9)? else {
        return Ok(Attempt::Ineligible(Ineligible::OutsideRadius));
    };
    let facing = facing(proximity, yaw_q16)?;
    if proximity.distance_squared_q18 != 0 && facing.dot_q9 <= 0 {
        return Ok(Attempt::Ineligible(Ineligible::OutsideFacing));
    }
    Ok(Attempt::Eligible(Eligible {
        feedback: ObjectTargetFeedback {
            identity: target.identity,
            kind: ObjectTargetFeedbackKind::Activated,
        },
        proximity,
        facing,
    }))
}

fn facing(proximity: CanonicalObjectProximity, yaw_q16: u32) -> Result<Facing> {
    let (direction_x, direction_z) = match yaw_q16 {
        0 => (1, 0),
        8_192 => (1, 1),
        16_384 => (0, 1),
        24_576 => (-1, 1),
        32_768 => (-1, 0),
        40_960 => (-1, -1),
        49_152 => (0, -1),
        57_344 => (1, -1),
        _ => {
            return Err(anyhow::anyhow!(
                "object action received non-eight-way actor yaw"
            ));
        }
    };
    let dot_q9 = proximity
        .delta_x_q9
        .checked_mul(direction_x)
        .and_then(|x| {
            proximity
                .delta_z_q9
                .checked_mul(direction_z)
                .and_then(|z| x.checked_add(z))
        })
        .ok_or_else(|| anyhow::anyhow!("object action facing dot product overflowed"))?;
    Ok(Facing {
        yaw_q16,
        direction_x,
        direction_z,
        dot_q9,
    })
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self {
            pending: false,
            acknowledgement: None,
            committed_count: 0,
            ineligible_count: 0,
            consumed: None,
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

    pub(crate) fn prepare_after_advance(
        &mut self,
        step_count: u32,
        origin: TerrainPosition,
        yaw_q16: u32,
        target: Option<Target>,
        resolution: Option<CanonicalObjectResolution>,
    ) -> Result<Option<Attempt>> {
        if !self.wants_completion(step_count) {
            return Ok(None);
        }
        let attempt = match target {
            _ if self.consumed.is_some() => {
                ensure!(
                    resolution.is_none(),
                    "exhausted object consumption capacity must not resolve another target"
                );
                Attempt::Ineligible(Ineligible::CapacityExhausted)
            }
            None => {
                ensure!(
                    resolution.is_none(),
                    "missing target must not carry a resolution"
                );
                Attempt::Ineligible(Ineligible::MissingTarget)
            }
            Some(target) if !target.available => {
                ensure!(
                    resolution.is_none(),
                    "unavailable target must not trigger object resolution"
                );
                Attempt::Ineligible(Ineligible::UnavailableTarget)
            }
            Some(target) => resolved_attempt(target, origin, yaw_q16, resolution)?,
        };
        let next_ineligible_count = if matches!(attempt, Attempt::Ineligible(_)) {
            self.ineligible_count
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("object action ineligible count overflowed"))?
        } else {
            self.ineligible_count
        };
        self.pending = false;
        self.ineligible_count = next_ineligible_count;
        if matches!(attempt, Attempt::Ineligible(_)) {
            self.acknowledgement = None;
        }
        Ok(Some(attempt))
    }

    pub(crate) fn frame_feedback(
        &self,
        target: Option<engine_runtime::CanonicalObjectIdentity>,
        attempt: Option<Attempt>,
    ) -> Option<ObjectTargetFeedback> {
        if let Some(Attempt::Eligible(eligible)) = attempt {
            return Some(eligible.feedback);
        }
        if let Some(acknowledgement) = self.acknowledgement {
            return Some(ObjectTargetFeedback {
                identity: acknowledgement.identity,
                kind: ObjectTargetFeedbackKind::Activated,
            });
        }
        target.map(|identity| ObjectTargetFeedback {
            identity,
            kind: ObjectTargetFeedbackKind::Selected,
        })
    }

    pub(crate) fn complete_frame(
        &mut self,
        attempt: Option<Attempt>,
        submitted: Option<ObjectTargetFeedback>,
        rendered: Option<ObjectTargetFeedback>,
    ) -> Result<Option<FrameCompletion>> {
        ensure!(
            rendered.is_none() || rendered == submitted,
            "rendered object feedback diverged from the submitted frame candidate"
        );
        if let Some(Attempt::Eligible(eligible)) = attempt {
            ensure!(
                submitted == Some(eligible.feedback),
                "eligible object action was not submitted as the frame candidate"
            );
            let applied = rendered == Some(eligible.feedback);
            if applied {
                ensure!(
                    self.consumed.is_none(),
                    "object consumption capacity changed after eligibility"
                );
                let committed_count = self
                    .committed_count
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("object action committed count overflowed"))?;
                self.acknowledgement = Some(Acknowledgement {
                    identity: eligible.feedback.identity,
                    remaining_frames: ACKNOWLEDGEMENT_FRAME_COUNT - 1,
                });
                self.consumed = Some(eligible.feedback.identity);
                self.committed_count = committed_count;
            } else {
                self.acknowledgement = None;
            }
            return Ok(Some(FrameCompletion {
                applied,
                feedback: eligible.feedback,
            }));
        }
        if let Some(mut acknowledgement) = self.acknowledgement
            && rendered
                == Some(ObjectTargetFeedback {
                    identity: acknowledgement.identity,
                    kind: ObjectTargetFeedbackKind::Activated,
                })
        {
            acknowledgement.remaining_frames -= 1;
            self.acknowledgement =
                (acknowledgement.remaining_frames != 0).then_some(acknowledgement);
        }
        Ok(None)
    }

    pub(crate) fn observe_target(&mut self, target: Option<Target>) {
        let resolved_identity = target
            .filter(|target| target.available)
            .map(|target| target.identity);
        if self
            .acknowledgement
            .is_some_and(|acknowledgement| Some(acknowledgement.identity) != resolved_identity)
        {
            self.acknowledgement = None;
        }
    }

    pub(crate) fn observe_source(&mut self, source_namespace: ObjectSourceNamespace) {
        if self
            .consumed
            .is_some_and(|identity| identity.source_namespace != source_namespace)
        {
            self.consumed = None;
            self.acknowledgement = None;
        }
    }

    pub(crate) const fn nearest_exclusion(
        &self,
    ) -> Option<engine_runtime::CanonicalObjectIdentity> {
        self.consumed
    }

    pub(crate) const fn frame_suppression(
        &self,
    ) -> Option<engine_runtime::CanonicalObjectIdentity> {
        if self.acknowledgement.is_none() {
            self.consumed
        } else {
            None
        }
    }

    pub(crate) const fn status(&self) -> Status {
        Status {
            pending: self.pending,
            acknowledgement: self.acknowledgement,
            committed_count: self.committed_count,
            ineligible_count: self.ineligible_count,
            consumed: self.consumed,
        }
    }
}
