# Experiment 0015: Atomic Terrain-Object Composition

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: ADR 0018

## Hypothesis

The independently accepted terrain and animated-object streams can stage and publish
one matched immutable region snapshot, ground every generated object to the exact
full-resolution terrain triangle on the GPU, and render both classes through one shared
reverse-Z depth and semantic attachment contract. CPU submission can remain independent
of candidate, visible-object, terrain-geometry, and pixel counts while an incomplete or
failed half of the pair leaves the complete old composition unchanged.

## Scope

This experiment composes the accepted Experiment 0010 direct skeletal mesh path with
the accepted Experiment 0013 full-resolution streamed terrain path. Experiment 0014 LOD
remains default-disabled during composition so grounding is measured against the exact
rendered 0.5-meter source triangles rather than an unresolved simplified-surface error
policy.

The generated instance fixture places one object at the center of each of the 32x32
terrain cells in a region. Terrain triangulation splits each cell along the edge from
`v10` to `v01`, so the center lies exactly on that diagonal. Its ground height is:

```text
ground_numerator = height(x + 1, z) + height(x, z + 1)
ground_meters = ground_numerator / 512
```

The numerator is exact signed integer evidence: the factor of two is the diagonal
midpoint and terrain source height units are 1/256 meter. GPU culling writes one bounded
ground numerator per candidate; mesh emission consumes the same value. CPU reconstructs
all values independently only for a requested probe.

The composition path uses direct skeletal color and semantic emission. Surface
visibility resolve, material reconstruction, prior-frame occlusion, terrain LOD
grounding, arbitrary object coordinates, feet/IK, slope orientation, physics, collision,
navigation, spatial partition redesign, authored placement, vegetation, shadows,
transparency, and broad compatibility are excluded.

## Workload

1. Run terrain-only and skeletal-only canonical workloads through their existing modes.
   Reproduce their accepted probes and attachments before composition is enabled.
2. Schedule the canonical radius-2 terrain pack and procedural instance regions under
   one composition transaction. Stage both copy completions and publish only when both
   immutable snapshots have the same `LoadConfig` and composition transaction token.
3. Render 25 terrain regions, 400 full-resolution patches, and 25,600 object candidates.
   Terrain and skeletal subpasses share the existing color, `R32_UINT` object-ID, and
   reverse-Z depth targets; composition clears each attachment exactly once.
4. Decode every candidate's terrain region and cell on the GPU during the existing
   fixed cull dispatch. Record the 25,600 signed ground numerators and compare them
   exactly against the CPU oracle. Require the mesh path to consume the same buffer.
5. Render terrain-first and object-first variants from the same snapshot. Require
   byte-identical color and object-ID attachments, identical visible-object aggregates,
   both semantic ID ranges in the final frame, and no unknown ID.
6. Capture default, near-contact, ridge, valley, region-edge, four-region-corner, and
   grazing cameras. Record ground samples, terrain/object occlusion samples, attachments,
   semantic joins, and diagnostic images. No visible object may retain the old flat
   `y=0` fixture base.
7. Move the paired center `[64,64] -> [65,64] -> [65,65]`, revisit `[64,64]`, and
   teleport to `[96,96]`. Require row-major logical agreement despite independent
   physical terrain and instance cache slots.
8. Hold terrain I/O, terrain copy, and instance copy independently after the other half
   is ready. The complete old pair, grounding evidence, attachments, and semantic joins
   must remain unchanged until the matching half can publish.
9. Corrupt a requested terrain chunk after the instance half stages. The complete pair
   must roll back without partial publication or slot loss, preserve the old frame, and
   permit a valid retry without restart.
10. Restart through Sidecar and reproduce the canonical pair token-independent probe and
    attachments. Verify both debug and benchmark namespaces are fully cleaned.
11. In the release namespace, warm and collect 32 requested probes for terrain-only,
    skeletal-only, terrain-first composition, and object-first composition. Report
    terrain validation/raster, skeletal classify/pose/mesh, grounding, combined GPU,
    publication, and output-work distributions separately.

## Controlled variables

- Terrain format V1, canonical pack, payload hashes, 50 terrain slots, 50 instance
  slots, radius 2, 25 active regions, row-major logical order, camera, 1280x720 extent,
  and semantic ID ranges remain unchanged.
- Terrain LOD is disabled and emits 400 patches, 32,400 vertices, and 51,200 triangles.
  The accepted `[25,2,1]` raw terrain seam dispatch remains enabled.
- Skeletal settings are 100 percent animated, 64 bones, 64 shared phases, tick 0, unique
  poses disabled, and automatic skeletal mesh LOD unless a controlled sweep says
  otherwise. Existing fixed classify, compact, pose, cull, and indirect mesh stages
  remain the owning execution path.
- One composition coordinator owns a monotonically increasing pair token. A staged
  terrain or instance publication is not renderer-visible until both token and complete
  `LoadConfig` match. Standalone terrain and instance publication retain their accepted
  behavior outside composition mode.
- Old terrain and instance slots remain protected until a pair commits or rolls back.
  A new pair cannot start while another pair is reserved, copying, staged, or failed
  without cleanup.
- CPU supplies mappings, scalar settings, and fixed dispatches. It does not enumerate
  candidate grounds, visible objects, emitted meshlets, terrain patches, or pixels before
  submission.
- Correctness uses the debug-layer Sidecar profile. Timing uses the release profile with
  validation disabled. Probe readback is requested-only and bounded to 25,600 signed
  ground numerators plus existing accepted probe resources.

## Metrics

- Pair token; terrain and instance transaction IDs; reservation, I/O, copy, staged,
  commit, rollback, and total publication durations; protected and changed slot counts.
- Terrain and instance config/mapping/payload hashes, logical region equality, physical
  slot divergence, generation, and pair publication count.
- Ground candidate count, numerator SHA-256, minimum/maximum, CPU/GPU mismatch count,
  first mismatch, readback bytes, and ground-buffer allocation bytes.
- Terrain patch/vertex/triangle aggregates; skeletal source-visible, final-visible,
  animated, pose, meshlet, vertex, triangle, and influence aggregates.
- Fixed CPU dispatch shape, descriptor counts, resource bytes, transitions, clears, and
  readback ownership.
- Color, object-ID, diagnostic, semantic, bounded contact, terrain-occludes-object, and
  object-occludes-terrain evidence by render order, camera, movement, hold, failure,
  retry, and restart.
- Terrain validation/raster, fused grounding/classify, skeletal pose/mesh, combined GPU,
  and publication P50/P95/P99 distributions. Grounding remains fused into the accepted
  skeletal cull dispatch; no duplicate timing-only dispatch is introduced.
- Validation errors, device removal, hidden fallback, process identities, and final
  Sidecar namespace cleanup.

## Acceptance criteria

- Terrain-only and skeletal-only modes reproduce their accepted canonical evidence.
  Composition does not replace or weaken either standalone owner.
- A composition frame is produced only from terrain and instance snapshots with the
  same pair token and exact `LoadConfig`. No frame observes one new half and one old
  half, including every hold, failure, movement, teleport, and restart workload.
- GPU and CPU produce exactly 25,600 identical signed ground numerators for every
  published pair. Mismatch count is zero, the mesh path consumes the validated ground
  buffer, and requested readback remains bounded and absent from normal frames.
- The final frame contains known terrain and object semantics. Terrain-first and
  object-first color and object-ID attachments are byte-identical, proving one shared
  reverse-Z depth contract and one clear owner rather than painter-order dependence.
- Contact cameras show grounded bases without flat-plane remnants, visible floating,
  terrain penetration, background cracks, or unknown IDs. Grounding follows logical
  region/cell addresses and is independent of either cache's physical slots.
- Terrain I/O, terrain copy, and instance copy holds preserve the exact complete old
  pair. Terrain corruption after instance staging rolls back both halves, preserves old
  attachments and protected slots, and permits a valid retry.
- CPU records the registered fixed terrain and skeletal submissions without reading
  ground, visibility, geometry, or pixel counts. Submission is independent of output
  work and render order.
- Optimized measurements report distributions and work separately. Lower frame time is
  not required for acceptance; unexplained synchronization, unbounded growth, device
  removal, validation error, or hidden fallback is failure.
- Experiments 0007-0014 and `runseal :guard` pass after final implementation with no
  residual debug or benchmark process.

## Environment

The final report records repository revision and dirty state, Windows, adapter and
driver, feature level, shader model, Agility SDK, DXC, Rust toolchain, debug-layer state,
pack hashes, both cache allocations, pair and transaction IDs, mappings, grounding
evidence, cameras, attachments, semantics, fixed submissions, timings, process
identities, and cleanup.

## Reproduction

```powershell
runseal :composition
```

The command recooks the canonical terrain pack, runs debug-layer correctness and
release timing through isolated Sidecar namespaces, and writes the ignored structured
report to
`out/captures/0015-atomic-terrain-object-composition/acceptance.json`.

## Results

The canonical pair publishes 25 terrain regions and 25 instance regions under token 1.
Terrain occupies physical slots 0 through 24 while composition instances use slots 49
through 25, so all 25 logical mappings differ physically. The GPU writes exactly 25,600
signed ground numerators. CPU and GPU hashes are both
`7e6779f8a69768b2c883aa339865c823d00dcaed63e3d6fa588e823a1e0e162c`,
the range is `[-965, 1279]`, and mismatch count is zero.

The grounded canonical camera reports 10,503 visible animated objects, 512 shared
poses, 45,833 meshlets, 1,791,346 emitted vertices, 2,463,576 emitted triangles, and
7,165,384 skin influences. Every GPU aggregate exactly equals the grounded CPU oracle.
Terrain remains the full-resolution 400-patch, 32,400-vertex, 51,200-triangle path with
zero raw edge mismatch.

Terrain-first and object-first frames are byte-identical: color hash
`0f3648042c98507c7e956779e1b1f2390c7ddd79185262a66553563448f7e8af`
and object-ID hash
`00b49db9b28d9669540d40c6e9f1cae808d6ecf6d2f44a3b378004e2d40c5c1d`.
Both semantic ranges are present and unknown-ID count is zero. The camera collection
records default, near-contact, ridge, valley, region-edge, four-region-corner, and
grazing views without an attachment or semantic validation failure.

Instance-copy hold reaches terrain `staged` with instance `in-flight`; terrain-I/O and
terrain-copy holds reach instance `staged` with terrain `in-flight`. Every held probe
and attachment is byte-identical to the complete old pair. Corrupting terrain region
12642 after the instance half stages records terrain `failed`, instance `discarded`,
preserves the old pair, and permits a valid teleport retry without restart. Movement,
revisit, teleport, and Sidecar restart all preserve exact logical grounding despite the
different physical mappings.

Release measurements contain 32 requested probes per mode. Terrain-first composition
combined GPU P50/P95/P99 is 0.409/0.451/0.452 ms; object-first is
0.384/0.396/0.400 ms. Their fused ground-and-cull P95 values are 0.014 and 0.026 ms.
Cached pair publication P50/P95/P99 is 25.310/25.662/25.933 ms terrain-first and
25.559/26.022/26.052 ms object-first. Terrain-only and skeletal-only accepted probes
also pass. These distributions characterize this instrumented workload; no speedup or
preferred render order is promoted.

## Conclusion

The hypothesis passes. Independently streamed terrain and skeletal instance data can be
staged under one pair token, atomically made renderer-visible, addressed through
different physical cache mappings, and joined in the existing GPU cull. Exact integer
grounding drives both culling and mesh emission. One shared reverse-Z depth and
object-ID contract makes output independent of subpass order.

The accepted boundary is deliberately narrow. It does not accept arbitrary placement,
terrain LOD grounding, surface resolve, occlusion, slope alignment, IK, physics,
collision, navigation, transparency, shadows, or a general scene/ECS abstraction.

## Promotion

Promote matched pair staging/commit, packed logical-to-physical terrain/instance
mapping, requested-only exact grounding evidence, and shared direct-pass attachment
ownership as the baseline for the next scene-composition experiment. Keep the
coordinator and grounding path workbench-owned until another experiment requires a
reusable engine boundary.
