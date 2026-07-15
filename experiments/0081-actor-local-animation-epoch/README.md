# Experiment 0081: Actor-Local Animation Epoch

Status: Accepted

## Hypothesis

One runtime-owned actor animation epoch in the existing renderer presentation-tick domain can make
spawn and committed clip transitions begin at authored phase zero and then follow exact source-
duration cadence. The epoch and actor presentation can remain transactional through fractional
steps, render backpressure, and failure without adding a second clock, changing streamed-object
time, or widening the 56-byte GPU visible record.

## Scope

Add one bounded animation epoch tick to `RuntimeActor`. Spawn assigns the current presentation tick.
A committed nonzero-step change between static/animated state, imported rig/archetype, or clip
assigns the transaction's current presentation tick. A same-stream change of material, yaw,
authored phase offset, or variation retains the epoch.

At frame encoding, resolve only the dynamic actor candidate's effective phase offset from
elapsed-since-epoch time. Keep the stored `ActorPresentation`, schema-3 streamed presentations,
renderer-global timeline, pose key space, shader, visible-record layout, frame-slot upload resource,
copy/synchronization counts, and source-duration tables unchanged.

Update maintained actor/prototype evidence to prove the retained epoch and exact GPU-visible local
phase. Blending, transition graphs, root motion, playback rates, Run policy, multi-actor storage,
gameplay state, source-service expansion, and Wulin content are out of scope.

## Workload

1. Unit-test epoch bounds, spawn assignment, common-period wrap, same-stream retention, and stream-
   identity reset.
2. At fixed nonzero presentation ticks, spawn an actor and inspect the read-back 56-byte candidate.
   Require decoded GPU phase zero at spawn, source-duration progress from elapsed ticks, and phase
   zero after a committed Survey-to-Walk transition.
3. Require same-clip committed changes to retain the epoch. Require zero-step, typed pending block,
   published-window failure, stale handle, and invalid presentation to preserve the complete actor,
   epoch, simulation schedule, and frame transaction as applicable.
4. Run stationary/restarted and native-W prototype processes. Require the bounded spawn epoch,
   stationary retention, and a Walk epoch advanced from spawn by the exact prior live canonical
   frame count; require identical restart formulas and no product-only diagnostic telemetry.
5. Preserve actor upload bytes/resources/copies, canonical frame/resource behavior, startup failure,
   traversal, camera, lifecycle, and final cleanup evidence.

## Controlled Variables

- The renderer presentation timeline remains the sole advancing clock and retains its exact
  31,002,560-frame common period.
- Imported Survey/Walk durations remain 16,400/3,400 presentation units at 80 units per frame and
  64 sampled phases.
- Authored phase offset remains additive after local elapsed phase; variation retains its existing
  meaning.
- Simulation admission, terrain motion/query order, pending-window preflight, actor identity,
  camera/frame order, traversal, and prototype locomotion/facing policy are unchanged.
- Streamed object presentation remains globally clocked and no source format or cooker changes.

## Metrics

- Stored actor epoch at spawn, same-stream commits, clip transitions, and rollback outcomes.
- Global tick, elapsed-since-epoch tick, authored/effective phase offset, and decoded GPU phase.
- Actor visible-record bytes, upload slots/allocation/resources/writes/copies, pose/workload counts,
  and exact capture/readback hashes where controlled.
- Stationary/native-W input/output epoch formulas, presentation, displacement, camera, traversal,
  restart, failure, and cleanup evidence.
- Runtime API, shader, source-format, GPU allocation/copy/synchronization, and streamed-object deltas.

## Acceptance Criteria

- Actor epochs are always below the common presentation period. Spawn uses the current tick;
  committed animated-state/rig/clip transitions reset to the current tick; same-stream commits
  retain it.
- Actor GPU phase equals
  `(authored offset + phase_at_frame(rig, clip, elapsed_since_epoch)) % 64` at fixed ticks, including
  wrap. Spawn and Survey-to-Walk transition both resolve to authored phase zero.
- Fractional advances, typed pending blocks, failures, and rejected operations cannot mutate epoch;
  epoch commits atomically with actor motion/presentation and schedule on admitted nonzero steps.
- Prototype initial Survey epoch is bounded by the common period; stationary output retains it;
  native-W Walk output epoch equals
  `(inputEpoch + liveFrameCount - 1) % commonPeriod` and differs from input.
- The actor record remains 56 bytes with two frame slots, one upload resource, zero GPU copies, and
  no added synchronization. Streamed object hashes/time behavior remain unchanged.
- Focused tests, `runseal :canonical-actor`, `runseal :canonical-prototype`, and `runseal :guard`
  pass. No compatibility field/parser, second clock, shader branch, format change, or fallback path
  is added.

## Reference Environment

The experiment uses the repository-pinned Windows/Rust/Deno toolchains, reference D3D12 GPU,
capacity-one runtime actor, imported Fox rig, fresh signed sources, and sole canonical renderer.

## Evidence

- Source inspection and the first prototype run corrected the audit's initial total-frame
  assumption: seven idle bootstrap shells do not advance presentation. Spawn occurs after the first
  canonical commit at epoch 1; native-W commits on global tick 3, where the old Walk path would
  select imported phase 4.
- Focused engine tests increased from 75 to 77 and passed, including exact common-period wrap and
  stream-identity reset/retention. Affected engine/prototype/workbench clippy passed with warnings
  denied; focused Deno formatting/type checks passed.
- `canonical-actor-v4` passed in 37.433 seconds. At global/epoch 42/42, Survey read back phase 0;
  global 46 / epoch 42 read phase 1. Survey-to-Walk atomically changed epoch 42 to 46 and read phase
  0 at global 46, then phase 1 at global 47. A same-clip yaw commit retained epoch 46 and phase 1;
  a fractional Survey command retained the complete actor and epoch.
- Every fixed-tick actor readback had zero exact-field mismatch. The visible/upload record remained
  56 bytes, two slots / 112 allocation bytes, one upload resource, and zero GPU copies. Existing
  actor workload, semantic, compaction, capture, pending-block, published-failure, and cleanup gates
  passed unchanged.
- `canonical-prototype-v8` passed in 35.284 seconds with 77 engine, ten prototype, and 21 host tests.
  Stationary processes 10180 and 10148 retained input/output epoch 1. Native-W process 23332 changed
  epoch 1 to 3 on live frame 3 while atomically changing clip/yaw/Z from `0/0/0` to
  `1/49152/-32`; camera, traversal, restart, failure, and zero-process cleanup evidence passed.
- Live lifecycle/simulation/prototype report revisions advanced directly to v2/v4/v3; focused
  wrappers advanced to actor v4 / prototype v8. No old live revision alias or fallback remains.
- Runtime actor state adds one bounded `u32` epoch. Shader, 56-byte GPU ABI, upload resources,
  source formats/cookers, streamed-object presentation, timeline advancement, traversal, camera,
  and Wulin content are unchanged. The new current-actor/epoch readiness concept moved intact from
  the process host into focused `prototype/actor.ts` support, keeping host orchestration below its
  source boundary. `runseal :init` and the single merge-checkpoint
  `runseal :guard` passed with zero Flavor denies and five pre-existing warnings.

## Conclusion

Accepted. Runtime actors now join the sole presentation timeline through an explicit local epoch;
spawn and committed clip transitions begin at authored local phase zero, progress at exact source
duration, and retain transactional rollback without a second clock or GPU compatibility path.
