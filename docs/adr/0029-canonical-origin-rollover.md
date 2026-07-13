# ADR 0029: Canonical Origin Rollover

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0028 made signed terrain and generated objects independent of bounded local aliases
and restored atomic V2 composition, but camera traversal still used one frozen origin.
Long-running traversal could therefore exhaust the local alias range even though both
resident caches and every durable identity were already canonical.

Experiment 0026 tests whether origin rollover can remain a narrow publication policy:
derive an exact signed target first, choose a new bounded projection only when needed,
and publish that projection with the matched terrain/object pair and camera translation.
The policy must not introduce a caller-selected GPU mode, expose mixed coordinate
frames, mutate V1 behavior, or become a general floating-origin framework.

## Decision

- Canonical V2 traversal keeps each published local-center axis inside the inclusive
  safe band `[32,96]`. The desired signed center is always derived from the currently
  published origin before any rollover decision.
- When one axis leaves the safe band, that axis's new origin becomes the exact desired
  signed center and its local center becomes 64. An axis still inside the band retains
  its published origin and local center.
- Origin and desired-center arithmetic is checked `i64`. Only the bounded integer
  origin delta is converted to an exact camera translation in region-sized `f32`
  meters; signed global coordinates never enter GPU floating-point identity.
- A new basis becomes visible only when its matched terrain/object pair commits. The
  scene camera position and target receive the opposite origin delta at that same
  commit boundary, before the new frame is recorded.
- Pending, held, superseded, rejected, and failed work retains the complete old pair,
  basis, and camera. Latest-wins traversal remains one in-flight pair plus one queued
  target, and failure cannot cause rollover retry churn.
- Canonical cache entries remain keyed by source namespace plus signed region. A basis
  change may therefore retain already resident terrain and object work without upload.
- Format-V1 local and signed traversal retains its frozen origin, mapping, attachments,
  and status shape. Rollover status is absent when canonical rollover does not apply.

## Consequences

A fixed signed window can normalize from alias 97 to alias 64 while both caches retain
all 25 regions with zero I/O or upload and every rendered attachment remains identical.
Crossing one safe-band axis retains 20 and uploads five entries in both halves; crossing
two axes diagonally retains 16 and uploads nine. Camera continuity no longer depends on
an indefinitely growing local coordinate, while GPU root constants, submission count,
cache capacities, and frame-local semantics remain bounded and unchanged.

Terrain I/O, terrain copy, and object copy holds expose only the old complete coordinate
frame until the final target commits. Missing or corrupt terrain cannot mutate the
published basis or camera, and a valid retry may reuse immutable object cache work.
Disabling traversal suppresses rollover; re-enabling performs one bounded catch-up.

This decision accepts canonical origin rollover, not predictive prefetch, authored
global objects, persistent public IDs, collision, navigation, networking, unrestricted
world precision, or a reusable general-purpose floating-origin subsystem.

## Evidence

- [Experiment 0026](../../experiments/0026-canonical-origin-rollover/README.md) records
  recursive compatibility, normalization, six signed boundary directions, three
  independent holds, failure/retry, disable/catch-up, restart, and release sweeps.
- Experiment 0025 and the complete recursive compatibility chain passed unchanged.
- Across 32 ordinary moves and 32 rollover normalizations, composition GPU median/P95/
  P99 was `0.121856/0.143360/0.144384 ms` and
  `0.106496/0.118784/0.119808 ms`, respectively. No oracle, semantic, attachment,
  validation, lifecycle, or device-removal failure occurred.

## Reproduction

```powershell
runseal :canonical-origin-rollover
```

The command writes the ignored report to
`out/captures/0026-canonical-origin-rollover/acceptance.json`.
