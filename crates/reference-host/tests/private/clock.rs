use std::time::{Duration, Instant};

use engine_runtime::SIMULATION_MAX_ELAPSED_NANOSECONDS;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::*;

#[test]
fn bounded_samples_are_exact() {
    let mut clock = HostClock::new();
    let mut now = Instant::now();

    assert_eq!(clock.sample_at(now).unwrap(), HostElapsedSample::Reset);
    assert_eq!(
        clock.sample_at(now).unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 0
        }
    );

    for elapsed_nanoseconds in [16_666_666, 16_666_667, SIMULATION_MAX_ELAPSED_NANOSECONDS] {
        now += Duration::from_nanos(elapsed_nanoseconds);
        assert_eq!(
            clock.sample_at(now).unwrap(),
            HostElapsedSample::Ready {
                elapsed_nanoseconds
            }
        );
    }

    now += Duration::from_nanos(SIMULATION_MAX_ELAPSED_NANOSECONDS + 1);
    assert_eq!(
        clock.sample_at(now).unwrap(),
        HostElapsedSample::Stalled {
            elapsed_nanoseconds: SIMULATION_MAX_ELAPSED_NANOSECONDS + 1,
            maximum_elapsed_nanoseconds: SIMULATION_MAX_ELAPSED_NANOSECONDS,
        }
    );
    now += Duration::from_nanos(1);
    assert_eq!(
        clock.sample_at(now).unwrap(),
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

    assert_eq!(clock.sample_at(base).unwrap(), HostElapsedSample::Reset);
    assert_eq!(
        clock.sample_at(base + Duration::from_nanos(8)).unwrap(),
        HostElapsedSample::Ready {
            elapsed_nanoseconds: 8
        }
    );
    clock.suspend().unwrap();
    clock.suspend().unwrap();
    assert_eq!(
        clock.sample_at(base + Duration::from_secs(60)).unwrap(),
        HostElapsedSample::Suspended
    );
    clock.resume().unwrap();
    clock.resume().unwrap();
    assert_eq!(
        clock.sample_at(base + Duration::from_secs(60)).unwrap(),
        HostElapsedSample::Reset
    );
    assert_eq!(
        clock
            .sample_at(base + Duration::from_secs(60) + Duration::from_nanos(3))
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
fn monotonic_regression_rolls_back() {
    let mut clock = HostClock::new();
    let base = Instant::now();
    clock.sample_at(base + Duration::from_secs(10)).unwrap();
    let before = clock.clone();

    let error = clock.sample_at(base + Duration::from_secs(9)).unwrap_err();

    assert!(error.to_string().contains("regressed"));
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
        "3a873571ca7a754272eeaecb0dc7fe9d5183703e88a100a1907cc9ae8bacea7d"
    );
}

fn replay_evidence() -> Value {
    let mut clock = HostClock::new();
    let mut now = Instant::now();
    let mut outcomes = vec![clock.sample_at(now).unwrap()];

    now += Duration::from_nanos(16_666_666);
    outcomes.push(clock.sample_at(now).unwrap());
    now += Duration::from_nanos(SIMULATION_MAX_ELAPSED_NANOSECONDS + 1);
    outcomes.push(clock.sample_at(now).unwrap());
    now += Duration::from_nanos(1);
    outcomes.push(clock.sample_at(now).unwrap());
    clock.suspend().unwrap();
    now += Duration::from_secs(60);
    outcomes.push(clock.sample_at(now).unwrap());
    clock.resume().unwrap();
    outcomes.push(clock.sample_at(now).unwrap());
    outcomes.push(clock.sample_at(now).unwrap());

    json!({
        "outcomes": outcomes,
        "status": clock.status(),
    })
}

fn evidence_hash(evidence: &Value) -> String {
    let mut digest = Sha256::new();
    digest.update(b"wulin-host-elapsed-clock-v1\0");
    digest.update(serde_json::to_vec(evidence).unwrap());
    format!("{:x}", digest.finalize())
}
