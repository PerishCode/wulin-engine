# Experiment 0064: Retained Runtime Actor Authority

Status: Accepted

## Hypothesis

The sole retained terrain body can be promoted directly into one generation-addressed runtime
actor that owns exact motion and the already accepted schema-3 presentation record, eliminating
the independent body lifecycle without introducing a second identity, a parallel presentation
schema, or a renderer actor path.

## Scope

Replace the capacity-one retained body slot with a capacity-one actor slot. One actor contains its
generation handle, exact `TerrainBodyMotion`, and exact `PresentationRecord`. Validate presentation
before issuing a generation or occupying the slot. Read, despawn, and simulation advance require
the exact live actor handle; simulation may replace only motion and must preserve presentation.

Make `PresentationRecord` validate its own accepted catalog bounds so cooked schema-3 reads and
runtime actor spawn share one rule. The prototype creates one imported Fox presentation after
canonical publication, advances that actor from the accepted Ready-only live driver, and publishes
actor authority in its single readiness record.

Delete the public retained-body handle/value/method vocabulary, old prototype body module, and old
readiness keys directly. Do not add aliases, wrappers, migration paths, detached physics handles,
actor collections, ECS concepts, mutable presentation, GPU actor buffers, rendering, camera,
gravity, locomotion, or gameplay input.

## Workload

1. Prove actor handles reject generation zero and that spawn/read/despawn/respawn preserve exact
   generation and stale-handle behavior.
2. Reject invalid archetype, material, yaw, clip, and phase before actor occupancy or generation
   mutation, using the same `PresentationRecord` validation as signed schema-3 packs.
3. Prove actor motion replacement and schedule/actor dual advance retain the exact presentation;
   invalid handles and controlled simulation failures preserve actor and schedule state.
4. Bootstrap a real prototype at a signed far target, spawn one imported Fox actor, reach a
   nonzero Ready-only commit and following frame, and require initial/input/output actor identity,
   presentation, and motion invariants.
5. Restart direct and Sidecar processes, require fresh process identity plus equal normalized actor
   evidence, preserve terminal invalid/missing/corrupt startup behavior, and finish with no PID.
6. Prove no live source or operator support retains the superseded retained-body public vocabulary.
7. Run focused region-format/runtime/prototype tests, the targeted prototype process gate,
   `runseal :init`, and `runseal :guard`. Do not run the full canonical workflow because the actor
   remains CPU-only and renderer/GPU/resource/synchronization ownership is unchanged.

## Controlled Variables

- Actor capacity remains exactly one and retains the existing checked nonzero `u64` generation
  policy.
- Motion, 60 Hz schedule, bounded elapsed, Ready-only admission, zero command, terrain query, and
  frame order remain unchanged.
- Prototype presentation is exactly imported archetype 7, material 63, yaw 0, animated clip 1,
  phase 0, variant 0.
- Actor presentation uses the exact schema-3 record type and validation; no conversion or fallback
  representation exists.
- Canonical cooked objects remain immutable streamed content and do not become runtime actors.

## Metrics

- Actor generation, capacity/live count, motion, presentation, and lifecycle failure result.
- Presentation validation result for every bounded field.
- Simulation tick/step/query counts plus input/output actor equality except for permitted motion.
- Prototype process identity, readiness ordering, restart equality, and cleanup state.
- Retired-symbol scan, focused test result, targeted gate duration, Flavor denies, and guard result.

## Acceptance Criteria

- Runtime owns one actor identity and one motion/presentation lifetime; no detached retained-body
  identity or public compatibility surface remains.
- Invalid presentation cannot consume a generation or mutate occupancy; all stale/wrong/empty
  operations are transactional.
- Simulation commit preserves actor handle and presentation byte-exactly while replacing only the
  accepted motion output; failure preserves actor and schedule.
- Prototype readiness proves one imported actor survived the first nonzero live commit and
  following successful frame at the exact configured terrain position.
- Direct restarts preserve normalized actor evidence with distinct PIDs; startup failure and
  Sidecar lifecycle gates remain exact with no residual process.
- Focused tests, targeted process gate, `runseal :init`, and `runseal :guard` pass with no full
  canonical run.

## Reference Environment

The experiment uses the pinned Windows reference platform, the unchanged accepted Experiment 0063
cooked fixture sources, the non-diagnostic prototype, Runtime's capacity-one authority and
transactional schedule, the accepted imported Fox catalogs, and the existing no-inspect lifecycle
gate.

## Evidence

All 63 engine-runtime tests, 7 region-format tests, and 3 prototype integration tests pass. They
cover generation admission, exact lifecycle, five invalid presentation fields, presentation-
preserving motion replacement, bounded motion batches, schedule/actor rollback, exact imported Fox
bootstrap, and Ready-only time admission. Clippy passes for region-format, engine-runtime,
prototype, and workbench; all changed TypeScript support checks pass.

The 27.7-second real workbench actor lifecycle gate rejected invalid archetype, material, yaw,
clip, and phase before generation one, proved empty/wrong/stale handles and two complete lifecycles,
replayed the exact evidence after a fresh process, and left no owned process. The 41.8-second cached-
source simulation-actor gate proved fractional zero-step commit, coarse/nominal one-second
partition equality, one query per step, byte-identical presentation, and complete query/arithmetic
rollback across three fresh workbench processes. It also proved every superseded body verb is
`unknown_event`.

The 32.21-second prototype gate covered invalid config, missing source, corrupt payload, two direct
launches, and Sidecar start/restart/stop. Both readiness records contained generation-one actor
state at signed center `(1099511627776,-1099511627776)`, exact terrain/center heights
`76288/141824`, imported presentation `(7,63,0,animation=1)`, a nonzero tick-zero actor commit, and
the following successful frame. Direct PIDs differed and final Sidecar state was empty.

`runseal :init` and the 13.14-second repository guard passed with zero denies. The full canonical
workflow was not run because cooked sources, renderer, GPU resources, traversal, synchronization,
and canonical frame/lifecycle evidence are unchanged; the actor is not yet a renderer input.
