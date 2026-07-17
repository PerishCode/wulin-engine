#![allow(dead_code)]

#[path = "../src/actor.rs"]
mod actor;
#[path = "../src/camera.rs"]
mod camera;
#[path = "../src/object/interaction.rs"]
pub(crate) mod interaction;
#[path = "../src/jump.rs"]
mod jump;
#[path = "../src/object/observation.rs"]
pub(crate) mod observation;
#[path = "../src/session.rs"]
mod session;

mod object {
    pub(crate) use crate::{interaction, observation};
}

use engine_runtime::{
    ActorHandle, ActorPresentation, CanonicalObjectIdentity, ObjectSourceNamespace, RegionCoord,
    RuntimeActor, TERRAIN_BODY_HEIGHT_DENOMINATOR, TerrainHeight, TerrainPosition, TerrainTriangle,
};
use reference_host::HostClockStatus;

fn identity() -> CanonicalObjectIdentity {
    CanonicalObjectIdentity {
        source_namespace: ObjectSourceNamespace::from_bytes([7; 32]),
        region: RegionCoord::new(3, -4),
        authored_local_id: 19,
    }
}

fn runtime_actor() -> RuntimeActor {
    let position = TerrainPosition::new(RegionCoord::new(3, -4), 64, 96).unwrap();
    let terrain = TerrainHeight {
        height_numerator: 0,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::First,
    };
    RuntimeActor {
        handle: ActorHandle::new(1).unwrap(),
        motion: actor::initial_motion(position, terrain).unwrap(),
        presentation: ActorPresentation::animated(7, 63, 0, 1, 0, 0),
        animation_epoch_tick: 80,
    }
}

fn clock() -> HostClockStatus {
    HostClockStatus {
        suspended: false,
        has_baseline: true,
        sample_count: 9,
        reset_count: 1,
        ready_count: 8,
        stall_count: 0,
        suspended_sample_count: 0,
        suspend_count: 0,
        resume_count: 0,
    }
}

fn completion(
    bootstrap_frame_count: u64,
    live_frame_count: u64,
    reason: session::CompletionReason,
) -> session::Completion {
    session::Completion {
        reason,
        actor: runtime_actor(),
        clock: clock(),
        bootstrap_frame_count,
        live_frame_count,
        camera_anchor_count: live_frame_count,
        render_block_count: 0,
        object_target_frame_count: 12,
        object_action_frame_count: 1,
        object_rejection_frame_count: 0,
        object_suppression_frame_count: 4,
        observation_status: observation::Status {
            pending: false,
            target: None,
        },
        interaction_status: interaction::Status {
            pending: false,
            acknowledgement: None,
            committed_count: 1,
            ineligible_count: 1,
            consumed: Some(identity()),
        },
    }
}

#[test]
fn graceful_completion_is_exact_bounded_final_state() {
    let value =
        session::completion_value(completion(4, 17, session::CompletionReason::Escape)).unwrap();
    assert_eq!(value["role"], "prototype-session-completion");
    assert_eq!(value["revision"], session::REVISION);
    assert_eq!(value["sequence"], 2);
    assert_eq!(value["outcome"], "completed");
    assert_eq!(value["reason"], "escape");
    assert_eq!(value["frames"]["totalFrameCount"], 21);
    assert_eq!(value["frames"]["suppressionProjectedFrameCount"], 4);
    assert_eq!(
        value["object_observation"]["target"],
        serde_json::Value::Null
    );
    assert_eq!(value["object_interaction"]["committedCount"], 1);
    assert_eq!(value["object_interaction"]["ineligibleCount"], 1);
    assert_eq!(
        value["object_interaction"]["consumed"],
        serde_json::json!(identity())
    );
    assert_eq!(
        value["object_interaction"]["nearestExclusion"],
        serde_json::json!(identity())
    );
}

#[test]
fn window_close_reason_is_distinct_and_frame_total_is_checked() {
    let value = session::completion_value(completion(2, 3, session::CompletionReason::WindowClose))
        .unwrap();
    assert_eq!(value["reason"], "window-close");
    assert!(
        session::completion_value(completion(u64::MAX, 1, session::CompletionReason::Escape,))
            .is_err()
    );
}
