#[path = "../src/locomotion.rs"]
mod locomotion;
#[path = "../src/presentation.rs"]
mod presentation;

use reference_host::{HostInput, input::NativeMessage};

fn key(key: usize, down: bool) -> NativeMessage {
    NativeMessage::Key { key, down }
}

fn command(keys: &[usize]) -> locomotion::Command {
    let mut input = HostInput::new();
    input.ingest(keys.iter().map(|value| key(*value, true)).collect());
    locomotion::command(&input)
}

fn expected(delta_x_q9: i32, delta_z_q9: i32) -> locomotion::Command {
    locomotion::Command {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 32_768,
        running: false,
    }
}

fn expected_run(delta_x_q9: i32, delta_z_q9: i32) -> locomotion::Command {
    locomotion::Command {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 32_768,
        running: true,
    }
}

#[test]
fn zero_and_cardinal_inputs_are_exact() {
    assert_eq!(command(&[]), expected(0, 0));
    assert_eq!(command(&[0x57]), expected(0, -32));
    assert_eq!(command(&[0x41]), expected(-32, 0));
    assert_eq!(command(&[0x53]), expected(0, 32));
    assert_eq!(command(&[0x44]), expected(32, 0));
}

#[test]
fn diagonals_use_the_fixed_nearest_normalized_component() {
    assert_eq!(command(&[0x57, 0x41]), expected(-23, -23));
    assert_eq!(command(&[0x57, 0x44]), expected(23, -23));
    assert_eq!(command(&[0x53, 0x41]), expected(-23, 23));
    assert_eq!(command(&[0x53, 0x44]), expected(23, 23));
}

#[test]
fn held_shift_selects_exact_run_components_only_for_nonzero_motion() {
    assert_eq!(command(&[0x10]), expected(0, 0));
    assert_eq!(command(&[0x10, 0x57]), expected_run(0, -64));
    assert_eq!(command(&[0x10, 0x41]), expected_run(-64, 0));
    assert_eq!(command(&[0x10, 0x53, 0x44]), expected_run(45, 45));
    assert_eq!(command(&[0x10, 0x57, 0x41]), expected_run(-45, -45));
    assert_eq!(command(&[0x10, 0x57, 0x53]), expected(0, 0));
    assert_eq!(command(&[0x10, 0x41, 0x44]), expected(0, 0));
}

#[test]
fn opposing_axes_cancel_independently() {
    assert_eq!(command(&[0x41, 0x44]), expected(0, 0));
    assert_eq!(command(&[0x57, 0x53]), expected(0, 0));
    assert_eq!(command(&[0x41, 0x44, 0x57]), expected(0, -32));
    assert_eq!(command(&[0x57, 0x53, 0x44]), expected(32, 0));
    assert_eq!(command(&[0x41, 0x44, 0x57, 0x53]), expected(0, 0));
}

#[test]
fn focus_loss_clears_motion_and_irrelevant_keys_do_not_change_it() {
    assert_eq!(command(&[0x10, 0x20, 0x31, 0x70]), expected(0, 0));

    let mut input = HostInput::new();
    input.ingest(vec![key(0x10, true), key(0x57, true), key(0x20, true)]);
    assert_eq!(locomotion::command(&input), expected_run(0, -64));
    input.ingest(vec![NativeMessage::FocusLost]);
    assert_eq!(locomotion::command(&input), expected(0, 0));
}

#[test]
fn gaits_use_exact_presentation() {
    let policy = presentation::Policy::new();
    let stationary = policy.command(command(&[]));
    assert_eq!(stationary, presentation::initial());
    assert_eq!(stationary.animation_clip(), Some(0));
    assert_eq!(policy.command(command(&[0x10])), presentation::initial());

    for (keys, yaw_q16) in [
        (vec![0x44], 0),
        (vec![0x53, 0x44], 8_192),
        (vec![0x53], 16_384),
        (vec![0x53, 0x41], 24_576),
        (vec![0x41], 32_768),
        (vec![0x57, 0x41], 40_960),
        (vec![0x57], 49_152),
        (vec![0x57, 0x44], 57_344),
    ] {
        let moving = policy.command(command(&keys));
        assert_eq!(moving.animation_clip(), Some(1));
        assert_eq!(moving.archetype, 7);
        assert_eq!(moving.material, 63);
        assert_eq!(moving.yaw_q16, yaw_q16);
        assert_eq!(moving.animation_phase_offset(), Some(0));
        assert_eq!(moving.animation_variant(), Some(0));
        moving.validate().unwrap();
    }

    for (keys, yaw_q16) in [
        (vec![0x10, 0x44], 0),
        (vec![0x10, 0x53, 0x44], 8_192),
        (vec![0x10, 0x53], 16_384),
        (vec![0x10, 0x53, 0x41], 24_576),
        (vec![0x10, 0x41], 32_768),
        (vec![0x10, 0x57, 0x41], 40_960),
        (vec![0x10, 0x57], 49_152),
        (vec![0x10, 0x57, 0x44], 57_344),
    ] {
        let running = policy.command(command(&keys));
        assert_eq!(running.animation_clip(), Some(2));
        assert_eq!(running.yaw_q16, yaw_q16);
        running.validate().unwrap();
    }
}

#[test]
fn facing_observes_only_nonzero_advances_and_stationary_retains_it() {
    let mut policy = presentation::Policy::new();
    let west = policy.command(command(&[0x57]));
    assert_eq!(west.yaw_q16, 49_152);

    policy.observe_advance(0, west);
    assert_eq!(policy.command(command(&[])).yaw_q16, 0);

    policy.observe_advance(1, west);
    let stopped = policy.command(command(&[]));
    assert_eq!(stopped.yaw_q16, 49_152);
    assert_eq!(stopped.animation_clip(), Some(0));

    let opposed = policy.command(command(&[0x57, 0x53]));
    assert_eq!(opposed.yaw_q16, 49_152);
    assert_eq!(opposed.animation_clip(), Some(0));

    let east = policy.command(command(&[0x44]));
    policy.observe_advance(0, east);
    assert_eq!(policy.command(command(&[])).yaw_q16, 49_152);
    policy.observe_advance(1, east);
    assert_eq!(policy.command(command(&[])).yaw_q16, 0);
}
