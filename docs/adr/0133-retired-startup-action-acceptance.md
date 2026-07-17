# ADR 0133: Retired Startup Action Acceptance

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0130 Retired Startup Action Acceptance

## Context

Acceptance still started helper processes before Prototype children, selected the next matching
window with an implicit PID-zero request, and attempted to prove action state in the first live
readiness value. Experiment 0129 established that queue-before-window-thread-resume cannot prove
message-pump-before-current-frame ordering. Removing redundant warm-up processes exposed the same
race in camera re-press readiness. Delayed keys also relied on one scheduler sleep even though the
evidence contract required a strict lower bound.

## Decision

- Delete every readiness-only action process and action capability from `capturedReady`.
- Delete `StartupInput`, startup request dispatch, next-window/PID-zero selection,
  `startupNativeInput`, and action-only expected command constants.
- Establish idle readiness and the exact child PID before every maintained native action.
- Keep the existing two-value session contract and prove final behavior through completion,
  exact native messages, and focused product tests.
- Use Stopwatch deadlines for every positive delayed key and delayed Escape.
- Keep schema 4, exact-window/thread atomic batches, authored key order, and current timing bounds.

## Consequences

- Acceptance has one explicit product-before-action boundary and no pre-child action transport.
- Plain forced-readiness processes remain only for current baseline/restart/source fixtures and
  contain no native-input field.
- Camera, Run transition, opposing-axis, diagonal, Jump, object, focus, and exit gates retain their
  current completion authority without an event stream or intermediate product report.
- Helper preparation remains an implementation detail for explicit-PID actions; it is no longer a
  child-launch handshake or next-window selector.
- Product behavior, Runtime, renderer/GPU resources, source formats, and synchronization are
  unchanged.

## Evidence

The first post-deletion full run observed camera re-press readiness at orbit 0 instead of the old
startup gate's expected orbit 1. The next run rejected a 199.2574 ms delayed key against an authored
200 ms lower bound. Neither failure was retried into acceptance; the obsolete boundary and
scheduler-sleep timing were removed directly.

Final `canonical-prototype-v45` passed in 144.642 seconds with all prior gates. Post-readiness camera
repeat/re-press ended at exact orbits 1/2, Run release/re-press ended as exact Walk/Run, opposed
locomotion completed 12 Run steps, and diagonal Walk/Run completed 13/13 exact normalized steps.
The report shrank by 77,834 bytes and the workflow by 18.789 seconds relative to v44. All focused
Rust tests passed and the product/Runtime/GPU diff was empty.
