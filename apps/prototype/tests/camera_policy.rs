#[path = "../src/camera.rs"]
mod camera;

use reference_host::{HostInput, input::NativeMessage};

fn key(key: usize, down: bool) -> NativeMessage {
    NativeMessage::Key { key, down }
}

fn input(messages: Vec<NativeMessage>) -> HostInput {
    let mut input = HostInput::new();
    input.ingest(messages);
    input
}

fn rig_bits(rig: camera::Rig) -> (u8, [u32; 3], [u32; 3], u32) {
    (
        rig.orbit_index,
        rig.position_offset.map(f32::to_bits),
        rig.target_offset.map(f32::to_bits),
        rig.vertical_fov_degrees.to_bits(),
    )
}

fn current(policy: &camera::Policy) -> camera::Rig {
    policy.candidate(&HostInput::new()).rig()
}

#[test]
fn exact_quarter_turns_form_one_closed_cycle() {
    let mut policy = camera::Policy::new();
    let expected = [
        (0, [9.0, 4.0, 12.0], [0.0, -1.0, -3.0]),
        (1, [12.0, 4.0, -9.0], [-3.0, -1.0, 0.0]),
        (2, [-9.0, 4.0, -12.0], [0.0, -1.0, 3.0]),
        (3, [-12.0, 4.0, 9.0], [3.0, -1.0, 0.0]),
    ];

    for (index, position, target) in expected {
        let rig = current(&policy);
        assert_eq!(rig.orbit_index, index);
        assert_eq!(
            rig.position_offset.map(f32::to_bits),
            position.map(f32::to_bits)
        );
        assert_eq!(
            rig.target_offset.map(f32::to_bits),
            target.map(f32::to_bits)
        );
        assert_eq!(rig.vertical_fov_degrees.to_bits(), 60.0_f32.to_bits());
        let clockwise = input(vec![key(0x45, true)]);
        let candidate = policy.candidate(&clockwise);
        policy.commit(candidate);
    }
    assert_eq!(
        rig_bits(current(&policy)),
        rig_bits(current(&camera::Policy::new()))
    );
}

#[test]
fn opposite_edges_cancel_and_counter_clockwise_wraps() {
    let mut policy = camera::Policy::new();
    let both = input(vec![key(0x51, true), key(0x45, true)]);
    let candidate = policy.candidate(&both);
    assert_eq!(candidate.rig().orbit_index, 0);
    policy.commit(candidate);

    let counter_clockwise = input(vec![key(0x51, true)]);
    let candidate = policy.candidate(&counter_clockwise);
    assert_eq!(candidate.rig().orbit_index, 3);
    policy.commit(candidate);
    assert_eq!(current(&policy).orbit_index, 3);
}

#[test]
fn held_key_does_not_repeat_and_uncommitted_candidate_has_no_effect() {
    let mut input = input(vec![key(0x45, true)]);
    let mut policy = camera::Policy::new();
    let candidate = policy.candidate(&input);
    assert_eq!(candidate.rig().orbit_index, 1);
    assert_eq!(current(&policy).orbit_index, 0);

    let dropped = policy.candidate(&input);
    assert_eq!(dropped.rig().orbit_index, 1);
    assert_eq!(current(&policy).orbit_index, 0);
    policy.commit(candidate);
    assert_eq!(current(&policy).orbit_index, 1);

    input.ingest(Vec::new());
    assert!(input.is_held(0x45));
    assert!(!input.was_pressed(0x45));
    let held_candidate = policy.candidate(&input);
    assert_eq!(held_candidate.rig().orbit_index, 1);
}

#[test]
fn focus_cleanup_retains_one_camera_press() {
    let mut input = input(vec![key(0x45, true), NativeMessage::FocusLost]);
    assert!(!input.is_held(0x45));
    assert!(input.was_pressed(0x45));
    assert!(input.was_released(0x45));

    let mut policy = camera::Policy::new();
    let candidate = policy.candidate(&input);
    assert_eq!(candidate.rig().orbit_index, 1);
    policy.commit(candidate);

    input.ingest(Vec::new());
    assert!(!input.was_pressed(0x45));
    assert!(!input.was_released(0x45));
    assert_eq!(policy.candidate(&input).rig().orbit_index, 1);
}
