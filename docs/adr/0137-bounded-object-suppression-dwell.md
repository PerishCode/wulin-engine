# ADR 0137: Bounded Object Suppression Dwell

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0134 Bounded Object Suppression Dwell

## Context

The post-readiness Activated session requires exactly 12 successful projected acknowledgement
frames followed by at least one suppression frame. Its helper requested Escape 200 ms after the
atomic F/Enter batch, which is also the nominal duration of twelve 60 Hz frames. Repeated failures
showed the action and acknowledgement were exact while suppression remained at zero, so prior
passes depended on scheduling beyond the requested boundary.

## Decision

- Keep the existing exact-PID F/Enter/Escape process and all semantic action oracles.
- Request Escape 250 ms after the atomic action batch.
- Require the observed action-to-Escape interval to be in `[250,750]` ms.
- Retain exactly 12 Activated frames and require at least one following suppression frame.
- Guard the new lower bound statically; add no retry or product timing behavior.

## Consequences

- The maintained gate has at least 50 ms of requested room beyond the nominal acknowledgement
  duration and no longer relies on scheduler overshoot for suppression evidence.
- The workflow remains bounded and uses the same process count and two-value product output.
- Product object policy, acknowledgement count, Runtime, renderer/GPU resources, source formats,
  synchronization, and session schema remain unchanged.
- No temporary diagnostic output or unproven focus-action extension is retained.

## Evidence

Three initial v49 runs failed with zero suppression. Temporary exact diagnostics on the third run
reported committed count 1, ineligible count 0, cleared target, 12 Activated frames, zero Rejected
frames, and zero suppression frames.

After changing only the helper/oracle dwell, final `canonical-prototype-v49` passed in 174.564
seconds. PID 28412 / thread 31632 posted F/Enter in a 0.0016 ms atomic batch and observed Escape
270.6458 ms later. Completion retained exactly 12 Activated frames and produced two suppression
frames with exact authored ID 496 consumption/exclusion, stationary actor state, idle final action
state, and zero render blocks. All 103 engine-runtime, 45 Prototype, and 20 reference-host tests
passed; Flavor remained at zero denies and five existing warnings.
