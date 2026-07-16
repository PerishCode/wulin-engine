#[path = "../src/jump.rs"]
mod jump;

use engine_runtime::{
    ActorHandle, ActorPresentation, ActorSimulationOutcome, ActorSimulationRenderBlock,
    ActorStateTransition, RegionCoord, RuntimeActor, TerrainBody, TerrainBodyMotion,
    TerrainPosition,
};
use reference_host::HostElapsedSample;

fn actor() -> RuntimeActor {
    RuntimeActor {
        handle: ActorHandle::new(1).unwrap(),
        motion: TerrainBodyMotion::new(
            TerrainBody::new(
                TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap(),
                65_536,
                65_536,
            )
            .unwrap(),
            0,
        ),
        presentation: ActorPresentation::animated(7, 63, 0, 0, 0, 0),
        animation_epoch_tick: 0,
    }
}

fn advance(step_count: u32, last_step_grounded: Option<bool>) -> ActorStateTransition {
    ActorStateTransition {
        input: actor(),
        output: actor(),
        step_count,
        terrain_query_count: step_count,
        last_step_grounded,
    }
}

#[test]
fn grounded_press_is_capacity_one_and_midair_press_is_ignored() {
    let mut policy = jump::Policy::new();
    assert_eq!(
        policy.status(),
        jump::Status {
            pending: false,
            grounded: true
        }
    );

    policy.ingest(true);
    policy.ingest(true);
    assert_eq!(
        policy.initial_velocity_delta_q16(),
        jump::JUMP_VELOCITY_DELTA_Q16
    );
    policy.observe_advance(advance(1, Some(false))).unwrap();
    assert_eq!(
        policy.status(),
        jump::Status {
            pending: false,
            grounded: false
        }
    );

    policy.ingest(true);
    assert_eq!(policy.initial_velocity_delta_q16(), 0);
    policy.observe_advance(advance(1, Some(true))).unwrap();
    policy.ingest(true);
    assert_eq!(
        policy.status(),
        jump::Status {
            pending: true,
            grounded: true
        }
    );
}

#[test]
fn fractional_and_render_block_retain_until_nonzero_commit() {
    let mut policy = jump::Policy::new();
    policy.ingest(true);
    policy.observe_advance(advance(0, None)).unwrap();
    policy
        .observe_outcome(ActorSimulationOutcome::RenderBlocked(
            ActorSimulationRenderBlock {
                prepared_step_count: 1,
                terrain_query_count: 1,
            },
        ))
        .unwrap();
    assert_eq!(
        policy.status(),
        jump::Status {
            pending: true,
            grounded: true
        }
    );

    policy.observe_advance(advance(1, Some(false))).unwrap();
    assert_eq!(
        policy.status(),
        jump::Status {
            pending: false,
            grounded: false
        }
    );
}

#[test]
fn discontinuity_cancels_while_stall_retains() {
    let mut policy = jump::Policy::new();
    policy.ingest(true);
    policy.observe_sample(HostElapsedSample::Stalled {
        elapsed_nanoseconds: 125_000_001,
        maximum_elapsed_nanoseconds: 125_000_000,
    });
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Reset);
    assert!(!policy.status().pending);

    policy.ingest(true);
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Ready {
        elapsed_nanoseconds: 1,
    });
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Suspended);
    assert!(!policy.status().pending);
}

#[test]
fn invalid_advance_shape_rolls_back() {
    let mut policy = jump::Policy::new();
    policy.ingest(true);
    let before = policy.status();
    assert!(policy.observe_advance(advance(0, Some(true))).is_err());
    assert_eq!(policy.status(), before);
    assert!(policy.observe_advance(advance(1, None)).is_err());
    assert_eq!(policy.status(), before);
}
