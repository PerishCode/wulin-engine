# ADR 0014: GPU Surface Visibility Resolve

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0013 established bounded GPU animation and meshlet skinning, but material evaluation
still occurred inside geometry execution. That coupling made geometry cost, pixel cost,
surface reconstruction, and later lighting work difficult to measure or evolve
independently.

Experiment 0011 separated those stages with a compact visibility attachment and a fixed
screen-space resolve. During validation it also exposed three contracts that cannot be
left implicit: a surface root signature must not inherit unrelated skeletal constants,
wrapped UVs need a canonical exact-boundary rule, and equal-depth fragments need a
deterministic winner independent of raster arrival order.

## Decision

- Geometry execution writes an `R32G32_UINT` visibility payload. Word 0 stores
  `candidateIndex + 1`, the expanded source primitive index, and one reserved zero bit;
  zero identifies background. Word 1 stores two round-to-nearest UNORM16 barycentric
  components and the third is reconstructed.
- Candidate identity is derived from the active-region ordinal and local instance index.
  It does not depend on atomic visible-record compaction order. A bounded
  candidate-to-visible map recovers the compact record and unique-pose slot.
- Static surface attributes live in a deterministic catalog separate from meshlet
  topology. The accepted topology and animation hashes remain unchanged.
- Visibility rasterization uses a same-pixel rasterizer-ordered winner target. The key
  orders reverse-Z depth bits first and deterministic candidate/primitive identity
  second; `GREATER_EQUAL` depth testing then commits exactly that winner to visibility,
  depth, and semantic-ID targets. Raster arrival order is not observable output.
- Surface reconstruction canonicalizes wrapped UV values at the exact upper boundary to
  zero before explicit point sampling. CPU validation applies the same rule.
- Material resolve uses one direct 8x8 compute dispatch over the fixed render extent and
  writes every pixel, including background. Geometry, pose, material, and occupancy
  changes do not alter its submission shape or maximum pixel work.
- The resolve pass owns a narrow 28-DWORD root-constant interface plus one descriptor
  table. It does not inherit the broader skeletal execution root contract.
- The independent `R32_UINT` semantic object-ID attachment remains authoritative for
  bounded perception. Visibility payloads are renderer reconstruction data and do not
  replace gameplay or server authority.
- Capability acceptance requires barycentrics, rasterizer-ordered views,
  `R32G32_UINT` render-target/shader-load/typed-UAV support, and
  `R8G8B8A8_UNORM` typed-UAV stores. Missing support is an explicit rejection on the
  reference platform, not a fallback path.

## Consequences

Geometry visibility and fixed-screen material work are independently bounded and timed.
The canonical 1280x720 resolve always submits 14,400 groups and shades 921,600 pixels,
while the geometry path retains one indirect mesh dispatch independent of visible
objects, meshlets, triangles, poses, and bones.

The deterministic winner target adds one bounded 8-byte-per-pixel resource. Together
with visibility, resolved color, catalog, maps, statistics, and samples, the experiment
reserves 20,023,008 execution bytes. This is accepted reference-path evidence, not a
permanent memory policy or resolution policy.

The generated normal/UV stream, 64 material records, texture-array fixtures, explicit
mip selection, point sampling, and one directional laboratory light are validation
inputs. General material graphs, authored asset formats, filtering, tangent space, PBR,
HDR, transparency, shadows, clustered lighting, virtual texturing, and post-processing
remain unaccepted.

## Evidence

- [Experiment 0011](../../experiments/0011-gpu-surface-resolve/README.md) records exact
  catalog hashes, visibility and color hashes, six full CPU/GPU reconstruction samples,
  geometry/pose/material/mip/radius sweeps, debug correctness, release distributions,
  movement, revisit, restart, and prior-path regressions.
