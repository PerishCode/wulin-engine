# Experiment 0053: Retained Terrain-Body Lifecycle

Status: Accepted

## Hypothesis

The runtime can own exactly one neutral retained terrain-body motion behind a nonzero generation
handle, with explicit spawn, exact read, and despawn semantics that reject occupied capacity and
stale handles without mutating accepted state, before any live clock, input mapping, stored motion
advance, actor container, presentation binding, or general ECS is introduced.

## Scope

This experiment changes body ownership, not body simulation. `Runtime` gains one optional
`TerrainBodyMotion` slot. A successful spawn issues the next checked nonzero generation and stores
the copied motion. Read and despawn require the exact live handle. A successful despawn empties the
slot; the next spawn uses a distinct generation so an older handle cannot alias the new body.

Capacity is deliberately one. No accepted workload currently needs multiple CPU-simulated bodies,
and choosing an arbitrary larger capacity would establish allocation policy without evidence. The
slot is called a retained terrain body rather than a player or actor, so gameplay authority is not
embedded in the engine.

Terrain queries, contact, planar/vertical advance, simulation-schedule consumption, elapsed-time
driving, input, camera, object identity, render presentation, and Wulin behavior remain out of
scope.

## Workload

1. Add one private runtime slot owner plus public immutable handle/retained-value types. Keep all
   mutation behind `Runtime` lifecycle methods.
2. Prove empty behavior, first spawn/read/despawn, occupied rejection, wrong handle rejection,
   exact failure rollback, respawn generation change, stale-handle rejection, and checked
   generation exhaustion.
3. Expose three strict diagnostic verbs for spawn, read, and despawn. Reject malformed payloads and
   preserve exact fixed-point motion values across the real process boundary.
4. Exercise lifecycle and immediate replay in one fresh workbench process, then restart and prove
   that process-owned state is empty. Require unchanged simulation/presentation status and zero
   terrain, source, GPU, frame, renderer, or synchronization work per lifecycle operation.
5. Run focused engine/workbench checks and `runseal :guard`. Retain the short gate in the canonical
   wrapper without executing the long GPU/lifecycle workflow for this CPU-only field.

## Controlled Variables

- Capacity is exactly one retained terrain body. No collection growth, heap allocation, free list,
  entity kind, or component storage exists.
- Handles contain only a checked nonzero `u64` generation. Callers cannot choose the generation
  issued by spawn.
- A failed spawn/read/despawn changes neither the live handle nor stored motion.
- Despawn returns the exact removed retained value. A subsequent spawn increments generation; it
  never wraps or reuses generation zero.
- The slot is independent from terrain publication, simulation schedule, presentation time, and
  frame execution.

## Metrics

- Focused test count and exact lifecycle/error-branch assertions.
- Process-issued generations, exact serialized motion, stale/occupied/malformed rejection, replay
  digest, restart emptiness, and short-gate elapsed time.
- Schedule/presentation snapshots and allocation, terrain-query, source-I/O, GPU, fence,
  synchronization, frame, and renderer work counters.
- `runseal :guard` result.

## Acceptance Criteria

- `Runtime` is the sole owner of one optional retained motion; the renderer, scene, host, and
  workbench hold no duplicate simulation body state.
- Spawn while occupied and every invalid/stale access fail explicitly with byte-identical retained
  state afterward. Despawn/respawn invalidates the prior handle.
- Generation overflow fails while the slot remains empty and does not wrap to zero or alias an old
  handle.
- The strict real-process lifecycle reproduces exact motion and generation evidence, resets on
  process restart, and leaves simulation and presentation states unchanged.
- Focused tests, the short process gate, and `runseal :guard` pass. The long canonical workflow is
  not repeated because no frame, renderer, GPU resource, synchronization, or existing lifecycle
  path consumes the new slot.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows workbench. It is a
CPU ownership/lifetime proof with no terrain-source or renderer dependency.

## Evidence

The implementation adds one private `TerrainBodySlot` owned directly by `Runtime`. Its public
surface contains only immutable `TerrainBodyHandle` / `RetainedTerrainBody` values and runtime
spawn, read, and despawn methods. The slot uses no collection, heap allocation, free list,
terrain query, simulation schedule, scene state, or renderer path.

All 54 focused `engine-runtime` tests passed. Five lifecycle tests cover nonzero-handle validation,
empty behavior, exact first spawn/read/despawn, occupied-spawn and wrong-handle rollback,
despawn/respawn generation change, stale access rejection, and forced `u64::MAX` generation
exhaustion with an unchanged empty slot. The workbench and strict protocol compile cleanly.

The final fresh-process gate passed in 18,477.4 ms. Each process retained two distinct signed
fixed-point motions in sequence, issued generations 1 and 2, rejected occupied spawn, zero,
malformed, wrong, stale, and empty handles, and returned exact removed values. The second process
began empty and replayed byte-equivalent evidence; result and replay SHA-256 were both
`74f1b0e22b17fdc603d66082773e0824e0a54307364b0e57c1162f4bc1e11ced`. Simulation and
presentation status were unchanged, and every declared terrain-query, allocation, source-I/O,
GPU, fence, synchronization, frame, and renderer counter was zero.

`runseal :guard` passed with zero Flavor denies and all repository suites green. The live canonical
wrapper now identifies Experiment 0053 and retains the lifecycle gate. The approximately
ten-minute canonical workflow was not executed because the new CPU slot is absent from frame,
renderer, GPU-resource, synchronization, and existing lifecycle paths.
