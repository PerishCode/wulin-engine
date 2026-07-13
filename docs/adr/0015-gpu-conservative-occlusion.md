# ADR 0015: GPU Conservative Occlusion

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0014 established a deterministic reverse-Z fragment winner, but all source-visible
skeletal objects still entered mesh execution. Dense crowds therefore paid geometry and
skinning work for objects whose complete conservative bounds were hidden.

Experiment 0012 tested whether the prior compatible winner could remove that work
without CPU enumeration. It also exposed two constraints: generated animation bounds
must be proven rather than guessed, and a second unordered atomic append needlessly
destroys the source list's data locality even when final attachments remain exact.

## Decision

- The complete prior-frame `R32G32_UINT` winner is reduced into an 11-mip `R32_UINT`
  hierarchy. Reverse-Z reduction uses minimum depth bits; background zero prevents
  unsafe rejection.
- History is queried only when the complete skeletal GPU constant image is bit-identical
  to the hierarchy producer. Startup, reset, camera, animation, LOD, residency,
  movement, resize, disable, and restart incompatibility bypass every source object and
  rebuild history.
- Each object query projects an eight-corner fixture AABB, expands its screen rectangle,
  selects a covering mip, samples all four rectangle corners, and rejects only when the
  nearest possible object depth is strictly behind every sampled farthest depth plus the
  registered bias.
- Generated fixture bounds require exhaustive CPU proof before renderer creation. The
  accepted radial half-extent is `0.35 * height + 0.25`; vertical padding is 0.25 and
  screen expansion is two pixels. These values are fixture evidence, not an authored
  asset-bound format.
- GPU compaction has a fixed shape: 100 classification groups, one prefix group, and 100
  stable-scatter groups. The filtered list must preserve the current source survivor
  order exactly. One indirect mesh dispatch consumes its GPU-produced count.
- The hierarchy is read through an SRV during query and written through per-mip UAVs
  during construction. Explicit state transitions contain the read/write lifetime.
- CPU submission does not inspect source counts, hierarchy values, survivor counts, or
  indirect arguments. Readback exists only for requested experiment probes.
- Semantic object IDs, candidate identity, the deterministic visibility winner, and
  fixed-screen resolve remain authoritative and byte-identical to the disabled path.

## Consequences

The reference surface path adds 5,632,332 bounded execution bytes: a 4,915,052-byte
hierarchy, a 614,400-byte filtered list, a 102,400-byte candidate mask, 400 bytes of
group offsets, and 80 bytes of counters. Total surface execution becomes 25,655,340
bytes. Submission shape is independent of source-visible, survivor, occluded, meshlet,
triangle, bone, and pixel counts.

The high-occlusion workload eliminated 74.916 percent of source meshlets and matching
vertex, triangle, and skin-influence work with exact GPU/CPU agreement and unchanged
attachments. This accepts work elimination, not a universal frame-time improvement.
ADR 0014's rasterizer-ordered validation pass showed wide scheduling-sensitive timing
distributions that did not track survivor count. Future production raster/shading
performance must be judged by a separate experiment whose timing surface is not the ROV
oracle path.

Temporal reprojection, moving-object bounds, authored bounds, meshlet-level occlusion,
current-frame depth prepasses, occluder ordering, portals, software rasterization, ray
queries, and adaptive enablement remain unaccepted. Exact full-signature invalidation is
intentionally conservative.

## Evidence

- [Experiment 0012](../../experiments/0012-gpu-conservative-occlusion/README.md) records
  complete hierarchy reduction, exhaustive fixture-bound proof, candidate-mask and
  stable-order validation, disabled/query attachment equality, invalidation sweeps,
  work elimination, optimized distributions, restart, and prior-path regressions.
