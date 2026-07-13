# ADR 0030: Bounded Canonical Traversal Prefetch

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0029 lets canonical traversal continue across bounded local origins, but every new
camera region still starts terrain I/O and generated-object preparation only after the
crossing. Old-frame retention prevents mixed publication, yet preparation latency can
remain directly on the traversal demand path.

Experiment 0027 tests whether the existing 50-slot caches and matched transaction can
prepare exactly one adjacent target without another residency tier, speculative active
snapshot, or change to signed identity. Speculation must remain invisible, bounded, and
strictly subordinate to actual camera demand.

## Decision

- Canonical prefetch is an explicit traversal capability, disabled by default, and is
  rejected for V1 composition. It adds no caller-selected signed target, physical slot,
  GPU projection mode, velocity history, or path queue.
- While the camera still maps to the published region, motion into the final four meters
  toward one or two boundaries selects exactly one adjacent target. Target derivation
  uses the same checked signed mapping and rollover policy as demand traversal.
- Prefetch runs the existing matched terrain/object transaction. Once both halves stage,
  their active mappings are discarded while immutable transfer cache state remains
  resident. The published pair, basis, camera, descriptors, and semantic attachments do
  not change.
- Crossing a completely prepared target schedules the normal atomic pair and must retain
  all 25 entries with zero upload. Crossing before completion promotes that exact pending
  transaction from prefetch to demand; it publishes only after both halves stage.
- Submitted I/O and copy work remains non-preemptible. A stale prefetch may finish and
  populate caches, but it never publishes. Backpressure stays bounded to one transaction
  plus one latest demand target.
- Prefetch failure is diagnostic and does not create a blocked demand target or retry
  without new motion. Valid immutable work from the other half may remain resident and
  be reused by a later demand retry.
- Existing 50-slot terrain and object caches, 25-entry active snapshots, protected active
  slots, copy queues, root constants, submissions, and V1 status remain unchanged.

## Consequences

One-axis preparation performs the same `20/5` cache movement as reactive traversal;
diagonal preparation performs `16/9`. The subsequent demand is `25/0` in both halves
with zero terrain payload bytes and effectively zero generated-object work. No second
cache, speculative descriptor set, or early coordinate-frame publication is required.

Terrain I/O, terrain copy, and object copy holds can promote the exact prefetch safely.
Opposite-direction demand remains queued at depth one while stale work is discarded.
Missing and corrupt chunks leave demand unblocked; after corruption, valid object work
is reused so retry is terrain `20/5` and objects `25/0`. A prepared rollover changes
basis and camera only when demand commits.

This decision accepts one adjacent canonical lookahead, not general path prediction,
multi-target prefetch, authored object streaming, collision, navigation, networking,
or a guarantee that already-submitted copy work can be preempted.

## Evidence

- [Experiment 0027](../../experiments/0027-canonical-traversal-prefetch/README.md)
  records recursive compatibility, six directions, three promotion gates, stale work,
  missing/corrupt failures, rollover, disable, restart, and release sweeps.
- Experiment 0026 and the complete recursive compatibility chain passed unchanged.
- Across 32 prepared release crossings, preparation median/P95/P99 was
  `0.4059/0.6718/0.7940 ms`; demand pair publication was
  `0.3090/0.5700/0.9533 ms`. All demand terrain reads transferred zero bytes.
- Control and prepared composition GPU median/P95/P99 was
  `0.117760/0.138240/0.141312 ms` and `0.119808/0.141312/0.142336 ms`, respectively,
  with no validation, oracle, semantic, lifecycle, or device-removal failure.

## Reproduction

```powershell
runseal :canonical-traversal-prefetch
```

The command writes the ignored report to
`out/captures/0027-canonical-traversal-prefetch/acceptance.json`.
