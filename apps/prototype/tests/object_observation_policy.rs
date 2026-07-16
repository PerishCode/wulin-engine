#[path = "../src/observation.rs"]
mod observation;

use engine_runtime::{
    CanonicalObject, CanonicalObjectIdentity, CanonicalObjectNearest, CanonicalObjectNearestQuery,
    CanonicalObjectPresentation, CanonicalObjectResolution, CanonicalObjectSnapshot,
    ObjectSourceNamespace, RegionCoord, TerrainPosition,
};
use reference_host::HostElapsedSample;

fn origin() -> TerrainPosition {
    TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap()
}

fn namespace(value: u8) -> ObjectSourceNamespace {
    ObjectSourceNamespace::from_bytes([value; 32])
}

fn snapshot(publication_token: u64, source: u8) -> CanonicalObjectSnapshot {
    CanonicalObjectSnapshot {
        publication_token,
        source_namespace: namespace(source),
    }
}

fn identity(source: u8, authored_local_id: u32) -> CanonicalObjectIdentity {
    CanonicalObjectIdentity {
        source_namespace: namespace(source),
        region: RegionCoord::ZERO,
        authored_local_id,
    }
}

fn object(source: u8, authored_local_id: u32) -> CanonicalObject {
    CanonicalObject {
        identity: identity(source, authored_local_id),
        position: [0.0, 0.0, 0.0],
        height: 1.0,
        presentation: CanonicalObjectPresentation::static_object(0, 0, 0),
    }
}

fn query(source: u8, authored_local_id: Option<u32>) -> CanonicalObjectNearestQuery {
    CanonicalObjectNearestQuery {
        candidate_count: 25_600,
        nearest: authored_local_id.map(|authored_local_id| CanonicalObjectNearest {
            object: object(source, authored_local_id),
            terrain_position: origin(),
            delta_x_q9: 0,
            delta_z_q9: 0,
            distance_squared_q18: 0,
        }),
    }
}

fn acquire(policy: &mut observation::Policy, source: u8, authored_local_id: u32) {
    policy.ingest(true);
    policy
        .complete_after_advance(
            1,
            origin(),
            snapshot(1, source),
            query(source, Some(authored_local_id)),
        )
        .unwrap()
        .unwrap();
}

#[test]
fn intent_waits_for_commit_then_retains_only_qualified_target() {
    assert_eq!(observation::OBJECT_OBSERVATION_RADIUS_Q9, 512);
    let mut policy = observation::Policy::new();
    assert!(!policy.has_target());
    policy.ingest(true);
    policy.ingest(true);
    assert_eq!(
        policy.status(),
        observation::Status {
            pending: true,
            target: None,
        }
    );
    assert!(!policy.wants_completion(0));
    assert!(
        policy
            .complete_after_advance(0, origin(), snapshot(1, 1), query(1, Some(7)))
            .unwrap()
            .is_none()
    );

    let completed = policy
        .complete_after_advance(1, origin(), snapshot(1, 1), query(1, Some(7)))
        .unwrap()
        .unwrap();
    assert_eq!(completed.origin, origin());
    assert_eq!(completed.snapshot, snapshot(1, 1));
    assert_eq!(completed.query, query(1, Some(7)));
    assert_eq!(
        policy.status(),
        observation::Status {
            pending: false,
            target: Some(observation::Target {
                identity: identity(1, 7),
                snapshot: snapshot(1, 1),
                availability: observation::Availability::Resolved,
            }),
        }
    );
    assert!(policy.has_target());
}

#[test]
fn successful_empty_scan_clears_previous_target() {
    let mut policy = observation::Policy::new();
    acquire(&mut policy, 1, 7);
    policy.ingest(true);
    let completed = policy
        .complete_after_advance(1, origin(), snapshot(1, 1), query(1, None))
        .unwrap();
    assert!(completed.is_some());
    assert_eq!(policy.status().target, None);
    assert!(!policy.status().pending);
}

#[test]
fn malformed_acquisition_preserves_pending_and_previous_target() {
    let mut policy = observation::Policy::new();
    acquire(&mut policy, 1, 7);
    policy.ingest(true);
    let before = policy.status();
    assert!(
        policy
            .complete_after_advance(1, origin(), snapshot(1, 1), query(2, Some(9)))
            .is_err()
    );
    assert_eq!(policy.status(), before);
}

#[test]
fn changed_snapshots_drive_one_typed_validation_and_equal_stamps_eliminate_work() {
    let mut policy = observation::Policy::new();
    acquire(&mut policy, 1, 7);
    assert_eq!(policy.validation_request(snapshot(1, 1)), None);

    assert_eq!(
        policy.validation_request(snapshot(2, 1)),
        Some(identity(1, 7))
    );
    policy
        .complete_validation(
            snapshot(2, 1),
            CanonicalObjectResolution::OutsidePublishedWindow,
        )
        .unwrap();
    assert_eq!(
        policy.status().target.unwrap(),
        observation::Target {
            identity: identity(1, 7),
            snapshot: snapshot(2, 1),
            availability: observation::Availability::OutsidePublishedWindow,
        }
    );
    assert_eq!(policy.validation_request(snapshot(2, 1)), None);

    assert_eq!(
        policy.validation_request(snapshot(3, 1)),
        Some(identity(1, 7))
    );
    policy
        .complete_validation(
            snapshot(3, 1),
            CanonicalObjectResolution::Resolved(object(1, 7)),
        )
        .unwrap();
    assert_eq!(
        policy.status().target.unwrap().availability,
        observation::Availability::Resolved
    );

    assert_eq!(
        policy.validation_request(snapshot(4, 2)),
        Some(identity(1, 7))
    );
    policy
        .complete_validation(snapshot(4, 2), CanonicalObjectResolution::SourceReplaced)
        .unwrap();
    assert_eq!(policy.status().target, None);
}

#[test]
fn invalid_validation_preserves_the_complete_target() {
    let mut policy = observation::Policy::new();
    acquire(&mut policy, 1, 7);
    let before = policy.status();
    assert!(
        policy
            .complete_validation(snapshot(2, 1), CanonicalObjectResolution::SourceReplaced)
            .is_err()
    );
    assert_eq!(policy.status(), before);
    assert!(
        policy
            .complete_validation(
                snapshot(2, 1),
                CanonicalObjectResolution::Resolved(object(1, 8)),
            )
            .is_err()
    );
    assert_eq!(policy.status(), before);
    assert!(
        policy
            .complete_validation(
                snapshot(1, 1),
                CanonicalObjectResolution::Resolved(object(1, 7)),
            )
            .is_err()
    );
    assert_eq!(policy.status(), before);
}

#[test]
fn discontinuity_cancels_only_pending_intent() {
    let mut policy = observation::Policy::new();
    acquire(&mut policy, 1, 7);
    let target = policy.status().target;
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
    assert_eq!(policy.status().target, target);

    policy.ingest(true);
    policy.observe_sample(HostElapsedSample::Suspended);
    assert!(!policy.status().pending);
    assert_eq!(policy.status().target, target);
}
