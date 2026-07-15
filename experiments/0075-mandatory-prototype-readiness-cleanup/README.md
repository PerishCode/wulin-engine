# Experiment 0075: Mandatory Prototype Readiness Cleanup

Status: Accepted

## Hypothesis

The live prototype readiness surface can remove its bootstrap-era terrain witness and stale spawned
actor snapshot, publish the committed simulation output as its sole top-level actor state, and keep
all current locomotion, camera, restart, failure, and lifecycle evidence. This can be done without
adding a compatibility schema, querying the actor again, or changing runtime behavior.

## Scope

Remove `ReadinessEvidence.terrain` and its top-level `actor.terrain` serialization. Remove the copied
spawn-time `ReadinessEvidence.runtime_actor`; keep the local actor handle required by the live loop,
but serialize `advance.actor.output` as top-level `actor.state` and camera authority.

Update maintained prototype acceptance so `advance.actor.input` owns initial spawn invariants,
top-level `actor.state` must equal `advance.actor.output`, and the camera must use that same current
actor. Replace the current canonical-prototype report revision directly; retain no version alias,
fallback parser, or legacy field tolerance.

Locomotion tuning, gravity, actor simulation, input injection, traversal, camera rig, renderer/GPU,
source formats, and lifecycle behavior are out of scope.

## Workload

1. Require live source and operator support to contain no serialized bootstrap terrain witness,
   copied readiness actor field, stale-initial top-level actor assumption, or prior report revision.
2. Run two stationary direct prototype processes and require input/output/current actor equality,
   exact restart equality, and unchanged camera/frame evidence.
3. Run the process-qualified native W workload and require initial input at the center, exact moved
   output, top-level current state equal to that output, and the camera anchored to the same state.
4. Preserve invalid-document, missing-source, corrupt-source, direct restart, Sidecar restart/stop,
   and complete cleanup behavior.

## Controlled Variables

- Actor spawn/query, fixed W/A/S/D mapping, 60 Hz schedule, gravity, step-up, pending backpressure,
  camera/frame order, and readiness timing are unchanged.
- `ActorSimulationAdvance.actor.input` remains the immutable pre-transaction actor;
  `ActorSimulationAdvance.actor.output` is the committed actor for every accepted readiness.
- Top-level actor capacity/live count remain current cardinality evidence. Only stale terrain/state
  payload ownership is removed.
- No Runtime readback/accessor, inspect endpoint, alternate schema parser, or compatibility alias is
  introduced.

## Metrics

- Removed stale serialized fields and copied readiness values.
- Equality of transaction input, transaction output, top-level actor state, and camera authority.
- Stationary/moving command, step/query, render-block, anchor/frame, and restart evidence.
- Failure readiness count, process replacement, and final cleanup.
- Runtime behavior, engine API, renderer/GPU resource, synchronization, and source-format deltas.

## Acceptance Criteria

- Live readiness contains no top-level spawn terrain witness and no stale spawn actor snapshot.
- Top-level `actor.state` is byte-equivalent JSON to `advance.actor.output`; stationary output equals
  input, while native-W output moves exactly and input remains the initial center actor.
- Camera actor identity and numeric anchor are derived from that same current top-level state.
- The focused prototype workflow passes under one new report revision, with no compatibility alias
  or fallback interpretation. Failure/restart/lifecycle evidence remains exact.
- `runseal :canonical-prototype` and `runseal :guard` pass. No runtime behavior, traversal, engine
  API, renderer/GPU resource, copy, synchronization, or source-format change occurs.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, reference Windows host, fresh
signed sources, capacity-one runtime actor, fixed locomotion/camera policy, and sole canonical
renderer.

## Evidence

- Live source/operator scans found no serialized `evidence.terrain`, copied
  `ReadinessEvidence.runtime_actor`, or `canonical-prototype-v3` reference. The top-level actor now
  serializes only `capacity`, `liveCount`, and current `state`.
- Production removes six net lines: the returned spawn terrain value, two copied readiness fields,
  stale terrain serialization, and stale camera/actor uses. Maintained acceptance adds only the six
  net lines needed to relate initial input, committed output, current authority, and camera.
- The final `runseal :canonical-prototype` emitted `canonical-prototype-v4` and passed in `36.921`
  seconds with 74 runtime tests plus semantic actor identity, ten prototype tests, and 21
  reference-host tests.
- Stationary processes `22112` and `33076` retained input = output = current actor and exact restart
  invariants. Native-W process `33360` retained transaction input Z 0 while output and top-level
  current state were byte-equivalent at Z -32 Q9.
- The moving process retained step/query `1/1`, render-block count `0`, current/camera handle
  equality, and anchor/live-frame count `4/4`. The maintained numeric camera oracle also passed.
- Invalid document, missing source, and corrupt payload emitted no readiness. Sidecar start/restart,
  stop, and final zero-process cleanup passed.
- `runseal :init` and `runseal :guard` passed. Guard reported zero Flavor denies and the same five
  pre-existing warnings.
- No compatibility field/revision, Runtime readback, inspect route, runtime behavior, traversal,
  engine API, renderer/GPU resource, copy, synchronization, or source-format path was added.

## Conclusion

Accepted. The prototype readiness surface now has one current actor authority and one explicit
transaction input witness; the stale spawn terrain/actor payload has no live compatibility form.
