# Experiment 0076: Prototype Traversal Activation

Status: Accepted

## Hypothesis

The prototype can activate the already accepted camera-driven composition traversal exactly once
after canonical bootstrap and actor spawn, and its fixed actor-relative camera can schedule the
exact first diagonal target in a live process. This application policy can be proved without a
prototype inspect endpoint, sustained-motion controller, prefetch, or new engine traversal logic.

## Scope

Call `Runtime::enable_composition_traversal` once after the initial actor is spawned and before the
window becomes visible. Keep prefetch disabled. Capture only the existing traversal member of
`Runtime::composition_status` after the readiness-producing successful frame and publish it as one
compact readiness snapshot.

Extend the focused source cook with the overlapping target centered at `base + (1,1)` so the exact
camera-driven schedule has valid terrain/object input. Maintained prototype acceptance requires the
same scheduled token/target and accepts automatic publication count zero or one according to async
progress at the readiness snapshot.

Renderer traversal algorithms, publication polling, failure handling, rollover, prefetch, movement
tuning, camera rig, actor simulation, source formats, Runtime API, prototype inspect/control modes,
and Wulin content are out of scope.

## Workload

1. Bootstrap the normal center `(64,64)`, spawn the actor, activate traversal once, and run the
   existing stationary direct/restart processes.
2. Require fixed camera position X/Z `9/12` to map through the accepted 16-meter region basis to
   desired and last-scheduled local target `(65,65)`, global center `base + (1,1)`.
3. Require one session, desired change, attempt, and schedule; no blocked/queued/failure state;
   prefetch absent; and either no completed automatic publication yet or one exact completed
   last-publication for the same token and target.
4. Repeat under process-qualified native W input and require identical traversal target evidence
   alongside the existing exact actor/camera displacement and zero render blocks.
5. Preserve invalid-document, missing-source, corrupt-source, direct restart, Sidecar restart/stop,
   and final cleanup behavior.

## Controlled Variables

- Bootstrap publication, actor spawn, camera offsets, W/A/S/D mapping, gravity, step-up,
  backpressure, frame order, and readiness timing are unchanged.
- Traversal is enabled once only after canonical publication and actor spawn. Prefetch is never
  enabled by the prototype.
- `WORLD_MIN_METERS = -1032`, region side 16 meters, base local center 64, and camera X/Z 9/12 make
  the first desired center deterministically `(65,65)`.
- Async completion timing is not an application contract: zero and one completed-publication
  snapshots are accepted only when scheduled/published target, counters, and failure state agree.
- Existing canonical-runtime evidence remains authoritative for completion, replacement, rollback,
  rollover, plateau, and lifecycle scaling.

## Metrics

- Traversal session/desired/attempt/schedule/publication counts and exact target.
- Scheduled-versus-published normalized token and target.
- Prefetch configuration, queue depth, block/failure state, and rollover count.
- Actor movement, render-block, camera anchor/frame, restart, failure, and cleanup evidence.
- Engine API/algorithm, GPU resource, synchronization, and source-format deltas.

## Acceptance Criteria

- Every valid direct prototype process reports traversal enabled with session/desired/attempt/
  schedule counts `1/1/1/1`, exact target `(65,65)` / `base + (1,1)`, no queue/block/failure, and
  no prefetch status.
- The snapshot has zero automatic publications and no last-publication, or one matching
  last-publication; no other timing-dependent shape is accepted.
- Stationary restart equality and native-W actor/camera evidence remain exact with zero render
  blocks. Failure and Sidecar lifecycle gates remain exact.
- `runseal :canonical-prototype` and `runseal :guard` pass. No engine traversal implementation,
  Runtime API, prefetch, prototype inspect mode, compatibility route, GPU resource, synchronization,
  or source format changes.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, reference Windows host, fresh
signed base/traversal/corrupt centers, capacity-one actor, fixed camera rig, and sole canonical
renderer.

## Evidence

- `runseal :canonical-prototype` passed as `canonical-prototype-v5` in 36.119 seconds.
- Fresh terrain and object packs each contained 59 regions and included base, traversal, and corrupt
  centers. Their SHA-256 values were respectively
  `04202df1680f9b601873610fb269aeb8e9cdfc3c8ebc0bdbc6d4cfee34dfa6ac` and
  `d12c4f5aa1d98ce8b9fc1144633672f0ed7bf71a96e882a478b8c122990ca386`.
- Direct processes 33632, 23512, and 10088 each reported session/desired/attempt/schedule
  `1/1/1/1`, token 2, local target `(65,65)`, and global center
  `(1099511627777,-1099511627775)`. All three readiness snapshots preceded async completion and
  therefore reported publication count zero with no last publication.
- Every process reported no queue, block, failure, prefetch, or rollover. Stationary restart and
  native-W normalized traversal evidence were identical; render blocks remained zero and
  camera anchor/frame counts remained `3/3`.
- Invalid document, missing source, and corrupt source produced no readiness. Direct termination,
  Sidecar restart/stop, and final process cleanup remained exact.
- Focused Rust tests covered 74 runtime, 10 prototype, and 21 reference-host cases. Strict clippy,
  Deno formatting/type checking, `runseal :init`, and `runseal :guard` passed; guard reported zero
  Flavor denies and the five existing warnings.

## Conclusion

The hypothesis is accepted. The plain prototype now activates the already accepted composition
traversal exactly once after bootstrap and actor spawn, and its first real frame schedules the exact
camera-derived diagonal target. Application readiness exposes one bounded snapshot of existing
runtime state while engine acceptance remains authoritative for eventual publication and traversal
failure semantics. Prefetch remains disabled, and no engine traversal algorithm, Runtime API,
renderer/GPU resource, synchronization, source format, inspect route, or compatibility path changed.
