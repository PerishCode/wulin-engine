# ADR 0134: Post-Readiness Finite-Boundary Run

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0131 Post-Readiness Finite-Boundary Run

## Context

The maintained finite-boundary process waited for readiness after Experiment 0130, but still held
only W. Product tests already owned exact Run maximum-batch and per-axis boundary reduction, while
real Run input was proven only by shorter completion sessions. Adding another 15-second process
would duplicate liveness cost, and adding post-action product output would widen the intentionally
bounded two-value session surface.

## Decision

- Replace the existing held-W boundary action with one atomic Shift/W batch after idle readiness.
- Target only the exact ready child PID and visible window.
- Require schema-4 Shift/W order, one positive window thread, atomic prefix length 2, and a batch
  span no greater than 50 ms.
- Preserve the same one-region configuration, 15-second liveness threshold, completion-free
  evidence cleanup, and single-process budget.
- Delete the W-only boundary helper and guard against its return.
- Keep pure product tests as the authority for exact Run boundary reduction; do not infer an actor
  position or presentation from process liveness.

## Consequences

- The maintained finite-boundary process now exercises the faster locomotion input without adding
  workflow duration or another product surface.
- Walk remains covered by exact post-readiness completion sessions and product locomotion tests;
  the replaced W-only boundary action is not retained as compatibility coverage.
- Product input, locomotion, presentation, playable bounds, Runtime, renderer/GPU resources, source
  formats, and synchronization remain unchanged.
- The process gate proves exact action ownership and bounded survival, not an unobserved post-action
  state.

## Evidence

`canonical-prototype-v46` passed in 144.312 seconds. PID 17072 received atomic Shift/W on window
thread 10284 with a 0.0013 ms interval/span after readiness, then remained live for 15,005.520 ms.
It emitted no stderr, trailing output, application failure, or completion value before
evidence-owned cleanup. All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed,
including the five exact boundary-policy tests. Flavor remained at zero denies and five existing
warnings, and the product/Runtime/GPU/source/resource diff was empty.
