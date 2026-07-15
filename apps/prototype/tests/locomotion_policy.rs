#[path = "../src/locomotion.rs"]
mod locomotion;

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
fn opposing_axes_cancel_independently() {
    assert_eq!(command(&[0x41, 0x44]), expected(0, 0));
    assert_eq!(command(&[0x57, 0x53]), expected(0, 0));
    assert_eq!(command(&[0x41, 0x44, 0x57]), expected(0, -32));
    assert_eq!(command(&[0x57, 0x53, 0x44]), expected(32, 0));
    assert_eq!(command(&[0x41, 0x44, 0x57, 0x53]), expected(0, 0));
}

#[test]
fn focus_loss_clears_motion_and_irrelevant_keys_do_not_change_it() {
    assert_eq!(command(&[0x20, 0x31, 0x70]), expected(0, 0));

    let mut input = HostInput::new();
    input.ingest(vec![key(0x57, true), key(0x20, true)]);
    assert_eq!(locomotion::command(&input), expected(0, -32));
    input.ingest(vec![NativeMessage::FocusLost]);
    assert_eq!(locomotion::command(&input), expected(0, 0));
}
