# ADR 0025: Signed Camera Traversal

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0024 accepted exact signed identity and atomic publication for manually scheduled
terrain/object pairs. Camera traversal still accepted only a bounded local
`LoadConfig`, so enabling it after a signed publication would either discard global
identity or require a separate scheduling policy.

Experiment 0022 tests whether the existing half-open camera mapping, one in-flight
pair, one latest desired target, and blocked-failure behavior can carry the signed pair
without changing their bounded ownership.

## Decision

- `composition.traversal.enable` derives its session type from the current published
  pair. A local pair starts the unchanged legacy session; a signed pair freezes its
  shared `global_origin` for the session.
- Camera XZ maps first to the existing bounded format-V1 local center. Signed centers
  are then calculated with checked integer arithmetic as
  `global_origin + (local_center - 64)` on each axis.
- Enabling signed traversal validates both extrema of the complete legal local-center
  range. An origin that cannot represent that range in `i64` is rejected before
  traversal state changes.
- One internal traversal target carries the local configuration and optional global
  configuration through desired, queued, blocked, scheduled, and published state.
  Optional fields are omitted so the legacy status shape remains unchanged.
- Automatic signed targets use the accepted atomic composition scheduler. Terrain and
  object halves receive one matching global/local configuration and publish together
  at a frame boundary.
- Existing one-in-flight, one latest desired target, maximum queue depth one,
  no-idle-retry blocked failure, disable, and re-enable catch-up rules remain in force.
- A different signed origin requires traversal disable followed by an explicit signed
  pair publication. Automatic origin changes and render-origin rebasing are not part of
  traversal.
- Format V1, local content aliases, cache capacities, physical slots, payloads, GPU
  descriptors, stable keys, semantic IDs, shaders, grounding, LOD, and fixed submission
  remain unchanged.

## Consequences

Camera movement can now drive exact terrain/object cache identities around signed
origins at plus or minus 2^40 regions without converting global coordinates to `f32`.
Boundary ownership and clamping remain identical to local traversal, while each
published status exposes the matched local and signed target.

A held crossing plus three later desired centers retains one complete old pair, stores
only the latest target, and performs exactly two schedules and publications after
release. Missing terrain blocks one exact signed target without retry or pair mutation.
Disable performs no work, re-enable catches up once, and restart reproduces the same
global mapping and attachments.

This decision accepts automatic movement only inside one explicit signed alias window.
It does not accept automatic rebase, predictive prefetch, cancellation, loading
presentation, authored world partitioning, cooked-object global lookup, collision,
navigation, network coordinates, or an unbounded map.

## Evidence

- [Experiment 0022](../../experiments/0022-signed-camera-traversal/README.md) records
  signed boundary samples, clamp extrema, adjacent movement, held latest-wins behavior,
  blocked failure, disable/catch-up, overflow rejection, restart, and release timings.
- Experiment 0021 passed unchanged and recursively replayed the accepted local
  composition and traversal chain.
- Across 32 release adjacent crossings, each terrain half uploaded 20,480 bytes and
  each object half uploaded 102,400 bytes. Both caches retained 20 and uploaded five
  entries per crossing.

## Reproduction

```powershell
runseal :global-traversal
```

The command writes the ignored report to
`out/captures/0022-signed-camera-traversal/acceptance.json`.
