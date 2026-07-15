# Experiment 0074: Prototype Horizontal Locomotion

Status: Accepted

## Hypothesis

The prototype can map the reference host's normalized W/A/S/D held state to one fixed integer
horizontal actor command per admitted 60 Hz sample, and an actual native key message can produce
exact actor and actor-relative-camera displacement through the live process. This can be proved
inside the already published render window without enabling traversal, adding velocity state, or
changing engine ownership.

## Scope

Add one prototype-owned locomotion policy over `HostInput::is_held`. Cardinal movement is 32 Q9
units per fixed step; simultaneous perpendicular directions use the fixed nearest normalized
component 23 Q9; opposing directions cancel independently. Every command carries a fixed 32,768
Q16 step-up limit and the existing -179 Q16 gravity acceleration.

The live loop samples this policy after message ingestion and before clock admission, sends its
command through the existing atomic actor simulation boundary, and reports that exact command in
readiness. A maintained `canonical-prototype` support module posts a real Windows key message to
the existing prototype window so the focused gate exercises the reference window and normalized
input path rather than a prototype inspect or test mode.

Traversal and prefetch, velocity/acceleration state, input configurability, jumping, yaw or
animation selection, camera smoothing, multi-actor storage, Wulin content, engine API changes,
renderer changes, GPU resources, and synchronization are out of scope.

## Workload

1. Unit-test zero, cardinal, diagonal, opposing, irrelevant-key, and focus-loss input reduction
   through the prototype policy.
2. Run two ordinary direct prototype processes and require zero horizontal displacement, exact
   restart equality, fixed command evidence, grounded simulation, and one camera anchor per frame.
3. Hold a native W message on a third direct process and require the first readiness-producing
   batch to move exactly -32 Q9 on Z per fixed step, retain X and region identity, query terrain
   once per step, encounter no render block, and anchor the camera at the moved actor.
4. Preserve invalid-document, missing-source, corrupt-source, direct restart, and Sidecar lifecycle
   gates without adding a diagnostic endpoint or alternate executable mode.

## Controlled Variables

- One admitted fixed step uses cardinal delta 32 Q9; a diagonal uses component 23 Q9. There is no
  accumulated horizontal velocity or frame-time scaling outside the accepted 60 Hz schedule.
- W/S map to negative/positive Z and A/D to negative/positive X. Opposing inputs cancel per axis.
- The step-up limit is 32,768 Q16 and gravity remains -179 Q16 for every sampled command.
- Input is reduced once after normalized message ingestion and before `HostClock` sampling. A
  multi-step admitted sample repeats the same immutable command for its complete batch.
- Typed pending-window backpressure keeps its no-retry/no-backlog consumption policy.
- Composition traversal remains disabled. Source pages, camera rig, actor presentation, frame
  order, shaders, passes, resources, uploads, barriers, fences, and waits are unchanged.

## Metrics

- Exact command components for each normalized input state.
- Native injected-key discovery/post result and live readiness command identity.
- Simulation step/query count, actor input/output position, region identity, and render-block count.
- Actor-relative camera displacement and camera-anchor/live-frame count.
- Direct restart equality, startup-failure readiness count, and final owned-process cleanup.
- Engine API, traversal state, GPU resource, copy, synchronization, and allocation deltas.

## Acceptance Criteria

- Policy tests prove the complete fixed W/A/S/D reduction, including opposing cancellation,
  diagonal component 23, irrelevant keys, and focus loss.
- An actual W key message posted to the native prototype window yields command `(0, -32)`, fixed
  step-up/gravity values, and exact `-32 * step_count` Q9 actor displacement in the live process.
- The actor remains in the published region, terrain queries equal fixed steps, render-block count
  remains zero, and the camera's X/Y rig is unchanged while both Z coordinates follow the actor.
- Ordinary direct processes remain stationary and restart-identical. Failure and Sidecar lifecycle
  gates retain their existing behavior.
- `runseal :canonical-prototype` and `runseal :guard` pass. No traversal, engine API, renderer/GPU,
  synchronization, compatibility route, prototype inspect mode, or temporary support import is
  added.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, reference Windows window/input
host, fresh signed canonical sources, capacity-one runtime actor, actor-relative camera, and sole
canonical renderer.

## Evidence

- Policy tests passed all four groups covering zero/cardinal input, all four diagonals, independent
  opposing-axis cancellation, irrelevant keys, and focus-loss clearing. The complete focused run
  passed 74 engine-runtime tests plus semantic actor identity, ten prototype tests, and 21
  reference-host tests.
- `runseal :canonical-prototype` emitted `canonical-prototype-v3` and passed in `38.742` seconds.
  Ordinary processes `12192` and `31160` reported `(0, 0)` locomotion, the fixed `32,768` step-up
  limit, exact stationary actor output, zero render blocks, and restart-identical camera state.
- Maintained native-input evidence `prototype-native-key-v1` located and process-qualified the
  third native window for process `2864`, then posted W key-down through Win32 `PostMessageW`.
  There is no prototype inspect endpoint or test executable mode.
- The third process reported `live-prototype-locomotion-driver-v1`, command `(0, -32)`, one fixed
  step and terrain query, input/output Z `0 -> -32` Q9, unchanged region/handle/presentation, zero
  terminal vertical velocity, and zero render blocks. Its terrain-following center height changed
  from `141824` to `142048` Q16 within the accepted step-up bound.
- The actor-relative camera moved from the stationary Z pair `12 / -3` to
  `11.9375 / -3.0625`; its Y coordinates followed the grounded output actor at
  `6.16748046875 / 1.16748046875`. Anchor/live-frame counts remained `3/3`.
- Invalid document, missing source, and corrupt payload each exited with code 1 and emitted no
  readiness. Direct restart, Sidecar restart, stop, and final zero-process cleanup passed.
- Fresh source hashes were terrain
  `17b07794c223c107f17dea9046bc390671501b3b79fa5249428e3dc20a68ab0b` and objects
  `c65096adfe3b3c36897ce562ef81678030b6c4e7884a3e36b47a5381373d7dba`.
- `runseal :init` and `runseal :guard` passed. Guard reported zero Flavor denies and the same five
  pre-existing warnings.
- No traversal, engine API, renderer/frame algorithm, GPU resource, copy, synchronization,
  compatibility route, or temporary support import changed.

## Conclusion

Accepted. Prototype v0 now has one deterministic fixed horizontal input-command boundary and exact
actor-relative camera following inside the current published window. Traversal remains a separate
future experiment with meaningful moving-actor evidence available.
