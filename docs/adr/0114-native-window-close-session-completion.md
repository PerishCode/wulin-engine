# ADR 0114: Native Window-Close Session Completion

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0111 Native Window-Close Session Completion

## Context

The bounded Prototype session contract names Escape and window close as its two graceful completion
reasons. Escape is proven through a real native process, while window close has only pure
serialization coverage. That leaves the concrete Win32 message-loop route, Runtime idle ordering,
and exact two-value process contract unproven together.

## Decision

- Treat a `WM_CLOSE` posted to the exact visible Prototype window/PID as the maintained live
  acceptance route for `window-close`.
- Apply the existing bounded session invariant unchanged: readiness sequence one, completion
  sequence two, stable process/actor identity, monotonic clock/frame state, and no trailing output.
- Keep native close evidence in the focused acceptance harness only. Do not change product
  behavior, session schema, or output cadence.
- Retain Escape as an independent graceful route and forced termination as completion-free.

## Consequences

- Both declared graceful completion reasons have concrete real-process evidence.
- The acceptance harness gains one bounded native window action, not another product control or
  diagnostic surface.
- This decision does not authorize direct `DestroyWindow`, process termination as graceful exit,
  recurring telemetry, an inspect route, retained event history, or engine/GPU/resource changes.

## Evidence

Experiment 0111 passes all 96 engine-runtime, 45 Prototype, and 20 reference-host tests plus the
repository guard with zero Flavor denies. `canonical-prototype-v28` passes in 86.089 seconds. A
visible class/title/PID-qualified window for process 21236 receives exactly one posted `WM_CLOSE`
with no activation, keys, direct destroy, or process termination; readiness live frame 5 is
followed by completion live frame/sample count 356, reason `window-close`, exit code zero, empty
stderr, exactly two stdout values, stable process/actor identity, and idle object policies.

Escape, forced-termination silence, and sustained capacity-one action/rejection/suppression gates
remain exact. No product or engine/GPU/resource source changed.
