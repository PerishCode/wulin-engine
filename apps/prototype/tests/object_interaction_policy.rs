#[path = "../src/interaction.rs"]
mod interaction;
use engine_runtime::{
    CanonicalObject, CanonicalObjectIdentity, CanonicalObjectPresentation,
    CanonicalObjectResolution, ObjectSourceNamespace, ObjectTargetFeedbackKind, RegionCoord,
    TerrainPosition,
};
use reference_host::HostElapsedSample;

fn identity(id: u32) -> CanonicalObjectIdentity {
    CanonicalObjectIdentity {
        source_namespace: ObjectSourceNamespace::from_bytes([3; 32]),
        region: RegionCoord::ZERO,
        authored_local_id: id,
    }
}

fn target(id: u32, available: bool) -> interaction::Target {
    interaction::Target {
        identity: identity(id),
        available,
    }
}

fn object(id: u32, x_q9: i32) -> CanonicalObject {
    CanonicalObject {
        identity: identity(id),
        position: [x_q9 as f32 / 512.0, 0.0, 0.0],
        height: 1.0,
        presentation: CanonicalObjectPresentation::static_object(0, 0, 0),
    }
}

fn origin() -> TerrainPosition {
    TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap()
}

#[test]
fn intent_lifetime_is_bounded() {
    let mut policy = interaction::Policy::new();
    policy.ingest(true);
    policy.ingest(true);
    assert!(policy.status().pending);
    assert!(
        policy
            .prepare_after_advance(0, origin(), None, None)
            .unwrap()
            .is_none()
    );
    policy.observe_sample(HostElapsedSample::Stalled {
        elapsed_nanoseconds: 125_000_001,
        maximum_elapsed_nanoseconds: 125_000_000,
    });
    assert!(policy.status().pending);
    policy.observe_sample(HostElapsedSample::Reset);
    assert!(!policy.status().pending);
    policy.ingest(true);
    policy.observe_sample(HostElapsedSample::Suspended);
    assert!(!policy.status().pending);
}

#[test]
fn ineligible_attempts_consume_commit() {
    for (target, resolution, expected) in [
        (None, None, interaction::Ineligible::MissingTarget),
        (
            Some(target(7, false)),
            None,
            interaction::Ineligible::UnavailableTarget,
        ),
        (
            Some(target(7, true)),
            Some(CanonicalObjectResolution::Resolved(object(7, 513))),
            interaction::Ineligible::OutsideRadius,
        ),
        (
            Some(target(7, true)),
            Some(CanonicalObjectResolution::SourceReplaced),
            interaction::Ineligible::SourceReplaced,
        ),
    ] {
        let mut policy = interaction::Policy::new();
        policy.ingest(true);
        assert_eq!(
            policy
                .prepare_after_advance(1, origin(), target, resolution)
                .unwrap(),
            Some(interaction::Attempt::Ineligible(expected))
        );
        assert!(!policy.status().pending);
        assert_eq!(policy.status().ineligible_count, 1);
    }
}

#[test]
fn malformed_resolution_rolls_back() {
    let mut policy = interaction::Policy::new();
    policy.ingest(true);
    let before = policy.status();
    assert!(
        policy
            .prepare_after_advance(
                1,
                origin(),
                Some(target(7, true)),
                Some(CanonicalObjectResolution::Resolved(object(8, 0))),
            )
            .is_err()
    );
    assert_eq!(policy.status(), before);
}

#[test]
fn projected_candidate_commits_twelve_frames() {
    let mut policy = interaction::Policy::new();
    policy.ingest(true);
    let attempt = policy
        .prepare_after_advance(
            1,
            origin(),
            Some(target(7, true)),
            Some(CanonicalObjectResolution::Resolved(object(7, 512))),
        )
        .unwrap();
    let submitted = policy.frame_feedback(Some(identity(7)), attempt);
    assert_eq!(submitted.unwrap().kind, ObjectTargetFeedbackKind::Activated);
    let completion = policy
        .complete_frame(attempt, submitted, submitted)
        .unwrap()
        .unwrap();
    assert!(completion.applied);
    assert_eq!(policy.status().committed_count, 1);
    assert_eq!(
        policy.status().acknowledgement.unwrap().remaining_frames,
        interaction::ACKNOWLEDGEMENT_FRAME_COUNT - 1
    );
    for _ in 1..interaction::ACKNOWLEDGEMENT_FRAME_COUNT {
        let submitted = policy.frame_feedback(Some(identity(7)), None);
        policy.complete_frame(None, submitted, submitted).unwrap();
    }
    assert_eq!(policy.status().acknowledgement, None);
}

#[test]
fn projection_and_target_change() {
    let mut policy = interaction::Policy::new();
    policy.ingest(true);
    let attempt = policy
        .prepare_after_advance(
            1,
            origin(),
            Some(target(7, true)),
            Some(CanonicalObjectResolution::Resolved(object(7, 0))),
        )
        .unwrap();
    let submitted = policy.frame_feedback(Some(identity(7)), attempt);
    let completion = policy
        .complete_frame(attempt, submitted, None)
        .unwrap()
        .unwrap();
    assert!(!completion.applied);
    assert_eq!(policy.status().committed_count, 0);
    assert_eq!(policy.status().acknowledgement, None);

    policy.ingest(true);
    let attempt = policy
        .prepare_after_advance(
            1,
            origin(),
            Some(target(7, true)),
            Some(CanonicalObjectResolution::Resolved(object(7, 0))),
        )
        .unwrap();
    let submitted = policy.frame_feedback(Some(identity(7)), attempt);
    policy
        .complete_frame(attempt, submitted, submitted)
        .unwrap();
    policy.observe_target(Some(target(8, true)));
    assert_eq!(policy.status().acknowledgement, None);
}
