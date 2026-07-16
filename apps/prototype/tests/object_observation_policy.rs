#[path = "../src/observation.rs"]
mod observation;

use engine_runtime::{CanonicalObjectNearestQuery, RegionCoord, TerrainPosition};
use reference_host::HostElapsedSample;

fn origin() -> TerrainPosition {
    TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap()
}

fn query() -> CanonicalObjectNearestQuery {
    CanonicalObjectNearestQuery {
        candidate_count: 25_600,
        nearest: None,
    }
}

#[test]
fn intent_waits_for_commit() {
    assert_eq!(observation::OBJECT_OBSERVATION_RADIUS_Q9, 512);
    let mut policy = observation::Policy::new();
    policy.ingest(true);
    policy.ingest(true);
    assert_eq!(policy.status(), observation::Status { pending: true });
    assert!(!policy.wants_completion(0));
    assert!(
        policy
            .complete_after_advance(0, origin(), query())
            .is_none()
    );
    assert!(policy.status().pending);

    let completed = policy.complete_after_advance(1, origin(), query()).unwrap();
    assert_eq!(completed.origin, origin());
    assert_eq!(completed.query, query());
    assert_eq!(policy.status(), observation::Status { pending: false });
    assert!(
        policy
            .complete_after_advance(1, origin(), query())
            .is_none()
    );
}

#[test]
fn discontinuity_cancels_intent() {
    let mut policy = observation::Policy::new();
    policy.ingest(true);
    policy.observe_sample(HostElapsedSample::Stalled {
        elapsed_nanoseconds: 125_000_001,
        maximum_elapsed_nanoseconds: 125_000_000,
    });
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Ready {
        elapsed_nanoseconds: 1,
    });
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Reset);
    assert!(!policy.status().pending);

    policy.ingest(true);
    policy.observe_sample(HostElapsedSample::Suspended);
    assert!(!policy.status().pending);
}
