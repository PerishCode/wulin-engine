# ADR 0092: Committed Prototype Jump Intent

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0089 Committed Prototype Jump Intent

## Context

The host exposes sample-scoped press edges, while simulation may emit zero steps or be blocked by a
pending render window. A Space edge cannot be sent directly as an impulse command without being
lost on fractional work, and it cannot be consumed before the actor/schedule transaction commits.

Experiments 0087 and 0088 now provide the required engine mechanisms: one nonzero-batch velocity
delta and one exact committed grounded witness. The remaining lifetime and eligibility decisions
belong to the prototype application.

## Decision

- Prototype owns one capacity-one Jump policy with pending and grounded booleans. Grounded starts
  true because the existing spawn is derived from an exact committed terrain query.
- A Space press sets pending only while grounded. Duplicate/held/mid-air input adds nothing.
- Pending selects a fixed velocity delta of 4369 Q16-per-step, the nearest 4 m/s encoding at 60 Hz.
- Prototype observes activation/time discontinuity before admitting the current normalized Space
  edge. Reset/Suspended therefore clears old pending intent without deleting a new post-transition
  action from the same native message batch.
- Ready zero-step, Stalled, and typed render-block outcomes retain pending. No elapsed backlog is
  retained. Reset and Suspended clear pending.
- A successful nonzero actor advance clears pending and replaces grounded with the exact committed
  final-step witness. Invalid witness shapes fail without policy mutation.
- Keep existing Survey/Walk selection. Add no jump clip, engine action/input state, compatibility
  route, secondary simulation transaction, or configurable control/tuning surface.

## Consequences

- A short Space press has bounded lifetime until one eligible fixed-step commit instead of being
  tied to a render-loop sample.
- Render backpressure can delay one action intent but cannot commit it early or create a queue.
- Focus/elapsed discontinuity drops pending action explicitly; a stall drops elapsed time but keeps
  the one boolean intent.
- Mid-air repeats, coyote time, jump animation, root motion, air-control changes, configurable
  physics, gameplay effects, multiplayer authority, and Wulin policy remain later decisions.

## Evidence

Experiment 0089 passes 83 engine-runtime, 22 prototype, and 20 reference-host tests. Focused policy
tests cover duplicate/mid-air rejection, zero-step/block/stall retention, discontinuity
cancellation, nonzero consumption, landing re-eligibility, and malformed-witness rollback.

`canonical-prototype-v14` passes in 67.470 seconds. A visible-window native Space/VK 32 process
commits delta 4369 for one step under gravity -179: exact velocity changes `0 -> 4190`, exact center
height changes `141824 -> 146014`, and the committed witness is false. Pending clears, grounded
becomes false, XZ/identity/Survey presentation/camera/traversal remain exact, and render-block count
is zero. All previous real processes retain zero delta and grounded policy state. Init and guard
pass. No engine, renderer/GPU/resource/synchronization/source/lifecycle/format/asset, or Wulin
boundary changed, so the long canonical runtime workflow was not required.
