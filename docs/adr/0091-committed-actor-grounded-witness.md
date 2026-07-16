# ADR 0091: Committed Actor Grounded Witness

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0088 Committed Actor Grounded Witness

## Context

The exact planar-first terrain-body step already computes whether its final contact is grounded, but
the private motion batch discards that result and the public actor transition exposes only motion
and query counts. A caller can observe zero vertical velocity, but that does not establish current
terrain contact and must not become a parallel grounded definition.

Experiment 0087 made a one-time vertical state transition expressible. A later product action still
needs exact eligibility and intent-lifetime decisions. Eligibility must be based on the committed
simulation result before any application latch or complete jump policy is selected.

## Decision

- `MotionBatch` carries `last_step_grounded: Option<bool>`, initialized to `None` and replaced with
  each successful existing `TerrainBodyAdvance.grounded` result.
- `ActorStateTransition` publishes the same option. Zero-step advanced transactions publish `None`;
  nonzero transactions publish the final fixed step's exact result.
- Only the existing successful actor/schedule transaction exposes this witness. Failures return no
  transition, and pending-window backpressure continues to expose only prepared step/query counts.
- Do not cache grounded state in `RuntimeActor`, add a terrain query, infer from velocity, or add a
  caller mutation/inspect route.
- Prototype acceptance consumes the explicit witness instead of labeling zero-velocity output as
  grounded. Product Space/jump intent remains deferred.

## Consequences

- Applications can retain one exact committed contact witness as policy input without duplicating
  terrain sampling or contact classification.
- `None` has precise temporal meaning: no fixed step occurred in that committed transaction.
- A blocked candidate cannot leak speculative contact state into application eligibility.
- Input-edge retention, eligibility lifetime, jump impulse/tuning/presentation, coyote time, Run,
  gameplay objects, sustained source service, and Wulin behavior remain later decisions.

## Evidence

Experiment 0088 passes 83 focused engine-runtime tests. Exact zero/departure/landing workloads prove
`None`, false, and true; a false→false→true three-step trajectory proves final-step replacement.
Fractional, overflow, and failure semantics remain exact.

`canonical-actor-v6` passes in 38.654 seconds with fractional null, admitted upward false, grounded
animation true, and no transition/witness on pending backpressure. Actor, schedule, composition,
retained frame, GPU, and animation evidence remains exact.

`canonical-prototype-v13` passes in 67.343 seconds with 83 runtime, 18 prototype, and 20 host tests.
All maintained product processes consume exact committed true and no longer infer
`groundedAfterBatch`. Init and guard pass. No RuntimeActor storage, duplicate query, input action,
renderer/GPU/resource/synchronization/source/lifecycle/format/asset, or Wulin implementation
changed, so the long canonical runtime workflow was not required.
