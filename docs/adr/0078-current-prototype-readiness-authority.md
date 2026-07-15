# ADR 0078: Current Prototype Readiness Authority

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0075 Mandatory Prototype Readiness Cleanup

## Context

Prototype readiness originally reported the terrain sample and actor value captured at spawn. That
was accurate while the actor remained stationary, but fixed locomotion made the top-level
`actor.state` stale: transaction output and the following camera represented the moved actor while
the nominal actor authority still represented the origin. The spawn terrain witness also described
only the old position and no longer qualified the current state.

Keeping both values as unlabeled live authority would preserve an experiment-era schema at the
expense of current meaning. Adding initial/current aliases would increase that ambiguity.

## Decision

- Prototype readiness removes the top-level spawn terrain witness and copied spawn-time actor
  field. The live loop retains only its local actor handle for simulation and camera operations.
- Top-level `actor.state` is the committed `ActorSimulationAdvance.actor.output` associated with
  the readiness-producing transaction. Capacity and live-count evidence remain unchanged.
- `ActorSimulationAdvance.actor.input` is the sole initial transaction witness. Maintained
  acceptance requires it to be the generation-one grounded center actor, requires top-level state
  to equal output, and requires camera identity/numeric anchoring to consume the same current state.
- The focused report becomes `canonical-prototype-v4`. Version 3 is removed from live code and no
  parser alias, fallback field, Runtime readback, or inspect route is added.

## Consequences

- Readiness has one unambiguous current actor authority after stationary or moving simulation.
- Spawn terrain remains an implementation input to initial grounding, not misleading live
  telemetry. Initial grounding continues to be covered by actor unit tests and transaction input
  invariants.
- Operator support validates current-state relationships rather than retaining old payload shape.
- Runtime behavior, locomotion, gravity, backpressure, camera/frame order, traversal state, engine
  API, renderer/GPU resources, synchronization, source formats, and lifecycle ownership are
  unchanged.

## Evidence

Experiment 0075 records the stale-field scan and final `canonical-prototype-v4` run. It passed in
36.921 seconds: stationary processes had input = output = current; native W had initial Z 0 and
output/current Z -32; current JSON equaled output, camera identity equaled current identity,
step/query was 1/1, render blocks were zero, and anchor/frame was 4/4. All startup failure and
Sidecar cleanup gates remained exact.
