# ADR 0136: Native Focus Jump Suppression

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0133 Native Focus Jump Suppression

## Context

The maintained focus-discontinuity session proved held-W cleanup and no-backlog recovery, while
separate Jump sessions proved ordinary grounded admission and midair rejection. It did not prove
that a Space edge queued in the same native batch immediately before focus loss could not cross the
activation/time discontinuity and reach resumed simulation.

## Decision

- Replace the W-only focus-loss helper with an exact-PID atomic Space/W/focus-loss batch on the
  visible window thread.
- Begin only after grounded idle readiness, retain the existing bounded suspended/resumed dwell,
  and exit through the existing two-value Escape completion.
- Require exactly one suspend/resume pair, one post-resume reset, increased suspended samples,
  later Ready progress, zero backlog/stalls/blocks, and final actor state exactly equal to readiness.
- Interpret the evidence as action suppression across Suspended/Reset before resumed nonzero work;
  do not claim that `HostInput` immediately deletes the edge in the original ingest.
- Delete the old W-only helper directly and forbid both it and the rejected temporary helper name.

## Consequences

- Same-batch Space and W now exercise action and held-input discontinuity handling in the existing
  focus process without adding a child or product output.
- The later Ready counter and unchanged actor exclude a false pass caused by never recovering from
  suspension.
- Product input, Jump, locomotion, activation/clock ordering, session schema, Runtime,
  renderer/GPU resources, source formats, synchronization, and process count remain unchanged.
- No compatibility alias, event history, intermediate telemetry, retry, or relaxed threshold is
  introduced.

## Evidence

The first focused guard rejected a five-word helper name under the existing Flavor policy. The
helper was renamed to `suspendWithActionBatch` without an exception.

Final `canonical-prototype-v48` passed in 144.949 seconds. PID 2252 / thread 20452 atomically posted
Space-down and W-down 0.0012 ms apart before focus loss; the whole batch span was 0.0012 ms. The
clock recorded exactly one suspend/resume pair, one additional reset, 740 suspended samples, 1,206
Ready samples, and 1,948 live frames. Final actor state remained exactly readiness with zero stalls,
elapsed backlog, or render blocks. All 103 engine-runtime, 45 Prototype, and 20 reference-host tests
passed; Flavor remained at zero denies and five existing warnings, and the
product/Runtime/GPU/source/resource diff was empty.
