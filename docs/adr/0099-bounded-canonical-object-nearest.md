# ADR 0099: Bounded Canonical Object Nearest

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0096 Exact Canonical Object Nearest

## Context

The runtime can look up one authored object by exact owner-region identity and convert its authored
planar coordinates into `TerrainPosition`. A later CPU policy still cannot discover an object from
a position. The current snapshot already owns exactly 25 immutable CPU pages containing 25,600
verified triples, so adding another registry or source query would duplicate publication and
lifetime authority before its scaling value is proven.

Renderer visibility is also the wrong selection source: it is camera-, LOD-, and occlusion-dependent
and cannot define gameplay proximity. Returning all objects would create an allocation and public
enumeration contract larger than the immediate capability requires.

## Decision

- Expose `Runtime::query_nearest_canonical_object(origin, max_distance_q9)` over the current
  committed snapshot.
- Require the exact `TerrainPosition` origin to lie in the published active window. Scan every
  active CPU page and triple exactly once, with an explicit maximum of 25,600 candidates.
- Validate snapshot/page shape, signed owner placement, complete unique authored IDs, stable seed,
  and checked object position while scanning. Any malformed committed state fails the whole query.
- Compute planar signed-region deltas in `i128`. Apply the inclusive `u32` Q9 axis/radius bounds
  before squaring, then return checked `i64` deltas and `u64` squared Q18 distance.
- Select the minimum tuple `(distance squared, owner region X, owner region Z, authored local ID)`.
  Physical record order and normalized spatial ownership cannot influence ties.
- Return the total validated candidate count and one optional `CanonicalObjectNearest`; do not
  allocate or return an enumeration.
- Perform no source I/O, GPU copy/readback, fence wait, synchronization, mutation, or visibility
  filtering. The strict workbench route and independent pack scanner are diagnostic evidence only.
- Do not add a spatial index, retained selection, interaction eligibility, facing/LOS, persistent
  identity, navigation/collision, networking, or Wulin policy.

## Consequences

- CPU policy can deterministically discover one exact committed object near a canonical position
  without a second scene or renderer dependency.
- Cost is intentionally linear but hard-bounded at the already resident 25,600 triples. A future
  index must justify itself with a separate scaling experiment and preserve the same result order.
- A radius may validly return no object even though all pages were validated and counted.
- Positive-edge candidates keep their owner region for identity/ties and use their separately
  normalized terrain position only for distance.
- Query failure exposes malformed committed state rather than silently skipping candidates or
  falling back to disk/GPU data.

## Evidence

Experiment 0096 passes all 95 engine-runtime tests. Four focused tests cover physical reordering,
zero-radius seam ties, inclusive/no-result radii, full capacity, signed edges, and malformed state.

`canonical-frame-v4` passes in 13.639 seconds with an independent scan of 25,600 source triples,
strict pre-publication/origin failures, exact seam and radius witnesses, zero query-side work, and
unchanged first/replay GPU hashes.

`canonical-runtime-v5` passes in 251.987 seconds. Twenty-eight accepted and three rejected nearest
events preserve exact results through A/B, revisit, adjacent publication, both pair rollbacks, and
restart. Both 32-sample traversal sweeps, the state-driven warm plus eight-publication resource
checkpoint, and two lifecycle cycles pass. Guard passes with zero Flavor deny issues.
