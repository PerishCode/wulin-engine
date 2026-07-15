#[path = "../src/time.rs"]
mod time;

use engine_runtime::{ActorSimulationOutcome, ActorSimulationRenderBlock};
use reference_host::HostElapsedSample;

#[test]
fn only_ready_is_admitted() {
    assert_eq!(time::admitted_elapsed(HostElapsedSample::Reset), None);
    assert_eq!(time::admitted_elapsed(HostElapsedSample::Suspended), None);
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Stalled {
            elapsed_nanoseconds: 125_000_001,
            maximum_elapsed_nanoseconds: 125_000_000,
        }),
        None
    );
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Ready {
            elapsed_nanoseconds: 0,
        }),
        Some(0)
    );
    assert_eq!(
        time::admitted_elapsed(HostElapsedSample::Ready {
            elapsed_nanoseconds: 125_000_000,
        }),
        Some(125_000_000)
    );
}

#[test]
fn render_block_is_consumed_without_an_advance_or_backlog() {
    let blocked = ActorSimulationOutcome::RenderBlocked(ActorSimulationRenderBlock {
        prepared_step_count: 1,
        terrain_query_count: 1,
    });
    let mut count = 0;
    assert_eq!(
        time::consume_actor_outcome(blocked, &mut count).unwrap(),
        None
    );
    assert_eq!(count, 1);

    let mut exhausted = u64::MAX;
    assert!(time::consume_actor_outcome(blocked, &mut exhausted).is_err());
    assert_eq!(exhausted, u64::MAX);
}
