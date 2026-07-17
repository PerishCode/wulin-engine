# ADR 0147: Exact Finite-Boundary Run Completion

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0144 Exact Finite-Boundary Run Completion

## Context

The maintained one-region boundary process proved exact post-readiness Shift/W ownership and
15-second liveness, but intentionally ended by evidence-owned process termination. It therefore
could not claim the actor's final position, presentation, clock, or object state even though the
product already owned a bounded graceful completion contract.

## Decision

- Keep the sole boundary child, exact-PID atomic Shift/W, and 15,000 ms monotonic hold.
- After the hold, post the existing Escape action to the same PID/window and consume the standard
  sequence-2 completion.
- Replace the custom spawn/read/kill implementation with the maintained graceful session owner.
- Require stable actor identity/region/shape, 64-Q9 Run quantization, an inclusive final local-Z
  band of `[-4096,-3648]`, zero local X/vertical velocity, Survey clip 0/yaw 49,152, a later
  presentation epoch, clock/frame progress, idle object state, and zero stalls/render blocks.
- Preserve the native helper's 1,000 ms delayed-exit maximum; do not move the long boundary hold
  into that shared permission.
- Delete `boundarySurvival` and `holdPrototypeBoundaryRun` without an alias or fallback.
- Add no product state/output type, process, retry, telemetry, Runtime behavior, or
  renderer/GPU/source/resource/synchronization ownership.

## Consequences

- The real process now observes the same conservative maximum-eight-step boundary policy previously
  owned only by pure tests.
- Variable host batch partition may stop at any 64-Q9 point from -3,648 through -4,096; that exact
  finite set is the contract, not one timing-dependent coordinate.
- The actor's final Survey presentation retains the last committed negative-Z facing after Run is
  reduced to zero.
- Normal boundary evidence now has the same two-value graceful lifetime as other maintained
  sessions; forced termination remains only for readiness-only evidence.

## Evidence

`canonical-prototype-v59` passed in 175.188 seconds with a 454,446-byte report. PID 30632 received
atomic Shift/W on thread 21468 with a 0.0019 ms span, held it for 15,014.8593 ms, and received
Escape on the same window. Completion reported exit code zero and exactly two values.

The actor retained generation/region/shape, committed 57 exact Run steps, and stopped at local
`(0,-3648)` Q9 with zero vertical velocity, Survey clip 0/yaw 49,152, and epoch 110. Ready/sample
counts reached `1007/1008`; 1,008 live frames had zero render blocks, the clock had zero
stall/suspend/resume, and object state remained idle.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor remained at zero
denies and five existing warnings. The shared 1,000 ms helper bound and every product/Runtime/GPU
source remained unchanged.
