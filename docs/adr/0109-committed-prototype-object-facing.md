# ADR 0109: Committed Prototype Object Facing

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0106 Committed Prototype Object Facing

## Context

ADR 0106 gates an object action by exact committed proximity, and ADR 0107 gives one successful
action a capacity-one lifetime. The accepted native `F+Enter+W` evidence nevertheless activated an
object at positive X while the actor's committed yaw faced negative Z. Radius alone therefore
allows side-on and rear product actions.

The committed actor output already carries one of eight exact locomotion yaws, and resolved object
proximity already carries exact signed Q9 deltas. Adding engine steering, scene visibility, or a
copied target state is unnecessary.

## Decision

- Prototype action eligibility consumes the yaw from the same committed actor output that supplies
  its terrain origin.
- The eight accepted yaws map to the exact integer directions `(1,0)`, `(1,1)`, `(0,1)`, `(-1,1)`,
  `(-1,0)`, `(-1,-1)`, `(0,-1)`, and `(1,-1)`.
- After exact inclusive-radius admission, a non-coincident target is eligible only when
  `delta_x * direction_x + delta_z * direction_z` is positive. Zero-distance targets remain
  eligible.
- A zero or negative non-coincident dot is the typed Prototype policy outcome `OutsideFacing`.
  A non-eight-way yaw is malformed state and fails before policy mutation.
- Facing evidence reports the committed yaw, exact integer direction, and signed Q9 dot. It adds no
  retained state and does not alter frame feedback or consumption transactions.

## Consequences

- Action intent, observation, resolution, proximity, acknowledgement, exclusion, suppression, and
  source/window lifetime retain their existing ownership and ordering.
- Runtime, canonical source/snapshot, renderer/GPU, resources, synchronization, and formats are
  unchanged.
- This decision does not authorize arbitrary steering, action cones, line of sight, visibility
  readback, registries, rewards, inventory, dispatch, respawn, persistence, networking, or Wulin
  semantics.

## Evidence

Nine focused policy tests cover every exact direction, front/side/rear/coincident outcomes and
rollback. `canonical-prototype-v23` passes in 75.629 seconds. Native `F+Enter+D` commits ID 496 with
yaw/direction/dot `0 / (1,0) / 128`; native `F+Enter+W` rejects the same ID as side-facing with yaw
49,152, exact delta `(160,0)`, one ineligible attempt, and zero Activated/consumption state.

`runseal :guard` passes with zero Flavor denies after the paired action gates were moved under the
existing object-gate owner, leaving the Prototype host at 497 lines. The final-worktree
`canonical-runtime-v14` passes in 254.675 seconds with all source/window, rollback, restart, 32+32
traversal, resource, and lifecycle gates unchanged. Five warm/eight measured publications retain
492 handles and 21 threads; private bytes change by +532,480. The report retains 24 files /
25,346,301 bytes.
