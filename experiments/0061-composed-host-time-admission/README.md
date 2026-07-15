# Experiment 0061: Composed Host Time Admission

Status: Accepted

## Hypothesis

The reference host can apply one ordered bounded activation batch before one monotonic clock sample
as a single checked state transition, so every focus interruption drops the stale baseline and no
elapsed time crosses suspension, without application-loop, Runtime, frame, input, or gameplay
coupling.

## Scope

Make activation-aware sampling the sole public `HostClock` transition. Apply `HostActivation`
values in order, sample exactly once, and commit the candidate clock only after the complete
operation succeeds. Remove public independent `suspend` and `resume` controls.

Do not change Win32 reduction, connect either application loop, call Runtime, create a retained
body, map input to motion, add an inspect command, or bind simulation to rendering.

## Workload

1. Start from the reducer's initial suspended state and require a suspended sample with no baseline.
2. Resume and require reset before any elapsed output, then preserve a nominal exact delta.
3. Apply suspended/resumed in one drain after a long gap and require reset rather than catch-up.
4. Apply resumed/suspended in one drain and require the clock to remain suspended.
5. Preserve the exact 125 ms ready and 125 ms + 1 ns stalled boundaries after composition.
6. Drive a controlled monotonic regression and require byte-identical rollback.
7. Replay a fixed activation/sample sequence twice and require identical outcomes/status/hash.
8. Run focused reference-host tests, `runseal :init`, and `runseal :guard`; do not run process/GPU
   workflows because the composed policy still has no live consumer.

## Controlled Variables

- The reducer continues to emit at most two order-equivalent transitions per drain.
- The accepted 125,000,000 ns maximum remains the sole ready/stalled boundary.
- Runtime schedule/body state, application loops, frames, input, and presentation remain unchanged.
- No elapsed value is clamped, split, queued, or accumulated while suspended.

## Metrics

- Ordered activation batch and typed sample outcome.
- Exact elapsed/max nanoseconds and complete clock status/counters.
- Regression rollback equality and deterministic replay SHA-256.
- Focused test count, Flavor denies, and guard result.

## Acceptance Criteria

- Activation order determines suspension before the same operation samples time.
- Any loss/resume interruption clears the old baseline and forces reset before ready elapsed.
- Existing exact ready/stalled/recovery behavior remains unchanged.
- Failure mutates no clock state and fixed replay evidence is byte-identical.
- No independent public clock pause mutation or live application/runtime consumer is introduced.
- Focused tests, `runseal :init`, and `runseal :guard` pass without a process or canonical workflow.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and the concrete Windows reference-host
crate. No generated process, runtime, or GPU evidence is required.

## Evidence

Six private clock tests preserve the prior exact ready/stalled boundaries and add initial
suspension, resume reset, both ordered two-transition outcomes, candidate rollback after an applied
activation followed by controlled counter overflow, monotonic-regression rollback, and deterministic
replay. Together with activation, input, and bootstrap proofs, all 21 reference-host tests pass.

A suspended/resumed batch after a controlled 60-second gap emitted `Reset`; resumed/suspended ended
`Suspended`. Exact 125,000,000 ns remained ready and 125,000,001 ns remained explicitly stalled.
Two fixed composition replays produced SHA-256
`15ab39e6b25ea2a63a97378c51f7ec73242d53d87331245174b4efffef01301e`.

Both application roots compile unchanged. `runseal :init` and `runseal :guard` passed with zero
Flavor denies. No process or canonical workflow was run because application loops, Runtime, frames,
GPU resources, synchronization, and lifecycle behavior remain unchanged.
