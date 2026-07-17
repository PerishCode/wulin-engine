# ADR 0146: Native Object Focus Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0143 Native Object Focus Readmission

## Context

The maintained focus process proved that F/Enter edges immediately before focus loss cannot cross
the activation/time discontinuity. The separate Activated object-feedback process proved an exact
post-readiness object action, but did not prove that focus cleanup leaves the observation and
interaction policies ready to admit a later fresh F/Enter lifetime.

## Decision

- Reuse the existing Activated object-feedback process.
- Atomically post F-down, Enter-down, and focus loss on the same visible PID/window thread.
- Resume focus, wait for the existing reset/recovery boundary, and retain the existing atomic fresh
  F/Enter plus delayed-Escape sequence.
- Require exactly one suspend/resume/reset boundary, zero stale object-policy effects, one exact
  fresh Activated commit, and the existing 12-frame acknowledgement and consumed-identity oracle.
- Keep the Rejected object-feedback and main focus-discontinuity processes unchanged.
- Add no product action/input state, queue, report field, compatibility path, process, Runtime
  behavior, or renderer/GPU/source/resource/synchronization ownership.

## Consequences

- Focus loss is proven to cancel pre-loss F observation and Enter activation intents.
- Later normalized F/Enter presses are proven to observe and activate the exact object after clock
  recovery.
- The stronger proof reuses the existing Activated child and exact two-value session rather than
  creating another process or diagnostic output.

## Evidence

`canonical-prototype-v58` passed on its first run in 169.078 seconds with a 448,810-byte report.
PID 13320 atomically posted F/Enter/focus-loss on thread 22076 with a 0.0013 ms batch span. After
recovery, fresh F/Enter used the same window/thread and 0.0013 ms span, then preceded Escape by
275.2869 ms.

The stationary actor retained generation 1 and local `(0,0)` Q9. The exact source oracle recorded
one committed Activated action, zero ineligible attempts, 12 Activated frames, no pending state,
and consumed authored local ID 496 under the exact source namespace and owner region. The clock
recorded exactly one suspend, one resume, and one post-resume reset; suspended samples advanced
`0 -> 83`, Ready/sample counts advanced `2/3 -> 179/264`, and stall/render-block counts remained
zero. The process emitted exactly two values.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor remained at zero
denies and five existing warnings.
