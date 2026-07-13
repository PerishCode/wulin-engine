# ADR 0017: GPU Terrain LOD Stitching

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0016 established exact same-resolution terrain continuity, bounded publication,
and one fixed mesh submission. It deliberately did not establish heterogeneous patch
resolution. Selecting fewer samples is not sufficient: an independently tessellated
fine edge can leave the line emitted by its coarse neighbor and expose a crack even
when both source payloads share identical samples.

Experiment 0014 tested whether three patch levels could be selected from GPU-visible
scalar state while preserving an exact common geometric edge, without changing the
terrain payload, streaming cache, semantic IDs, or CPU submission shape.

## Decision

- Each 8x8-cell terrain patch selects sample step 1, 2, or 4. Automatic selection uses
  integer Chebyshev distance from one camera patch address and ordered near/middle band
  radii. Forced levels are retained only as controlled experiment settings.
- CPU supplies the camera patch, two radii, and optional forced level. It does not
  enumerate patch levels, transitions, vertices, triangles, or indirect work before
  submission. The GPU amplification shader classifies all 400 patch groups.
- Ordered distance bands guarantee adjacent LOD delta at most one because neighboring
  Chebyshev distances differ by at most one. Invalid bands and forced levels are
  rejected before settings mutation.
- At a transition, the coarser edge owns the geometric segmentation. Every fine-only
  edge vertex interpolates the same two coarse source heights and the same transformed
  clip-space endpoints. Corners remain original shared samples. Skirts are not used.
- Enabled LOD retains one fixed `[400,1,1]` mesh dispatch and adds one fixed
  `[400,2,1]` patch-edge validation dispatch. The accepted `[25,2,1]` raw region-edge
  validation remains unchanged. All shapes are independent of the selected
  distribution and emitted geometry.
- A separate 64-byte UAV records level and transition evidence. Requested probes alone
  read it back and compare all 760 patch edges at nine finest-grid positions against an
  independent CPU oracle. Normal frame submission does not consume readback.
- LOD remains default-disabled. The disabled path and forced LOD 0 must reproduce ADR
  0016 geometry, attachments, mapping, payload, resources, and submission evidence.
- The terrain payload, 50-slot cache, immutable publication, object IDs, and workbench
  ownership remain unchanged. This decision does not promote a general terrain API.

## Consequences

The canonical automatic workload classifies 25/144/231 patches at LOD 0/1/2 and emits
7,704 vertices and 9,656 triangles instead of 32,400 and 51,200. This removes 76.222
percent of vertices and 81.141 percent of triangles while preserving one fixed mesh
dispatch. Fifty-nine transition edges project 158 fine vertices; CPU and GPU compare
all 6,840 registered edge positions with zero mismatch.

This accepts exact geometric continuity and emitted-work reduction, not lower GPU time.
The fixed validation dispatch, fixed 400-group mesh dispatch, requested instrumentation,
and pixel workload dominate this small laboratory scene. Optimized timings did not
decrease monotonically with emitted geometry, so no runtime speed claim is promoted.

Distance bands, three fixed levels, fixed 400-group overdispatch, and experiment stats
are registered evidence, not a production terrain policy. Geomorphing, authored error,
normal or material continuity, lighting quality, anisotropic selection, clipmaps,
virtual texturing, horizon or occlusion culling, collision, navigation, grounding,
object composition, and broad compatibility remain unaccepted.

## Evidence

- [Experiment 0014](../../experiments/0014-gpu-terrain-lod-stitching/README.md) records
  disabled compatibility, forced and automatic sweeps, exact CPU/GPU edge arithmetic,
  camera and band movement, visual captures, publication holds, cache movement,
  restart determinism, and optimized distributions.
- Disabled and forced LOD 0 reproduce the accepted Experiment 0013 color, object-ID,
  diagnostic, mapping, payload, geometry, resource, and submission evidence exactly.
- Every forced, automatic, band, camera, movement, teleport, held, revisit, and restarted
  probe classifies 400 patches and exactly matches the independent CPU aggregate and
  edge oracle. Maximum adjacent delta is one and mismatch count is zero.
- Transition, corner, and grazing captures contain no unknown semantic ID or visible
  background crack. Restart reproduces canonical automatic probe and attachments.
- I/O and copy holds preserve the complete prior LOD snapshot until immutable
  publication. Classification for the revisited logical addresses is independent of
  changed physical cache slots.
- The typed Sidecar control path rejects invalid settings before mutation; the CPU and
  GPU oracles independently detect any divergent duplicated edge data. Experiment 0013,
  affected prior paths, and repository guard pass without validation error, device
  removal, fallback, or residual process.

## Environment

The ignored acceptance report records repository and toolchain revisions, Windows,
adapter and driver, D3D12 capabilities, debug-layer state, pack and payload hashes,
settings, cameras, geometry and exact edge evidence, attachment hashes, resource sizes,
process identities, optimized distributions, and namespace cleanup.

## Reproduction

```powershell
runseal :terrain-lod
```

The command recooks the unchanged canonical pack, runs debug-layer correctness and
release timing through isolated Sidecar namespaces, and writes the ignored structured
report to `out/captures/0014-gpu-terrain-lod-stitching/acceptance.json`.
