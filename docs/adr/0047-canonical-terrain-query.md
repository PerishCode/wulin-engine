# ADR 0047: Canonical Terrain Height Query

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The accepted runtime retains exact CPU terrain tiles after atomic canonical publication, but the
only current height sampler is private evidence code driven by render-projected floating-point
positions. The visual contact probe also selects camera-dependent terrain LOD. Neither is a safe
dependency for later body or spatial-simulation experiments.

The next dependency is narrower than a physics system: callers need an exact read-only answer for
one position in the currently published terrain, with global identity preserved and publication
failure semantics made explicit.

## Decision

- `engine-runtime` defines a typed terrain query position as signed `RegionCoord` plus local X/Z
  Q9 coordinates in the half-open range `[-4096, 4096)`.
- The query result contains a signed height numerator with fixed denominator 65,536 and an exact
  triangle class. Normal, slope, material, contact policy, and body state are not part of this
  contract.
- `Runtime` exposes one read-only query delegated to the last committed canonical CPU terrain
  snapshot. The renderer may retain physical tile storage, but cache slots, descriptors, GPU
  resources, projections, and render LOD remain private.
- Queries before publication, outside the published active window, with invalid local coordinates,
  or against inconsistent assignment/tile identity fail. They do not clamp, normalize, load,
  wait, read back, or fall back to another snapshot or representation.
- Sampling is pure bounded integer interpolation over the canonical 33x33 height lattice. One
  query performs no heap allocation and no source or GPU operation.
- The existing render grounding sampler remains independent until Experiment 0044 completes its
  oracle comparison. Any consolidation is a later cleanup decision based on accepted evidence,
  not a compatibility obligation.

## Consequences

- Later body/contact experiments can depend on one stable spatial height primitive without
  depending on camera, render LOD, workbench inspect, or floating-point global coordinates.
- The initial lookup is a bounded linear scan over at most 25 published assignments. A separate
  index is unjustified until measurement proves this fixed bound inadequate.
- Region seams are unambiguous: local positive bounds are invalid for the current region and must
  be represented as the adjacent signed region at local `-4096`.
- Terrain ownership remains under the canonical renderer during this experiment. Moving retained
  CPU snapshot storage requires independent lifecycle or simulation pressure.

## Evidence

Experiment 0044 passed the 616-second direct workflow. Its 76,800-sample 5x5 probe produced exactly
25,600 samples in each triangle class, zero mismatches against the independent grounding oracle,
and exact result/identity hashes across reorder, revisit, alias, restart, hold, rollback,
traversal, rollover, and 16 lifecycle processes. Successful queries reported zero allocations,
source reads, GPU copies/readbacks, fence waits, and synchronization. All prior controlled GPU
hashes, the 527-handle zero-growth resource plateau, and complete lifecycle cleanup remained exact.

Generated evidence will remain ignored under
`out/captures/0044-exact-canonical-terrain-query/`.
