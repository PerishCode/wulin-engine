# Experiment 0058: Bounded Host Elapsed Clock

Status: Accepted

## Hypothesis

The reference host can own a deterministic policy around monotonic wall-time samples that emits
exact bounded elapsed nanoseconds, explicitly classifies stalls, and resets across suspension without
depending on Runtime, frames, input commands, or Win32 focus transport.

## Scope

Add one `HostClock` state machine using `Instant::now` in production and the same controlled sampling
transition in private tests. Bound ready samples to the accepted 125 ms simulation maximum. Expose
typed reset/ready/stalled/suspended outcomes and typed status/counters.

Do not connect the clock to workbench/prototype loops, add focus-gained messages, call Runtime,
sample gameplay input, clamp elapsed time, queue backlog, catch up missed steps, or bind presentation.

## Workload

1. Drive reset, zero, nominal 16,666,666/16,666,667 ns, exact 125 ms, and 125 ms + 1 ns samples.
2. Require over-bound samples to emit explicit `Stalled`, advance the baseline, and allow an exact
   next sample without repeating the stall.
3. Suspend after ready samples, sample while suspended, resume, and require a fresh reset before any
   elapsed output. Repeated suspend/resume calls must be idempotent.
4. Drive a controlled monotonic regression and require an error with byte-identical clock status.
5. Replay a fixed transition sequence twice and require identical outcomes/status SHA-256.
6. Run focused reference-host tests, `runseal :init`, and `runseal :guard`. Do not run process/GPU
   workflows because this pure host policy has no live consumer.

## Controlled Variables

- `SIMULATION_MAX_ELAPSED_NANOSECONDS` remains 125,000,000 and is the sole ready/stall boundary.
- Runtime schedule/body operations, frame loops, window messages, and input journal remain unchanged.
- `Instant` supplies monotonic samples only; policy uses exact integer nanoseconds.
- Stall elapsed is explicitly reported and not converted to a ready sample.

## Metrics

- Outcome kind and exact elapsed/max nanoseconds.
- Active/suspended/baseline state plus sample/reset/ready/stall/suspended/transition counters.
- Regression rollback status and replay SHA-256.
- Focused test count, Flavor denies, and guard result.

## Acceptance Criteria

- Ready samples preserve every bounded nanosecond exactly; exact max passes and max+1 stalls.
- Stall recovery, suspension, resume reset, and idempotent suspend/resume transitions match the
  defined state machine without hidden clamp/backlog behavior.
- Regression returns no outcome and changes no clock state.
- Two deterministic replays produce identical outcomes, status, and SHA-256.
- Focused tests, `runseal :init`, and `runseal :guard` pass without a process or canonical workflow.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and Windows reference-host crate. No
generated runtime or GPU evidence is required.

## Evidence

Four private reference-host tests cover exact bounded samples, post-stall recovery, idempotent
suspend/resume and resume reset, monotonic regression rollback, and deterministic replay. All 14
reference-host tests pass.

Zero, 16,666,666 ns, 16,666,667 ns, and exact 125 ms samples were preserved byte-exactly. A
125,000,001 ns sample emitted `Stalled` with the accepted 125,000,000 ns maximum, advanced its
baseline, and the next 1 ns sample emitted `Ready`. A suspended 60-second interval accumulated
nothing; resume emitted `Reset` before a later 3 ns ready sample. Repeated suspend/resume calls
changed no counters or state, while a controlled monotonic regression returned an error with exact
clock rollback.

Two fixed replays produced identical typed outcomes/status with SHA-256
`3a873571ca7a754272eeaecb0dc7fe9d5183703e88a100a1907cc9ae8bacea7d`. `runseal :init` and
`runseal :guard` passed with zero Flavor denies. No process or canonical workflow was run because
the clock remains disconnected from Win32 transport, application loops, Runtime, and GPU/lifecycle
ownership.
