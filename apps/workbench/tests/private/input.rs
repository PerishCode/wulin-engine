use super::*;

fn key(key: usize, down: bool) -> NativeMessage {
    NativeMessage::Key { key, down }
}

#[test]
fn normalization_suppresses_duplicates_and_releases_focus_in_key_order() {
    let mut input = HostInput::new();
    input.start_recording().unwrap();
    input.ingest(vec![
        key(87, true),
        key(87, true),
        key(65, false),
        key(65, true),
        key(68, true),
        NativeMessage::FocusLost,
        key(68, false),
        key(0, true),
        key(256, true),
    ]);
    let summary = input.stop_recording().unwrap();
    let recording = input.completed_recording.as_ref().unwrap();
    let transitions = &recording.transactions[0].transitions;

    assert_eq!(held_key_list(&input.held), Vec::<u8>::new());
    assert_eq!(summary["transactionCount"], 1);
    assert_eq!(summary["rawMessageCount"], 9);
    assert_eq!(summary["transitionCount"], 6);
    assert_eq!(summary["invalidKeyCount"], 2);
    assert_eq!(summary["repeatedDownCount"], 1);
    assert_eq!(summary["unmatchedUpCount"], 2);
    assert_eq!(summary["focusReleaseCount"], 3);
    assert_eq!(
        transitions,
        &[
            KeyTransition {
                key: 87,
                down: true,
            },
            KeyTransition {
                key: 65,
                down: true,
            },
            KeyTransition {
                key: 68,
                down: true,
            },
            KeyTransition {
                key: 65,
                down: false,
            },
            KeyTransition {
                key: 68,
                down: false,
            },
            KeyTransition {
                key: 87,
                down: false,
            },
        ]
    );
}

#[test]
fn replay_starts_from_recorded_state_and_never_mutates_live_state() {
    let mut input = HostInput::new();
    input.ingest(vec![key(32, true)]);
    input.start_recording().unwrap();
    input.ingest(vec![key(32, false), key(65, true)]);
    let completed = input.stop_recording().unwrap();
    input.ingest(vec![key(66, true)]);
    let live_before = input.held;

    let replay = input.replay().unwrap();

    assert_eq!(input.held, live_before);
    assert_eq!(replay["matchesRecord"], true);
    assert_eq!(replay["liveStateUnchanged"], true);
    assert_eq!(replay["initialHeldKeys"], json!([32]));
    assert_eq!(replay["finalHeldKeys"], json!([65]));
    assert_eq!(replay["streamSha256"], completed["streamSha256"]);
}

#[test]
fn equivalent_records_have_identical_canonical_hashes() {
    fn record() -> Value {
        let mut input = HostInput::new();
        input.start_recording().unwrap();
        input.ingest(vec![key(87, true), key(87, true)]);
        input.ingest(vec![key(87, false), NativeMessage::FocusLost]);
        input.stop_recording().unwrap()
    }

    let first = record();
    let second = record();
    assert_eq!(first["streamSha256"], second["streamSha256"]);
    assert_eq!(
        first["finalHeldStateSha256"],
        second["finalHeldStateSha256"]
    );
}

#[test]
fn record_overflow_is_explicit_and_preserves_last_completed_record() {
    let mut input = HostInput::with_limits(1, 2);
    input.start_recording().unwrap();
    input.ingest(vec![key(65, true), key(65, false)]);
    let first = input.stop_recording().unwrap();

    input.start_recording().unwrap();
    input.ingest(vec![key(66, true)]);
    input.ingest(vec![key(66, false)]);

    assert!(input.active_recording.is_none());
    assert!(input.recording_fault.is_some());
    assert!(
        input
            .stop_recording()
            .unwrap_err()
            .to_string()
            .contains("exceeded")
    );
    let replay = input.replay().unwrap();
    assert_eq!(replay["streamSha256"], first["streamSha256"]);
    assert_eq!(held_key_list(&input.held), Vec::<u8>::new());
}

#[test]
fn invalid_record_lifecycle_operations_do_not_change_state() {
    let mut input = HostInput::new();
    let initial = input.held;
    assert!(input.stop_recording().is_err());
    assert!(input.replay().is_err());
    input.start_recording().unwrap();
    assert!(input.start_recording().is_err());
    assert!(input.replay().is_err());
    assert_eq!(input.held, initial);
}
