# Experiment 0009: GPU Meshlet Scene Execution

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md),
  [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md),
  [ADR 0009](../../docs/adr/0009-resident-region-storage.md),
  [ADR 0010](../../docs/adr/0010-asynchronous-region-publication.md),
  [ADR 0011](../../docs/adr/0011-cooked-region-storage.md),
  [ADR 0012](../../docs/adr/0012-gpu-meshlet-scene-execution.md)

## Hypothesis

A bounded GPU scene pipeline can transform the accepted cooked resident snapshot into
frustum-culled, LOD-selected meshlet work and execute all visible static geometry with
one indirect mesh dispatch whose CPU command shape is independent of candidate count,
visible count, archetype diversity, and meshlet count.

## Scope

The experiment replaces the asynchronous resident path's procedural six-vertex proxy
with real indexed static geometry represented by a deterministic GPU meshlet catalog.
It introduces eight visually distinct archetypes, three LODs per archetype, GPU object
culling and LOD selection, a compact visible-object record, meshlet expansion through
amplification and mesh shaders, indirect `DispatchMesh`, exact GPU counter readback,
and Sidecar controls and evidence for the resulting workload.

The accepted `WLRGN001` format, cooked worker, copy queue, cache reservation, immutable
publication, region descriptors, camera, reverse-Z convention, semantic object-ID
attachment, and bounded perception contract remain unchanged. Archetype and orientation
are derived deterministically from stable object identity; this is an experiment input,
not a proposed scene format.

Asset import, a new cooked-format version, arbitrary scene schemas, skeletal animation,
materials, textures, lighting, shadows, transparency, occlusion culling, mesh or texture
streaming, ECS, gameplay, seamless traversal, multiple I/O workers, and compatibility
fallbacks are excluded.

## Workload

1. Query and record the reference adapter's mesh-shader tier and highest accepted shader
   model before creating the meshlet pipeline. Unsupported capability fails the experiment.
2. Build a deterministic catalog of eight static archetypes and three geometric LODs,
   partitioned into meshlets with explicit vertex and primitive bounds.
3. Publish the canonical cooked radius-2 snapshot centered at `[64,64]`: 25 regions and
   25,600 candidate objects.
4. Cull candidates and select LOD entirely on the GPU, compact visible object records,
   finalize one indirect mesh dispatch, and render catalog meshlets through amplification,
   mesh, and pixel shaders.
5. Compare visible-object, per-LOD, dispatched-meshlet, emitted-vertex, and emitted-triangle
   counters against a deterministic CPU oracle for the same snapshot and camera.
6. Revisit `[64,64]`, restart through Sidecar, and require identical workload counters,
   color capture, raw object-ID attachment, diagnostic perception image, and semantic joins.
7. Sweep active region side lengths `1`, `3`, and `5`, logical-world side lengths `32`,
   `64`, and `128`, then sweep archetype masks and forced LODs without changing the
   number of CPU draw or dispatch submissions.

## Controlled variables

- Reference platform, adapter selection, D3D12 feature level, Agility SDK, DXC, debug
  layer, swap chain, capture dimensions, camera, region dimensions, records per region,
  active mapping, and cooked payload bytes remain fixed.
- Catalog generation uses fixed topology recipes, ordering, winding, meshlet limits,
  bounds, and hashes. No runtime randomness or file-dependent ordering is permitted.
- Stable physical object reference determines archetype and orientation. Camera-space
  projected extent determines LOD unless the sweep explicitly forces one LOD.
- Candidate enumeration follows the published active-slot order. Atomics may determine
  compacted execution order, but output pixels, semantic IDs, aggregate counters, and
  catalog selection must not depend on that order.
- The direct command list records one compute cull dispatch and one indirect mesh dispatch
  per measured iteration. CPU code does not enumerate visible objects or meshlets.

## Metrics

- Adapter, mesh-shader tier, shader model, catalog hash, catalog bytes, archetype/LOD
  counts, meshlet/vertex/primitive counts, and maximum meshlet limits.
- Logical, active, candidate, visible, rejected, per-LOD, dispatched-meshlet,
  emitted-vertex, and emitted-triangle counts, with CPU-oracle deltas.
- Direct compute dispatch count, indirect mesh dispatch count, CPU submission shape,
  GPU cull/LOD time, GPU mesh execution time, GPU total time, and P50/P95/P99 over the
  fixed sample window.
- Published center, resident slots, payload hash, frame index, color/object-ID/diagnostic
  hashes, semantic samples, renderer errors, device removal reason, and cleanup state.

## Acceptance criteria

- The reference adapter reports mesh-shader tier 1 or newer and accepts shader model 6.6
  or newer. The implementation contains no indexed-draw or CPU-expansion fallback path.
- The catalog contains exactly eight archetypes and three LODs each, passes meshlet bound
  validation, reproduces its full byte hash across two independent builds, and contains
  strictly less geometry at each successively coarser LOD for every archetype.
- For every canonical movement, active-side, logical-world-side, archetype-mask, and
  forced-LOD sample, all
  GPU aggregate counters exactly equal the CPU oracle and never exceed allocated bounds.
- Every measured frame records one cull dispatch and one indirect mesh dispatch. CPU
  submission counts remain constant across candidate, visible, archetype, and meshlet sweeps.
- Initial, cached revisit, and restart runs reproduce exact workload counters, color hash,
  raw object-ID hash, diagnostic hash, and bounded semantic perception results.
- The radius-2 canonical workload retains 25 regions and 25,600 candidates, performs real
  catalog vertex/primitive reads, selects at least two LODs and all enabled archetypes, and
  dispatches more meshlets than visible objects without device or validation errors.
- The workload sweeps demonstrate bounded resources and report P50/P95/P99 GPU timings
  without changing command shape. Timing values are observations, not pass thresholds.
- Existing `runseal :cooked-region`, `runseal :async-region`, `runseal :resident-stream`,
  and `runseal :guard` regressions pass. Final Sidecar status contains no target or broker
  process.

## Environment

Accepted on the reference Windows/D3D12 platform with an NVIDIA GeForce RTX 4070 Ti
SUPER, feature level 12_1, mesh-shader tier 1, shader model 6.9, and the D3D12 debug
layer enabled. The repository-pinned Agility SDK, DXC, and Rust toolchain were used.

## Reproduction

Run from the repository root:

```powershell
runseal :meshlet-scene
```

## Results

Accepted results on 2026-07-13 for revision `gpu-meshlet-scene-v1`:

The deterministic catalog contains 8 archetypes, 3 LODs per archetype, 1,872 vertices,
88 meshlets, 2,704 meshlet vertex indices, and 3,648 primitive entries in 57,152 GPU
bytes. Its independently reproduced full-byte SHA-256 is
`9553748209f9de17e9b524b1c21080404f32df57be62959714b58db1121f0a4e`.

The canonical radius-2 snapshot produced these exactly matching GPU and CPU-oracle
counts:

| Candidates | Visible | Rejected | LOD 0/1/2 | Meshlets | Vertices | Triangles |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 25,600 | 18,928 | 6,672 | 6,243 / 12,640 / 45 | 69,270 | 2,399,960 | 3,316,944 |

Every probe recorded one reset dispatch, one cull/LOD dispatch, and one indirect mesh
dispatch. All eight archetypes were observed. Eight timing samples, each averaging 16
probe iterations, reported:

| GPU stage | Median | P95 | P99 |
| --- | ---: | ---: | ---: |
| Cull and LOD | 0.014208 ms | 0.014272 ms | 0.014272 ms |
| Mesh execution | 0.307328 ms | 0.309312 ms | 0.309312 ms |
| Total | 0.321728 ms | 0.323712 ms | 0.323712 ms |

Active radii 0, 1, and 2 produced 1,024, 9,216, and 25,600 candidates, with GPU counts
equal to the oracle in every case. Logical sides 32, 64, and 128 represented 1,048,576,
4,194,304, and 16,777,216 logical instances while retaining the same 25,600 candidates,
18,928 visible objects, and 69,270 meshlets. Archetype masks 1, 15, and 255 and forced
LODs 0, 1, and 2 also retained exact oracle equality and fixed command shape.

Initial, cached revisit, and a new Sidecar process reproduced color SHA-256
`a71c79cb4e12ec615b1d4ff66c940ed96c1ea334cf64911922b2d9ed036a1334`, raw object-ID
SHA-256 `af9bad8ed8ee938caa4132508cd111451a88d72b4a1f438aaab4e121af2cfc44`, and diagnostic
PNG SHA-256 `dbbaa2ccb07e6478265a0b2753b65c75f53d44fcd7f656ca35e19f11938ab707`.
Each perception pass joined 20 visible region semantics with no unknown IDs.

`runseal :resident-stream`, `runseal :async-region`, `runseal :cooked-region`, and
`runseal :guard` passed after the implementation. No renderer error, device removal, or
final Sidecar process remained. The complete generated report is
`out/captures/0009-gpu-meshlet-scene/acceptance.json`.

## Conclusion

Accepted. A cooked immutable resident snapshot can drive culling, LOD selection,
visible-object compaction, amplification, and real meshlet emission without CPU-visible
work expansion. Logical-world extent, active-set size, archetype diversity, and LOD
complexity change GPU work counts while leaving direct command submission fixed.

## Promotion

ADR 0012 promotes the meshlet catalog bounds, GPU scene execution stages, capability
gate, fixed indirect submission shape, bounded resources, exact validation oracle, and
semantic output contract. Deterministic archetype derivation, synthetic geometry, and
forced-LOD controls remain laboratory inputs.
