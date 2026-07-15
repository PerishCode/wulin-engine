use std::time::{Duration, Instant};

use engine_runtime::SIMULATION_MAX_ELAPSED_NANOSECONDS;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::*;

#[test]
fn bounded_samples_are_exact() {
    let mut clock = HostClock::new();
    let mut now = Instant::now();

    assert_eq!(clock.sample_at(&[], now).unwrap(), HostElapsedSample::Reset);
    assert_eq!(
        clock.sample_at(&[], now).unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 0
        }
    );

    for elapsed_nanoseconds in [16_666_666, 16_666_667, SIMULATION_MAX_ELAPSED_NANOSECONDS] {
        now += Duration::from_nanos(elapsed_nanoseconds);
        assert_eq!(
            clock.sample_at(&[], now).unwrap(),
            HostElapsedSample::Ready {
                elapsed_nanoseconds
            }
        );
    }

    now += Duration::from_nanos(SIMULATION_MAX_ELAPSED_NANOSECONDS + 1);
    assert_eq!(
        clock.sample_at(&[], now).unwrap(),
        HostElapsedSample::Stalled {
            elapsed_nanoseconds: SIMULATION_MAX_ELAPSED_NANOSECONDS + 1,
            maximum_elapsed_nanoseconds: SIMULATION_MAX_ELAPSED_NANOSECONDS,
        }
    );
    now += Duration::from_nanos(1);
    assert_eq!(
        clock.sample_at(&[], now).unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 1
        }
    );

    assert_eq!(
        clock.status(),
        HostClockStatus {
            suspended: false,
            has_baseline: true,
            sample_count: 7,
            reset_count: 1,
            ready_count: 5,
            stall_count: 1,
            suspended_sample_count: 0,
            suspend_count: 0,
            resume_count: 0,
        }
    );
}

#[test]
fn suspend_resume_are_idempotent() {
    let mut clock = HostClock::new();
    let base = Instant::now();

    assert_eq!(
        clock.sample_at(&[], base).unwrap(),
        HostElapsedSample::Reset
    );
    assert_eq!(
        clock
            .sample_at(&[], base + Duration::from_nanos(8))
            .unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 8
        }
    );
    assert_eq!(
        clock
            .sample_at(
                &[HostActivation::Suspended, HostActivation::Suspended],
                base + Duration::from_secs(60),
            )
            .unwrap(),
        HostElapsedSample::Suspended
    );
    assert_eq!(
        clock
            .sample_at(
                &[HostActivation::Resumed, HostActivation::Resumed],
                base + Duration::from_secs(60),
            )
            .unwrap(),
        HostElapsedSample::Reset
    );
    assert_eq!(
        clock
            .sample_at(
                &[],
                base + Duration::from_secs(60) + Duration::from_nanos(3),
            )
            .unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 3
        }
    );

    assert_eq!(
        clock.status(),
        HostClockStatus {
            suspended: false,
            has_baseline: true,
            sample_count: 5,
            reset_count: 2,
            ready_count: 2,
            stall_count: 0,
            suspended_sample_count: 1,
            suspend_count: 1,
            resume_count: 1,
        }
    );
}

#[test]
fn activation_batches_precede_sampling() {
    let mut clock = HostClock::new();
    let base = Instant::now();

    assert_eq!(
        clock.sample_at(&[HostActivation::Suspended], base).unwrap(),
        HostElapsedSample::Suspended
    );
    assert_eq!(
        clock
            .sample_at(&[HostActivation::Resumed], base + Duration::from_secs(60))
            .unwrap(),
        HostElapsedSample::Reset
    );
    assert_eq!(
        clock
            .sample_at(
                &[],
                base + Duration::from_secs(60) + Duration::from_nanos(7)
            )
            .unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 7
        }
    );

    assert_eq!(
        clock
            .sample_at(
                &[HostActivation::Suspended, HostActivation::Resumed],
                base + Duration::from_secs(120),
            )
            .unwrap(),
        HostElapsedSample::Reset
    );
    assert_eq!(
        clock
            .sample_at(
                &[HostActivation::Resumed, HostActivation::Suspended],
                base + Duration::from_secs(180),
            )
            .unwrap(),
        HostElapsedSample::Suspended
    );
}

#[test]
fn monotonic_regression_rolls_back() {
    let mut clock = HostClock::new();
    let base = Instant::now();
    clock
        .sample_at(&[], base + Duration::from_secs(10))
        .unwrap();
    let before = clock.clone();

    let error = clock
        .sample_at(&[], base + Duration::from_secs(9))
        .unwrap_err();

    assert!(error.to_string().contains("regressed"));
    assert_eq!(clock, before);
}

#[test]
fn activation_and_sample_failure_roll_back_together() {
    let mut clock = HostClock::new();
    clock.counters.sample_count = u64::MAX;
    let before = clock.clone();

    let error = clock
        .sample_at(&[HostActivation::Suspended], Instant::now())
        .unwrap_err();

    assert!(error.to_string().contains("sample count overflowed"));
    assert_eq!(clock, before);
}

#[test]
fn replay_is_exact() {
    let first = replay_evidence();
    let second = replay_evidence();
    assert_eq!(first, second);

    let first_hash = evidence_hash(&first);
    let second_hash = evidence_hash(&second);
    assert_eq!(first_hash, second_hash);
    assert_eq!(
        first_hash,
        "15ab39e6b25ea2a63a97378c51f7ec73242d53d87331245174b4efffef01301e"
    );
}

fn replay_evidence() -> Value {
    let mut clock = HostClock::new();
    let mut now = Instant::now();
    let mut outcomes = vec![clock.sample_at(&[HostActivation::Suspended], now).unwrap()];

    now += Duration::from_secs(10);
    outcomes.push(clock.sample_at(&[HostActivation::Resumed], now).unwrap());

    now += Duration::from_nanos(16_666_666);
    outcomes.push(clock.sample_at(&[], now).unwrap());
    now += Duration::from_nanos(SIMULATION_MAX_ELAPSED_NANOSECONDS + 1);
    outcomes.push(clock.sample_at(&[], now).unwrap());
    now += Duration::from_nanos(1);
    outcomes.push(clock.sample_at(&[], now).unwrap());
    now += Duration::from_secs(60);
    outcomes.push(
        clock
            .sample_at(&[HostActivation::Suspended, HostActivation::Resumed], now)
            .unwrap(),
    );
    outcomes.push(clock.sample_at(&[], now).unwrap());
    outcomes.push(
        clock
            .sample_at(&[HostActivation::Resumed, HostActivation::Suspended], now)
            .unwrap(),
    );

    json!({
        "outcomes": outcomes,
        "status": clock.status(),
    })
}

fn evidence_hash(evidence: &Value) -> String {
    let mut digest = Sha256::new();
    digest.update(b"wulin-composed-host-time-admission-v1\0");
    digest.update(serde_json::to_vec(evidence).unwrap());
    format!("{:x}", digest.finalize())
}
