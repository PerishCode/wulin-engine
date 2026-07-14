use super::*;

#[test]
fn auto_commit_samples_tick() {
    let mut timeline = PresentationTimeline::new();
    let sampled = timeline.tick();

    timeline.commit_canonical_frame();

    assert_eq!(sampled, 0);
    assert_eq!(timeline.tick(), 1);
    assert_eq!(timeline.status_json()["automaticAdvanceCount"], 1);
}

#[test]
fn paused_manual_wrap() {
    let mut timeline = PresentationTimeline::new();
    timeline.pause();
    timeline
        .set(animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD - 1)
        .unwrap();

    timeline.commit_canonical_frame();
    assert_eq!(
        timeline.tick(),
        animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD - 1
    );

    timeline.step(1).unwrap();
    assert_eq!(timeline.tick(), 0);
    assert_eq!(timeline.status_json()["manualStepCount"], 1);
    assert_eq!(timeline.status_json()["wrapCount"], 1);
}

#[test]
fn invalid_changes_rollback() {
    let mut timeline = PresentationTimeline::new();
    let initial = timeline.status_json();
    assert!(timeline.set(1).is_err());
    assert!(timeline.step(1).is_err());
    assert_eq!(timeline.status_json(), initial);

    timeline.pause();
    let paused = timeline.status_json();
    assert!(
        timeline
            .set(animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD)
            .is_err()
    );
    assert!(timeline.step(0).is_err());
    assert_eq!(timeline.status_json(), paused);
}
