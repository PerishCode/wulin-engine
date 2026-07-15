# ADR 0084: Actor-Local Animation Epoch

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0081 Actor-Local Animation Epoch

## Context

Actor motion and clip selection are transactional, but actor pose time is not. A runtime actor
stores only the schema-3 presentation record, so the renderer evaluates it at the same global tick
as every streamed object. Accepted prototype evidence records eight total bootstrap frames, but
only the first canonical frame advances presentation; the actor therefore spawns at tick 1 and
Survey happens to quantize to phase 0. Two prior live canonical commits put the first Walk
transition at global tick 3 / imported Walk phase 4. Correctness currently depends on that accidental
global alignment rather than a local clip lifetime.

The presentation record's authored phase offset cannot serve as a one-time epoch: one offset can
force phase zero at transition, but later sample-boundary cadence remains aligned to the unrelated
global clip cycle. Adding an independently advancing actor clock would duplicate the renderer's
time authority and require per-frame actor mutation.

## Decision

- `RuntimeActor` owns one animation epoch tick bounded by the existing presentation common period.
  Runtime spawn initializes it from the current presentation timeline.
- An admitted nonzero-step transition resets the epoch when animated/static state, rig/archetype,
  or clip identity changes. Same-stream material, yaw, phase-offset, or variation changes retain it.
  Fractional work and rejected/blocked/failing candidates preserve the complete input actor.
- The renderer computes elapsed ticks from global tick and epoch modulo the common period, evaluates
  exact source-duration local phase, and rewrites only the dynamic actor candidate's effective
  phase-offset bits for that frame. Stored actor presentation remains authored and unchanged.
- The existing shader, pose key space, 56-byte visible-record layout, two-frame upload resource,
  streamed schema-3 presentation path, and global presentation timeline remain unchanged.
- Maintained actor and prototype acceptance derive epoch/phase from existing state, GPU readback,
  and readiness frame counts. No test-only product control or telemetry is added.

## Consequences

- Actor spawn and committed clip transitions have deterministic local phase zero and exact imported
  source-duration progression while retaining one renderer clock.
- Epoch becomes part of authoritative actor state and therefore participates in spawn/read/despawn,
  transaction rollback, readiness, and replay equality.
- Streamed objects keep global phase sharing and pose reuse; actor candidates may carry a frame-
  resolved phase offset while retaining authored clip/variant semantics in runtime state.
- Blending, root motion, playback rate, Run policy, multi-actor timing, and gameplay animation state
  remain later experiments.

## Evidence

Experiment 0081 added two focused engine tests and passed `canonical-actor-v4` in 37.433 seconds:
Survey resolved phase 0 at global/epoch 42/42 and phase 1 after four ticks; Walk reset epoch 42 to
46 with phase 0 and reached phase 1 on tick 47; same-clip yaw retained epoch 46 and fractional work
rolled back. All readbacks had zero exact-field mismatch while the actor upload remained 56 bytes,
two slots / 112 bytes, one resource, and zero copies. `canonical-prototype-v8` passed in 35.284
seconds: stationary/restart retained epoch 1 and native-W atomically changed epoch 1 to 3 with
clip/yaw/Z `0/0/0 -> 1/49152/-32`. Final `runseal :guard` passed with zero Flavor denies and five
pre-existing warnings.
