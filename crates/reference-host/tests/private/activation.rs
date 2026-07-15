use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::*;

#[test]
fn initial_and_single_states_are_exact() {
    let mut state = ActivationState::new();

    assert_eq!(state.drain(), vec![HostActivation::Suspended]);
    assert!(state.drain().is_empty());
    state.record(false);
    assert!(state.drain().is_empty());

    state.record(true);
    state.record(true);
    assert_eq!(state.drain(), vec![HostActivation::Resumed]);
    state.record(false);
    state.record(false);
    assert_eq!(state.drain(), vec![HostActivation::Suspended]);
}

#[test]
fn interrupted_states_preserve_order() {
    let mut state = delivered_state(true);
    state.record(false);
    state.record(true);
    assert_eq!(
        state.drain(),
        vec![HostActivation::Suspended, HostActivation::Resumed]
    );

    let mut state = delivered_state(false);
    state.record(true);
    state.record(false);
    assert_eq!(
        state.drain(),
        vec![HostActivation::Resumed, HostActivation::Suspended]
    );
}

#[test]
fn every_burst_reduces_to_two() {
    for delivered_focused in [false, true] {
        for length in 1..=8 {
            for bits in 0..(1_u16 << length) {
                let mut state = delivered_state(delivered_focused);
                let mut native_focused = delivered_focused;
                let mut changed = false;
                for index in 0..length {
                    let focused = bits & (1 << index) != 0;
                    changed |= focused != native_focused;
                    native_focused = focused;
                    state.record(focused);
                }

                let batch = state.drain();
                assert!(batch.len() <= 2);
                assert_eq!(
                    apply(delivered_focused, &batch),
                    native_focused,
                    "delivered={delivered_focused} length={length} bits={bits}"
                );
                let expected_length = if !changed {
                    0
                } else if delivered_focused == native_focused {
                    2
                } else {
                    1
                };
                assert_eq!(batch.len(), expected_length);
                assert!(state.drain().is_empty());
            }
        }
    }
}

#[test]
fn reset_restores_initial_state() {
    let mut state = delivered_state(true);
    state.record(false);
    state.reset();

    assert_eq!(state, ActivationState::new());
    assert_eq!(state.drain(), vec![HostActivation::Suspended]);
    assert!(state.drain().is_empty());
}

#[test]
fn replay_is_exact() {
    let first = replay_evidence();
    let second = replay_evidence();
    assert_eq!(first, second);

    let first_hash = evidence_hash(&first);
    assert_eq!(first_hash, evidence_hash(&second));
    assert_eq!(
        first_hash,
        "eed23eab9230c591d895eaede20bbe19284a0bf309302b8d692ed8c1029738f1"
    );
}

fn delivered_state(focused: bool) -> ActivationState {
    let mut state = ActivationState::new();
    state.record(focused);
    let expected = if focused {
        HostActivation::Resumed
    } else {
        HostActivation::Suspended
    };
    assert_eq!(state.drain(), vec![expected]);
    state
}

fn apply(mut focused: bool, transitions: &[HostActivation]) -> bool {
    for transition in transitions {
        focused = match transition {
            HostActivation::Suspended => false,
            HostActivation::Resumed => true,
        };
    }
    focused
}

fn replay_evidence() -> Value {
    let mut state = ActivationState::new();
    let initial = state.drain();
    state.record(true);
    let resumed = state.drain();
    state.record(false);
    state.record(true);
    let interrupted_active = state.drain();
    state.record(false);
    let suspended = state.drain();
    state.record(true);
    state.record(false);
    let interrupted_suspended = state.drain();

    json!({
        "initial": initial,
        "resumed": resumed,
        "interruptedActive": interrupted_active,
        "suspended": suspended,
        "interruptedSuspended": interrupted_suspended,
        "empty": state.drain(),
    })
}

fn evidence_hash(evidence: &Value) -> String {
    let mut digest = Sha256::new();
    digest.update(b"wulin-host-activation-v1\0");
    digest.update(serde_json::to_vec(evidence).unwrap());
    format!("{:x}", digest.finalize())
}
