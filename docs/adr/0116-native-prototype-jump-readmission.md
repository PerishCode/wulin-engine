# ADR 0116: Native Prototype Jump Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0113 Native Prototype Jump Readmission

## Context

The accepted Jump policy uses one grounded bit derived only from the last committed contact
witness. Focused tests prove landing re-eligibility, while the real native process currently proves
only the first committed step after one Space press. The bounded session completion now exposes
enough final actor state to prove a complete landing followed by a second action without adding
product telemetry.

## Decision

- Maintain one real-process post-readiness session that posts Space, waits beyond the complete
  fixed flight with no host stall, then posts Space release/press and exits before the second
  flight lands.
- Derive the second-flight step count only from exact final vertical velocity and require the
  existing discrete impulse/gravity height equation against the readiness ground body.
- Keep input evidence and wall-time bounds in acceptance support. Add no Jump status to completion,
  event stream, inspect route, input history, or product clock/schedule field.

## Consequences

- Grounded re-eligibility gains one live native proof rather than relying only on policy-unit
  evidence.
- The harness gains one Space re-press action; product input and Jump policy remain unchanged.
- This decision does not authorize jump animation, coyote time, configurable physics, retained
  history, gameplay effects, networking, or Runtime/GPU/resource changes.

## Evidence

Experiment 0113 passed `canonical-prototype-v30` in 100.135 seconds. One exact visible process
window received the first Space down, ran for a measured lower bound of 1,265.727 ms with zero
stalls, then received Space up/down and Escape after an exact same-helper 104.278 ms post-to-post
interval.
Readiness grounded state was true. Final motion derived exactly seven second-flight steps:
velocity 3,116, rise 25,571, and center `141824 -> 167395`, with unchanged actor identity, X/Z,
shape, presentation, and epoch. Clock reset/suspend/resume/stall counts did not change, object
policies stayed idle, render blocks stayed zero, and the existing two-value session exited cleanly.
