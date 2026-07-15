# Experiment 0078: Committed Locomotion Facing

Status: Accepted

## Hypothesis

The prototype can derive exact eight-way Q16 yaw from its normalized W/A/S/D locomotion command and
retain the last successfully committed heading while stationary, without adding another runtime
actor authority or allowing fractional/render-blocked work to advance application policy ahead of
the transactional actor.

## Scope

Replace the stateless prototype presentation function with one prototype-owned committed-facing
policy. Nonzero locomotion chooses yaw from command signs. Zero locomotion uses the policy's last
committed yaw. The policy observes an advanced actor output only when the simulation emitted at
least one fixed step; a zero-step advance cannot update it, and a typed render block supplies no
advanced output.

The imported Fox remains archetype 7/material 63, Survey clip 0 while stationary, Walk clip 1 while
moving, phase offset 0, and variation 0. The imported hip/spine/head direction establishes local +X
as authored forward; the accepted renderer yaw transform maps this to world X/Z. Exact headings are
D `0`, S+D `8192`, S `16384`, S+A `24576`, A `32768`, W+A `40960`, W `49152`, and W+D `57344`.

Animation phase restart/continuity, blending, root motion, Run, camera rotation, analog input,
horizontal acceleration, jump, traversal/prefetch changes, runtime APIs, inspect verbs, renderer or
shader changes, GPU resources, source formats, and Wulin content are out of scope.

## Workload

1. Prove all eight normalized cardinal/diagonal command signs map to the exact Q16 heading and
   opposing/zero input requests the current committed heading.
2. Prove a zero-step observed advance leaves policy yaw unchanged, a nonzero admitted output updates
   it, and a following stationary Survey command retains that yaw.
3. In direct stationary prototype processes, require initial/command/output/current Survey yaw 0.
4. Under the existing process-qualified native W input, require input Survey yaw 0 and the same
   transaction's command/output/current Walk yaw 49152, exact Z `0 -> -32`, existing camera,
   traversal, zero-block, restart, failure, and cleanup evidence.
5. Replace the maintained prototype report revision in place and retain no fixed-yaw assertion,
   optional field, alias, fallback, or second presentation mutation.

## Controlled Variables

- Input normalization, displacement magnitudes, step-up/gravity policy, simulation timing,
  transactional render admission, actor ownership, and camera/frame ordering remain unchanged.
- Presentation policy stores only the last admitted yaw needed to author the next command. The
  runtime actor remains the complete motion/presentation authority.
- Policy state follows only nonzero `ActorSimulationAdvance.actor.output`. Fractional advance and
  render-block behavior remain owned by the accepted runtime transaction.
- Traversal stays one-time enabled with prefetch disabled. Existing runtime frame driving remains
  sufficient; no sustained-crossing telemetry is introduced.

## Metrics

- Exact command-to-yaw mapping for all eight nonzero directions.
- Policy yaw before/after zero-step and nonzero-step observations.
- Initial, command, transaction input/output, and current actor yaw/clip values.
- Native-W displacement, step/query, presentation mutation, render block, camera/frame, traversal,
  restart, failure, and process cleanup evidence.
- Runtime API, inspect, renderer/shader/GPU resource, synchronization, and format deltas.

## Acceptance Criteria

- All eight headings equal the declared Q16 values; zero/opposing input retains the committed yaw.
- Zero-step observation and absence of an advanced outcome cannot change policy state. One nonzero
  admitted output changes policy yaw exactly, and subsequent stationary Survey retains it.
- Native W atomically changes yaw `0 -> 49152`, clip `0 -> 1`, and Z `0 -> -32` in one step/query,
  while stationary direct processes remain exact at yaw/clip 0.
- `runseal :canonical-prototype` and `runseal :guard` pass. No fixed-yaw live assertion, old report
  revision, compatibility path, separate actor mutation, runtime/inspect API, renderer/shader/GPU
  resource, synchronization, source-format, traversal, or camera behavior change remains.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, reference Windows host, existing signed
canonical sources, imported Fox, normalized W/A/S/D input, capacity-one actor, and sole renderer.

## Evidence

- All eight exact heading cases passed in prototype policy tests. The same policy test proved a
  zero-step observation retained yaw 0, a nonzero W output committed yaw 49152, stationary and
  opposed input retained 49152 with Survey, a zero-step D output retained 49152, and a nonzero D
  output committed yaw 0.
- `runseal :canonical-prototype` passed as `canonical-prototype-v7` in 38.726 seconds. Direct
  processes 4576 and 16328 reported initial/command/input/output/current yaw/clip `0/0`, zero
  displacement, zero render blocks, and camera/frame `3/3`.
- The process-qualified native-W process 4524 reported input yaw/clip `0/0`, command/output/current
  yaw/clip `49152/1`, exact Z `0 -> -32`, step/query `1/1`, zero render blocks, and camera/frame
  `3/3`.
- Existing exact traversal token/target, no-prefetch/queue/block/failure, bootstrap failure,
  restart equality, native input qualification, Sidecar restart/stop, and final cleanup evidence
  remained unchanged.
- Focused validation passed 75 engine-runtime, 12 prototype, and 21 reference-host tests, strict
  prototype clippy, Rust/Deno formatting and type checking, `runseal :init`, and
  `runseal :guard`. Guard reported zero Flavor denies and the five existing warnings.
- The production delta adds only prototype command-authoring state and observes accepted output. No
  Runtime or inspect API, actor storage/readback, renderer/shader/GPU resource, synchronization,
  source format, traversal, camera, or second presentation mutation changed.

## Conclusion

The hypothesis is accepted. The prototype now maps every normalized nonzero locomotion command to
the exact imported-Fox world heading and retains only the last nonzero admitted output yaw for
stationary authoring. Fractional advances and typed blocks cannot move policy ahead of the runtime
actor. Native W proves yaw, Walk, and displacement commit together. No fixed-yaw live assertion,
old report revision, compatibility path, duplicate actor authority, runtime/inspect API,
renderer/shader/GPU resource, synchronization, format, traversal, or camera change remains.
