use super::*;

fn key(key: usize, down: bool) -> NativeMessage {
    NativeMessage::Key { key, down }
}

#[test]
fn normalization_exposes_only_state_changing_edges_and_focus_cleanup() {
    let mut input = HostInput::new();
    input.ingest(vec![
        key(87, true),
        key(87, true),
        key(65, false),
        key(65, true),
        key(68, true),
        NativeMessage::FocusLost,
        key(68, false),
        key(0, true),
        key(0x145, true),
    ]);

    for key in [65, 68, 87] {
        assert!(!input.is_held(key));
        assert!(input.was_pressed(key));
        assert!(input.was_released(key));
    }
    assert!(!input.was_pressed(0));
    assert!(!input.was_released(0));
    assert!(!input.is_held(0x45));
    assert!(!input.was_pressed(0x45));
    assert!(!input.was_released(0x45));
}

#[test]
fn sample_edges_preserve_both_directions_and_final_held_state() {
    let mut input = HostInput::new();
    input.ingest(vec![key(27, true), key(27, true)]);
    assert!(input.is_held(27));
    assert!(input.was_pressed(27));
    assert!(!input.was_released(27));

    input.ingest(vec![key(27, false), key(27, true)]);
    assert!(input.is_held(27));
    assert!(input.was_pressed(27));
    assert!(input.was_released(27));
}

#[test]
fn empty_ingest_expires_edges_and_owner_has_only_fixed_state() {
    assert_eq!(
        std::mem::size_of::<HostInput>(),
        3 * std::mem::size_of::<KeyBits>()
    );
    assert!(!std::mem::needs_drop::<HostInput>());

    let mut input = HostInput::new();
    input.ingest(vec![key(32, true), key(32, false), key(87, true)]);
    assert!(input.was_pressed(32));
    assert!(input.was_released(32));
    assert!(input.is_held(87));
    assert!(input.was_pressed(87));

    input.ingest(Vec::new());

    assert!(!input.was_pressed(32));
    assert!(!input.was_released(32));
    assert!(input.is_held(87));
    assert!(!input.was_pressed(87));
    assert!(!input.was_released(87));
}
