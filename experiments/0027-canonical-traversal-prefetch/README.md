# Experiment 0027: Canonical Traversal Prefetch

Status: Accepted

- Related ADRs: [ADR 0030](../../docs/adr/0030-bounded-canonical-traversal-prefetch.md)

## Hypothesis

The accepted 50-slot canonical terrain and generated-object caches can hide one-region
traversal preparation without another residency tier. A deterministic one-target
lookahead may run the existing matched transaction, retain its immutable cache work,
and discard its staged active mappings. The published pair, basis, camera, and semantic
attachments therefore remain unchanged until camera demand schedules or promotes the
same target.

## Scope

Prefetch is available only while canonical V2 composition traversal is enabled and is
off by default. The policy observes bounded local camera motion. While the camera still
maps to the published center, entering the final quarter of a region toward one or two
boundaries selects exactly one adjacent signed target through the accepted rollover
policy. No caller supplies signed regions, physical slots, or a GPU projection mode.

One speculative matched pair may be in flight. Once both halves stage, their active
mappings are discarded while the underlying canonical cache population remains
resident. Crossing into a completely prepared target schedules a normal atomic pair
that must retain all 25 regions and transfer zero bytes. Crossing before preparation
finishes promotes that exact transaction to demand; it may publish only after both
halves complete.

A direction change never publishes stale speculative work. At most one non-preemptible
I/O/copy transaction and one latest demand target remain bounded by the existing
backpressure contract. Prefetch failure is diagnostic only and must not block a later
demand retry. Format-V1 traversal, manual scheduling, cache capacities, authored
objects, collision, navigation, networking, and general path prediction remain out of
scope.

## Workload

1. Reproduce Experiment 0026 and its recursive compatibility chain unchanged with
   prefetch disabled and no prefetch status fields.
2. Enable prefetch on one canonical pair around `(2^40,-2^40)`. Move from region center
   into the final positive-X quarter without crossing. Require one 20/5 speculative
   transaction in both halves, no pair/basis/camera/attachment mutation, then one 25/0
   demand publication after crossing.
3. Repeat positive/negative X, positive/negative Z, and both diagonals. Require one
   adjacent candidate, exact signed target, deterministic trigger, and matching terrain/
   object retained/uploaded counts.
4. Cross before terrain I/O, terrain copy, and object copy prefetch completion. Require
   exact-target promotion, no early half or basis visibility, and one atomic demand
   publication after release.
5. Turn away before a held or completed prefetch publishes. Require stale work to remain
   invisible, one latest demand target, bounded queue depth, and deterministic cache
   reuse or eviction when the new direction schedules.
6. Prefetch missing and corrupt terrain. Require no blocked demand target, no automatic
   retry churn, no pair/basis/camera mutation, and a successful later demand retry.
7. Prefetch across a safe-band boundary. Require exact signed candidate generation,
   zero early rollover/camera delta, and one rollover only when actual demand commits.
8. Disable prefetch while traversal remains enabled, restart, and reproduce the base
   frame. Run release sweeps over 32 prepared crossings and 32 unprepared controls under
   fixed observation and capture settings.

## Controlled Variables

- Canonical terrain/object source identity, signed keys, 50-slot caches, 25-entry active
  snapshots, rollover policy, and atomic pair publication remain unchanged.
- Existing terrain I/O, generated-object preparation, copy queues, gates, descriptors,
  root constants, dispatches, indirect execution, and active-slot protection are reused.
- The lookahead horizon is one adjacent region. Trigger distance and motion epsilon are
  fixed experiment constants; no unbounded velocity history or path queue is added.
- Demand uses the accepted one-in-flight plus one latest target policy. Submitted GPU
  copy work remains non-preemptible; stale results may populate caches but never publish.
- Correctness uses the debug workbench. Release uses validation-disabled benchmark mode
  and makes no speedup claim.

## Metrics

- Camera positions, observed motion, trigger boundary/direction, candidate signed/local
  target, schedule/completion/promotion/failure counters, pending purpose, latest demand,
  and maximum queue depth.
- Both source namespaces, retained/uploaded/evicted/resident counts, payload bytes,
  stable-seed overlap, cache slots, copy fences, and prepared-target identity.
- Published pair token, basis/origin, rollover count/delta, camera/view, semantic joins,
  grounding/contact, skeletal/terrain CPU/GPU aggregates, and all attachment hashes.
- I/O, generation, copy, speculative completion, demand publication, combined GPU,
  capture, operator observation, validation, process, and device status distributions.

## Pass Criteria

- Experiment 0026 passes unchanged while prefetch is disabled. V1 status and behavior
  remain byte-compatible and cannot enable canonical prefetch.
- A completed adjacent prefetch changes no published state and both halves agree on
  `20/5`; crossing that target schedules a normal pair with `25/0` and zero transfer.
- An in-flight exact-target prefetch promotes once and publishes atomically only after
  both halves complete. Stale, failed, disabled, or superseded prefetch never publishes.
- Direction reversal remains bounded to one transaction and one latest demand. Demand
  cannot be marked blocked by speculative failure and retries through normal policy.
- A prepared rollover target changes basis and camera only at demand publication. Signed
  joins, stable seeds, oracles, attachments, fixed submission, capacities, validation,
  Flavor, Sidecar lifecycle, and device status all pass.

## Evidence

The complete recursive workflow passed on 2026-07-14 in 946.3 seconds. It reproduced
Experiment 0026 and its compatibility chain with prefetch disabled, then passed the
0027 debug and validation-disabled release workloads. Debug validation was enabled,
release validation was disabled as required, and neither process reported device
removal.

All four cardinal and two diagonal directions selected one exact adjacent signed target.
Cardinal preparation retained/uploaded `20/5` in both halves; diagonal preparation was
`16/9`. Preparation left the published token and visible attachments unchanged. Every
completed cardinal prefetch then produced a `25/0` demand publication with zero terrain
payload bytes and zero object bytes.

Terrain I/O, terrain copy, and object copy holds promoted the same transaction without
early publication. Opposite-direction demand stayed bounded at queue depth one while
stale work remained invisible. Missing and corrupt terrain each failed once without
blocking demand or retry churn. Corrupt retry reused valid object work and reported
terrain `20/5`, objects `25/0`. Prepared rollover changed basis/camera exactly once at
demand commit; disable and restart were deterministic.

The 32 unprepared control samples reported pair-publication median/P95/P99 of
`0.3683/0.7543/0.9213 ms`. The 32 prepared samples reported preparation
`0.4059/0.6718/0.7940 ms` followed by demand publication
`0.3090/0.5700/0.9533 ms`; every demand terrain read was zero bytes. Control and prepared
composition GPU distributions were effectively unchanged. These observations prove
work relocation and bounded cache reuse, not a general performance speedup.

Canonical reproduction:

```powershell
runseal :canonical-traversal-prefetch
```

The ignored structured report is
`out/captures/0027-canonical-traversal-prefetch/acceptance.json`.
