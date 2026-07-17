use anyhow::{Context, Result};
use engine_runtime::{
    ActorSimulationAdvance, ActorSimulationCommand, CanonicalObjectIdentity, ObjectTargetFeedback,
    RuntimeActor,
};
use reference_host::{HostClockStatus, HostElapsedSample};
use serde_json::{Value, json};

use crate::{
    camera, jump,
    object::{interaction, observation},
};

pub(crate) const REVISION: &str = "live-prototype-session-completion-v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionReason {
    Escape,
    WindowClose,
}

impl CompletionReason {
    const fn name(self) -> &'static str {
        match self {
            Self::Escape => "escape",
            Self::WindowClose => "window-close",
        }
    }
}

pub(crate) struct Readiness {
    pub startup: Value,
    pub traversal: Value,
    pub sample: HostElapsedSample,
    pub clock: HostClockStatus,
    pub advance: ActorSimulationAdvance,
    pub command: ActorSimulationCommand,
    pub bootstrap_frame_count: u64,
    pub live_frame_count: u64,
    pub anchored_rig: camera::Rig,
    pub anchored_camera: Value,
    pub camera_anchor_count: u64,
    pub render_block_count: u64,
    pub object_target_frame_count: u64,
    pub object_action_frame_count: u64,
    pub object_rejection_frame_count: u64,
    pub object_suppression_frame_count: u64,
    pub submitted_object_feedback: Option<ObjectTargetFeedback>,
    pub rendered_object_feedback: Option<ObjectTargetFeedback>,
    pub submitted_object_suppression: Option<CanonicalObjectIdentity>,
    pub rendered_object_suppression: Option<CanonicalObjectIdentity>,
    pub jump_status: jump::Status,
    pub object_observation: Option<observation::Completed>,
    pub observation_status: observation::Status,
    pub interaction_status: interaction::Status,
}

pub(crate) struct Completion {
    pub reason: CompletionReason,
    pub actor: RuntimeActor,
    pub clock: HostClockStatus,
    pub bootstrap_frame_count: u64,
    pub live_frame_count: u64,
    pub camera_anchor_count: u64,
    pub render_block_count: u64,
    pub object_target_frame_count: u64,
    pub object_action_frame_count: u64,
    pub object_rejection_frame_count: u64,
    pub object_suppression_frame_count: u64,
    pub observation_status: observation::Status,
    pub interaction_status: interaction::Status,
}

pub(crate) fn publish_readiness(evidence: Readiness) -> Result<()> {
    println!("{}", readiness_value(evidence)?);
    Ok(())
}

pub(crate) fn publish_completion(evidence: Completion) -> Result<()> {
    println!("{}", completion_value(evidence)?);
    Ok(())
}

fn readiness_value(evidence: Readiness) -> Result<Value> {
    let total_frame_count =
        total_frames(evidence.bootstrap_frame_count, evidence.live_frame_count)?;
    let object_observation = evidence.object_observation.map(|completed| {
        json!({
            "origin": completed.origin,
            "snapshot": completed.snapshot,
            "query": completed.query,
        })
    });
    Ok(json!({
        "role": "prototype",
        "sequence": 1,
        "instance_id": std::process::id().to_string(),
        "session_contract": {
            "revision": REVISION,
            "readinessSequence": 1,
            "completionSequence": 2,
            "completion": "graceful-exit-only",
        },
        "startup": evidence.startup,
        "traversal": evidence.traversal,
        "actor": {
            "capacity": 1,
            "liveCount": 1,
            "state": evidence.advance.actor.output,
        },
        "simulation_driver": {
            "revision": "live-prototype-locomotion-driver-v8",
            "sample": evidence.sample,
            "clock": evidence.clock,
            "command": evidence.command,
            "advance": evidence.advance,
            "renderBlockCount": evidence.render_block_count,
            "bootstrapFrameCount": evidence.bootstrap_frame_count,
            "liveFrameCount": evidence.live_frame_count,
            "totalFrameCount": total_frame_count,
        },
        "camera_driver": {
            "revision": "live-prototype-actor-camera-v2",
            "actor": evidence.advance.actor.output.handle,
            "rig": {
                "orbitIndex": evidence.anchored_rig.orbit_index,
                "positionOffset": evidence.anchored_rig.position_offset,
                "targetOffset": evidence.anchored_rig.target_offset,
                "verticalFovDegrees": evidence.anchored_rig.vertical_fov_degrees,
            },
            "camera": evidence.anchored_camera,
            "anchorCount": evidence.camera_anchor_count,
            "liveFrameCount": evidence.live_frame_count,
        },
        "jump_driver": {
            "revision": "live-prototype-jump-policy-v1",
            "stepVelocityDeltaQ16": jump::JUMP_VELOCITY_DELTA_Q16,
            "status": {
                "pending": evidence.jump_status.pending,
                "grounded": evidence.jump_status.grounded,
            },
        },
        "object_observation_driver": {
            "revision": "live-prototype-object-target-v4",
            "maxDistanceQ9": observation::OBJECT_OBSERVATION_RADIUS_Q9,
            "status": {
                "pending": evidence.observation_status.pending,
                "target": object_target(evidence.observation_status),
            },
            "completed": object_observation.is_some(),
            "observation": object_observation,
            "frameFeedback": {
                "submitted": evidence.submitted_object_feedback,
                "projected": evidence.rendered_object_feedback,
                "submittedFrameCount": evidence.object_target_frame_count,
            },
        },
        "object_interaction_driver": {
            "revision": "live-prototype-object-rejected-feedback-v3",
            "input": "Enter",
            "maxDistanceQ9": interaction::OBJECT_ACTION_RADIUS_Q9,
            "facingRule": {
                "domain": "committed-eight-way-yaw",
                "nonCoincidentDot": "positive",
                "coincidentEligible": true,
            },
            "acknowledgementFrameCount": interaction::ACKNOWLEDGEMENT_FRAME_COUNT,
            "status": interaction_status(evidence.interaction_status),
            "activatedFrameCount": evidence.object_action_frame_count,
            "rejectedFrameCount": evidence.object_rejection_frame_count,
            "suppression": {
                "submitted": evidence.submitted_object_suppression,
                "projected": evidence.rendered_object_suppression,
                "projectedFrameCount": evidence.object_suppression_frame_count,
            },
            "nearestExclusion": evidence.interaction_status.consumed,
        },
    }))
}

pub(crate) fn completion_value(evidence: Completion) -> Result<Value> {
    let total_frame_count =
        total_frames(evidence.bootstrap_frame_count, evidence.live_frame_count)?;
    Ok(json!({
        "role": "prototype-session-completion",
        "revision": REVISION,
        "sequence": 2,
        "instance_id": std::process::id().to_string(),
        "outcome": "completed",
        "reason": evidence.reason.name(),
        "actor": {
            "capacity": 1,
            "liveCount": 1,
            "state": evidence.actor,
        },
        "clock": evidence.clock,
        "frames": {
            "bootstrapFrameCount": evidence.bootstrap_frame_count,
            "liveFrameCount": evidence.live_frame_count,
            "totalFrameCount": total_frame_count,
            "cameraAnchorCount": evidence.camera_anchor_count,
            "renderBlockCount": evidence.render_block_count,
            "objectTargetFrameCount": evidence.object_target_frame_count,
            "activatedFrameCount": evidence.object_action_frame_count,
            "rejectedFrameCount": evidence.object_rejection_frame_count,
            "suppressionProjectedFrameCount": evidence.object_suppression_frame_count,
        },
        "object_observation": {
            "pending": evidence.observation_status.pending,
            "target": object_target(evidence.observation_status),
        },
        "object_interaction": {
            "capacity": 1,
            "pending": evidence.interaction_status.pending,
            "acknowledgement": evidence
                .interaction_status
                .acknowledgement
                .map(interaction::report::acknowledgement),
            "committedCount": evidence.interaction_status.committed_count,
            "ineligibleCount": evidence.interaction_status.ineligible_count,
            "consumed": evidence.interaction_status.consumed,
            "nearestExclusion": evidence.interaction_status.consumed,
        },
    }))
}

fn object_target(status: observation::Status) -> Option<Value> {
    status.target.map(|target| {
        json!({
            "identity": target.identity,
            "snapshot": target.snapshot,
            "availability": match target.availability {
                observation::Availability::Resolved => "resolved",
                observation::Availability::OutsidePublishedWindow => "outside-published-window",
            },
        })
    })
}

fn interaction_status(status: interaction::Status) -> Value {
    json!({
        "pending": status.pending,
        "acknowledgement": status
            .acknowledgement
            .map(interaction::report::acknowledgement),
        "committedCount": status.committed_count,
        "ineligibleCount": status.ineligible_count,
        "consumed": status.consumed,
    })
}

fn total_frames(bootstrap_frame_count: u64, live_frame_count: u64) -> Result<u64> {
    bootstrap_frame_count
        .checked_add(live_frame_count)
        .context("prototype total frame count overflowed")
}
