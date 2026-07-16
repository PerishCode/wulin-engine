# ADR 0090: Transactional Actor Vertical Impulse

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0087 Transactional Actor Vertical Impulse

## Context

The retained actor transaction already composes explicit elapsed time, bounded fixed steps,
horizontal translation, per-step vertical acceleration/contact, presentation, canonical render
admission, and atomic schedule/actor commit. It cannot express a one-time change to vertical state
at the boundary of an emitted fixed-step batch without an independent actor mutation route.

Prototype Run is already expressible as product selection of existing horizontal/presentation
values. A complete jump would additionally decide grounded eligibility, input intent lifetime,
retry, tuning, and presentation. Gameplay-object interaction first needs a separate CPU authority.
The missing foundation selected here is therefore a generic transaction input, not a product
action.

## Decision

- Add one required `initial_step_velocity_delta_q16` to `ActorSimulationCommand` and the strict
  workbench payload, replacing the live schema in place.
- If the prepared schedule emits at least one fixed step, checked-add the delta to the retained
  vertical step velocity once before the first planar-first step. Existing per-step acceleration
  and contact then execute unchanged.
- If no step is emitted, apply nothing and preserve the complete actor. Commands are caller-owned
  values, not queued intent; later resubmission is explicit caller policy.
- Arithmetic, terrain-query, published-window, and pending-window failures keep the existing atomic
  rollback. A pending-only block may report prepared step/query counts but commits no candidate.
- Prototype and all behavior-neutral callers explicitly provide zero. Add no default, optional
  field, alias, old revision, compatibility adapter, independent actor mutation, or jump API.

## Consequences

- A later prototype action can request a fixed-step vertical transition without bypassing the sole
  actor/schedule/render transaction.
- Exactly-once semantics are scoped to one nonzero prepared batch. They do not imply persistent
  action intent across fractional elapsed time or backpressure.
- Grounded eligibility, jump input buffering/repeat/retry, impulse tuning, animation, Run dynamics,
  gameplay objects, and Wulin policy remain later decisions.
- The engine command surface changes deliberately in place; every maintained caller must state its
  delta explicitly.

## Evidence

Experiment 0087 passes 81 focused engine-runtime tests. Exact private evidence proves zero-step
non-consumption, delta-before-acceleration ordering, once-only multi-step application, equivalent
zero-delta continuation, and pre-query overflow rollback.

The final typed-command `canonical-actor-v5` passes in 56.174 seconds. It rejects both the
missing-field old request and an invented alias, commits exact velocity `0 -> 16384` and center
`141824 -> 158208` for one admitted
step, and prepares but does not commit a delta-8192 pending-window candidate. Existing actor,
schedule, pending composition, retained frame, GPU, and animation evidence remains exact.

`canonical-prototype-v12` passes in 81.960 seconds with 81 runtime, 18 prototype, and 20 host tests;
all product witnesses explicitly carry zero delta and preserve existing behavior. Init and guard
pass. The initial guard rejected an eight-scalar private motion signature; one private typed
`MotionBatchCommand` now carries that projection without a lint exception or public API. No
renderer, GPU resource, synchronization, source lifetime, lifecycle, format, asset, or Wulin
implementation changed, so the long canonical runtime workflow was not required.
