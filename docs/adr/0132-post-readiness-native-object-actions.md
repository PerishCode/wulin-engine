# ADR 0132: Post-Readiness Native Object Actions

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0129 Post-Readiness Native Object Actions

## Context

Schema-4 startup transport suspends the selected visible window thread, queues an atomic input
prefix, and resumes it. Expanded acceptance showed two different object-action gates occasionally
publishing readiness without consuming F/Enter. The helper can suspend the thread after the live
loop has already returned from its current `pump_messages`; messages queued before resume are then
available only to the next frame. Atomic transport ordering therefore cannot establish
message-pump-before-current-frame ordering.

## Decision

- Establish product readiness and exact child PID before every maintained native object action.
- Post F/Enter atomically to that PID and use the existing one completion value for final action
  evidence.
- Keep stationary base and `base + 4` source fixtures and derive exact final identities through the
  independent source nearest oracle.
- Commit the sustained session's first consumption after readiness, then perform its existing
  motion and capacity-rejection action.
- Delete the `"object-action"` startup request and action-specific readiness observation and
  interaction oracle branches.
- Preserve the two-value product session schema and add no retry, event stream, product delay, or
  threshold relaxation.

## Consequences

- Activated, Rejected, and capacity-exhaustion evidence has an explicit product-before-action
  boundary rather than an inferred external-thread timing boundary.
- Readiness is required to be object-idle; completion is the sole final action-state authority.
- Exact target selection remains independently source-derived, while product reports retain only
  identity/state and no copied canonical object.
- Schema-4 atomic startup prefixes remain for workloads whose maintained contracts still require
  them, but no object-action startup alias or fallback remains.
- Product input/object/session behavior, Runtime, renderer/GPU resources, source formats, and
  synchronization remain unchanged.

## Evidence

Pre-fix full runs separately observed `consumed=null` in sustained readiness and
`object_observation_driver.completed=false` in Rejected readiness. A fixed three-run isolated
sustained sample passed, confirming timing variance. Source and product policy evidence remained
exact.

Final `canonical-prototype-v44` passed in 163.431 seconds. Activated PID 8,908 atomically posted
F/Enter in 0.0012 ms after readiness, consumed ID 496, and projected 12 Activated plus 64
suppression frames. Rejected PID 3,764 used 0.0270 ms, retained ID 495, and projected 12 Rejected
frames. Sustained PID 1,596 committed ID 496 after readiness, moved to local X 2,400, retained
exclusion-aware target ID 505, and projected 12 Activated, 12 Rejected, and 2,133 suppression
frames. All prior gates passed; Flavor reported zero denies, and the product/Runtime/GPU diff was
empty.
