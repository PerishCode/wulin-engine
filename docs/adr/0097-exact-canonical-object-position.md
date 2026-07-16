# ADR 0097: Exact Canonical Object Position

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0094 Exact Canonical Object Position

## Context

The committed object query returns the exact raw schema-3 triple by owner region and authored local
ID. Later CPU policy needs the same signed-region/half-open-local-Q9 position already used by terrain
query, contact, motion, actor state, and camera anchoring, but schema validation currently guarantees
only finite floats.

Silently rounding raw values would create a second coordinate authority. Treating the authored
closed `+8m` edge as half-open without normalization would reject canonical fixture data. Replacing
the owner region at that edge would also corrupt the region-local identity scope.

## Decision

- Add `CanonicalObject::terrain_position` as a checked conversion of the already returned value.
- Require finite X/Z values whose multiplication by 512 is an exact integer in closed
  `[-4096,4096]`. Reject every other value without rounding or clamping.
- Reuse `TerrainPosition` normalization. Values below `+4096` remain in the owner region; an exact
  positive edge advances that axis by one region and becomes local `-4096`. Propagate checked signed
  region overflow.
- Keep `(CanonicalObject.region, authored_local_id)` as identity. The returned `TerrainPosition`
  describes space and may carry an adjacent region independently.
- Advance the strict workbench query response to `exact-canonical-object-position-v1` and return both
  the raw object and derived position. Retain no old revision or alternate conversion route.
- Keep enumeration, selection, distance/facing, visibility, interaction, collision/navigation,
  persistent identity, networking, and Wulin policy outside this decision.

## Consequences

- A caller can join an exact committed object to terrain-position consumers without float global
  coordinates, source I/O, GPU readback, allocation, or runtime mutation.
- Malformed future packs fail at the conversion boundary even if their finite float records pass the
  existing schema decoder.
- Positive-edge objects make the distinction between identity ownership and spatial ownership
  observable instead of hiding it in renderer-specific clamping.
- No object-selection policy or long-lived object handle is implied.

## Evidence

Experiment 0094 passes four public-API conversion tests and the full 90-test engine-runtime run.
`canonical-frame-v3` passes in 17.289 seconds with independent schema-3 byte evidence for IDs
0/31/511/992/1023, exact same/X/Z/diagonal normalization, zero query-side work, and unchanged first/
replay color and object-ID hashes.

`canonical-runtime-v3` passes in 273.472 seconds. A/B physical order, source revisit, adjacent
replacement, object/terrain failure rollback, and restart preserve raw and derived positions. Both
32-sample traversal sweeps, the eight-publication resource checkpoint, and two lifecycle cycles pass.
Guard passes with zero Flavor deny issues.
