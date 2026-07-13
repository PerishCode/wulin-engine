# ADR 0012: GPU Meshlet Scene Execution

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADRs 0008 through 0011 established bounded GPU candidate work, resident storage,
asynchronous publication, and cooked region I/O. Their render paths still emitted a
procedural six-vertex proxy per visible object, so they did not prove that a published
snapshot could drive real static geometry, geometric LOD selection, or meshlet expansion
without CPU-visible work enumeration.

Experiment 0009 added a deterministic static meshlet catalog and executed it directly
from the accepted cooked resident snapshot through compute, amplification, mesh, and
pixel shaders.

## Decision

- Reusable static geometry may be represented by an explicitly encoded immutable meshlet
  catalog. Catalog bytes, topology ordering, LOD ordering, and meshlet bounds are stable
  data contracts rather than Rust memory layout.
- Every meshlet must satisfy the selected shader contract. The accepted laboratory bound
  is at most 64 unique vertices and 126 primitives per meshlet.
- A GPU compute stage reads published region descriptors, performs object culling and
  LOD selection, and compacts visible object records. CPU code does not enumerate visible
  objects or meshlets.
- One indirect `DispatchMesh` consumes the compacted records. Amplification shaders map
  visible objects to meshlet work, and mesh shaders read catalog vertex and primitive
  buffers to emit geometry.
- Submission shape remains fixed at one reset dispatch, one cull/LOD dispatch, and one
  indirect mesh dispatch per measured frame, independent of logical world extent,
  candidate count, visible count, enabled archetypes, selected LOD, and meshlet count.
- Candidate, visible-object, dispatch-argument, meshlet, counter, timestamp, and readback
  storage are explicitly bounded before execution. An overflow or unsupported catalog is
  an error, not permission to expand work on the CPU.
- The meshlet mode requires D3D12 mesh-shader tier 1 and shader model 6.6 or newer. It has
  no indexed-draw or CPU-expansion fallback. Earlier proxy paths remain separate
  regression modes rather than compatibility branches inside meshlet execution.
- Real geometry preserves the accepted reverse-Z, color capture, and `R32_UINT` semantic
  object-ID contracts. Aggregate GPU counters must exactly match a deterministic CPU
  oracle in validation workloads; the oracle is not part of ordinary frame submission.
- GPU timings are reported as distributions and scaling evidence. Values measured on the
  reference machine are observations, not architecture thresholds.

## Consequences

The first real static-geometry path now consumes the same immutable snapshots as cooked
streaming, while logical-world growth and geometry diversity do not increase CPU command
submission. Mesh shaders are a hard reference-platform dependency for this path, and
catalog data consumes persistent default-heap storage even when an archetype is not
visible.

The accepted catalog is intentionally synthetic. Production asset metadata, imported
meshes, materials, textures, occlusion, mesh streaming, skeletal animation, and content
schemas remain unaccepted. Later experiments may replace catalog construction and object
selection only while preserving bounded buffers, GPU work generation, fixed submission,
semantic IDs, and immutable snapshot consumption.

## Evidence

- [Experiment 0009](../../experiments/0009-gpu-meshlet-scene/README.md) records the
  capability gate, catalog hash and bounds, exact GPU/oracle counters, active/logical/
  archetype/LOD sweeps, deterministic captures, movement, restart, and prior-path
  regressions.
