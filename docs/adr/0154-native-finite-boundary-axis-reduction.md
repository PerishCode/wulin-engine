# ADR 0154: Native Finite-Boundary Axis Reduction

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0151 Native Finite-Boundary Axis Reduction

## Context

The maintained boundary process already proved a 15-second cardinal Run endpoint and graceful
completion. Exact independent per-axis reduction remained owned only by pure product tests: the
real process did not show that one safe component continues after the other component's
maximum-eight-step candidate becomes unsafe.

## Decision

- Reuse the sole exact-PID boundary child, atomic Shift/W start, 15,000 ms hold, and existing
  readiness/completion values.
- After reaching the negative-Z edge, post A-down for 500 ms, release A/W/Shift, wait 250 ms, and
  post Escape through the current schema-4 helper on the same window.
- Require final X to encode 16..=48 negative 45-Q9 Run steps while final Z remains in inclusive
  `[-4096,-3648]`.
- Treat nine as the maximum possible X/Z-coupled step count across every 1..=8 batch partition;
  require the derived tangential-only count to remain at least seven.
- Require final Survey/yaw 32,768, stable actor identity/region/shape, zero vertical velocity,
  continuous clock/frame progress, idle object policy, zero render blocks, and clean completion.
- Add no product code/output, intermediate state, polling, retry, process, compatibility route, or
  relaxed boundary.

## Consequences

- The real process now complements the pure boundary tests with a native end-to-end witness of
  independent per-axis admission.
- The final Z value need not retain the earlier 64-Q9 cardinal lattice because a bounded prefix of
  the later diagonal Run may still be admitted before Z becomes unsafe.
- Final stationary facing follows the admitted tangential axis rather than the blocked raw
  diagonal request.
- Acceptance cost and process count remain structurally fixed; the existing long child performs
  the additional bounded phase.

## Evidence

`canonical-prototype-v66` passed in 170.657 seconds with a 457,402-byte report. PID 2188 used window
`31526174` and thread 27276. Atomic Shift/W spanned 0.0013 ms and remained held for 15,012.6987 ms;
the same window held A for 506.4753 ms, released A/W/Shift across 2.3234/0.0528 ms, and received
Escape 263.6341 ms later.

The actor finished at local `(-1395,-3738)` Q9 with generation/region/shape unchanged and zero
vertical velocity. Thirty-one exact X steps minus the nine-step maximum coupled prefix prove at
least 22 tangential-only commits. Completion reported Survey clip 0/yaw 32,768, epoch 1,065,
Ready/sample `1079/1080`, 1,080 live frames, zero stalls/render blocks/object effects, exit zero,
exactly two values, and empty stderr/trailing output.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor remained at zero
denies and five existing warnings; product, Runtime, renderer/GPU, source, resource,
synchronization, schema, and process count were unchanged.
