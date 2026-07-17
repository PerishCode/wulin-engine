# Experiment 0131: Post-Readiness Finite-Boundary Run

Status: Accepted

## Hypothesis

The maintained finite-boundary process can exercise the faster Run input without another long
process or product report. After idle readiness establishes the exact child PID, one atomic
Shift/W batch can target that visible window and the configured one-region Prototype can remain
live for at least 15 seconds without weakening strict Runtime source/query failure.

## Scope

- Replace the existing post-readiness held-W boundary action with atomic held Shift/W.
- Keep the same real process, one-region playable rectangle, 15-second duration, completion-free
  evidence cleanup, and current canonical-prototype workflow.
- Require exact PID/window ownership, ordered Shift then W messages, one window-thread atomic
  batch, and bounded native batch span.
- Delete the now-unused held-W boundary helper and reject its return through the static guard.

Prototype product code, playable bounds, input normalization, locomotion/presentation policy,
Runtime, renderer/GPU resources, source formats, synchronization, and session output are out of
scope.

## Workload

1. Preserve the existing product boundary-policy tests, including exact cardinal/diagonal Run
   maximum-batch admission and per-axis reduction.
2. Launch the real Prototype with the focused one-region playable rectangle and wait for idle
   readiness.
3. Address the exact ready process, atomically post Shift/W to its visible window thread, and
   validate the schema-4 native evidence.
4. Require the process to remain live for at least 15 seconds, emit no stderr or trailing session
   value, and terminate only through evidence-owned cleanup.
5. Run every existing Prototype session, Sidecar lifecycle, static guard, and initialization gate.

## Controlled Variables

- The boundary process still starts at the same grounded actor position and uses the same strict
  bootstrap/source/runtime composition.
- Duration, cleanup ownership, no-completion contract, and process-count budget are unchanged.
- Existing Walk/Run completion sessions and pure boundary-policy tests remain their current
  authorities.
- No retry, wait for a desired product state, intermediate output, telemetry, relaxed duration,
  fallback, or compatibility alias is admitted.

## Metrics

- Exact process/window/thread identity, native key/message order, atomic prefix length, batch span,
  key interval, held duration, process liveness, stderr/trailing output, completion publication,
  workflow duration, report bytes, test counts, Flavor findings, and process cleanup.

## Acceptance Criteria

- The boundary action occurs strictly after readiness and targets that readiness value's exact PID.
- Schema 4 records atomic Shift/W key-down messages on one positive window thread with a span in
  `[0,50]` ms and no delayed exit.
- The configured real process remains live for at least 15,000 ms and emits no application failure,
  stderr, trailing output, or completion value before evidence cleanup.
- All five product boundary-policy tests and every prior canonical-prototype gate pass.
- No second boundary process, product output, product/Runtime/GPU/source/resource change, old
  held-W helper, or historical compatibility route is introduced.

## Results

`canonical-prototype-v46` passed in 144.312 seconds. The boundary readiness established PID 17072;
the native helper targeted that exact visible process and atomically posted Shift then W on window
thread 10284. The inter-key interval and complete batch span were both 0.0013 ms, with atomic prefix
length 2 and no key or exit delay.

The process remained live for 15,005.520 ms against the 15,000 ms minimum. It emitted no stderr,
trailing output, application failure, or completion value and was terminated only by the existing
evidence cleanup. The ignored report was 443,819 bytes. The workflow retained one boundary process
and was 0.330 seconds faster than v45, within ordinary run variance.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. The five focused boundary
tests retained exact maximum-batch contact, per-axis reduction, signed coordinates, and failure
semantics; the Run case admitted exact 64-Q9 cardinal and 45-Q9 diagonal components and cleared or
reduced unsafe axes. `runseal :guard` passed with zero Flavor denies and five existing warnings.
No Rust product, Runtime, renderer/GPU, source, resource, or synchronization code changed.

The real-process result establishes input ownership and bounded liveness under a finite
configuration. Because the product intentionally publishes no post-action intermediate state, it
does not claim an observed final actor position or continuous Run presentation; the pure product
tests remain the authority for exact boundary reduction.

## Conclusion

Accepted. The sole finite-boundary process now receives exact-PID atomic Shift/W only after idle
readiness and remains live for the existing 15-second gate. The older W-only helper is removed, the
workflow adds no process or product surface, and exact Run boundary behavior remains separately
owned by product tests.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
