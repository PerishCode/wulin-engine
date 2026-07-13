# Experiment 0010: GPU Skeletal Crowd Execution

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md),
  [ADR 0010](../../docs/adr/0010-asynchronous-region-publication.md),
  [ADR 0011](../../docs/adr/0011-cooked-region-storage.md),
  [ADR 0012](../../docs/adr/0012-gpu-meshlet-scene-execution.md)

## Hypothesis

A bounded GPU scene pipeline can classify animated visible objects, compact shared pose
keys, evaluate hierarchical skeleton palettes, and skin real meshlet geometry without
CPU-visible character, pose, bone, or meshlet enumeration. CPU submission remains fixed
while animated count, pose diversity, bone count, and emitted geometry change.

## Scope

The experiment extends the accepted cooked resident and meshlet scene path with a
deterministic animation catalog, packed four-influence skin bindings, a maximum 128-bone
hierarchy, eight clips, quantized pose keys, GPU active-pose compaction, indirect pose
evaluation, and linear blend skinning in the mesh shader.

Shared mode evaluates each active `(clip, phase)` pose once and lets visible objects
reference its palette. Unique mode evaluates one seeded pose per animated visible object
using the same hierarchy and skinning path. The unique path is the bounded worst case;
shared mode proves that expected crowd work scales with pose diversity rather than
character count.

The static `WLMSH001` meshlet catalog, `WLRGN001` cooked format, copy queue, immutable
publication, region descriptors, reverse-Z, object-ID target, capture, perception, and
Sidecar lifecycle remain unchanged. Laboratory animation state is derived from stable
object identity and explicit operator time; it is not a proposed scene schema.

Imported assets, a cooked-format revision, materials, textures, normals, lighting,
shadows, occlusion, root motion, IK, ragdolls, morph targets, cloth, animation graphs,
compression policy, retargeting, gameplay, seamless traversal, and compatibility
fallbacks are excluded.

## Workload

1. Build a deterministic animation catalog containing a 128-bone maximum hierarchy,
   inverse bind transforms, eight clips, 64 samples per clip, and one packed skin binding
   for every accepted meshlet-catalog vertex.
2. Publish the canonical cooked radius-2 snapshot at `[64,64]`, producing 25 regions,
   25,600 candidates, and the accepted frustum-visible set.
3. Cull and select LOD on the GPU, classify animated objects, mark shared pose keys, and
   preserve one compact visible-object record per accepted object.
4. Compact active shared pose keys with one fixed dispatch, then issue one indirect
   compute dispatch whose thread groups evaluate complete skeleton palettes.
5. Dispatch accepted meshlets indirectly and apply four-weight skinning from the GPU
   palette before the existing instance transform and reverse-Z projection.
6. Compare GPU object, pose, bone, meshlet, vertex, triangle, and influence counters with
  a deterministic CPU oracle. Compare sampled palette matrix elements within absolute
  tolerance `0.00002`.
7. Sweep animated fractions `0`, `25`, `50`, and `100` percent; bone counts `16`, `32`,
   `64`, and `128`; shared phase counts `1`, `8`, and `64`; and fully unique poses.
8. Set deterministic times `0`, `11`, and `0`, move to an adjacent center, revisit,
   restart through Sidecar, and collect exact visual and semantic evidence.

## Controlled variables

- Reference adapter, mesh-shader capability, shader model, Agility SDK, DXC, debug layer,
  swap chain, capture dimensions, camera, region records, active mapping, cooked bytes,
  static meshlet topology, LOD thresholds, and object IDs remain fixed.
- Catalog generation has fixed parent ordering, hierarchy depths, bind transforms, clip
  sample ordering, skin influence ordering, quantization, and full-byte hash.
- The operator supplies an integer animation tick. There is no wall-clock sampling,
  frame-delta accumulation, or runtime randomness.
- Shared pose identity is `(clip, quantized phase)`. Unique poses add stable object
  identity to the same deterministic clip sample.
- GPU atomic order may change compact buffer order, but palette destinations, object
  transforms, output pixels, aggregate counters, and semantic IDs must not depend on it.
- The measured direct command list always records one reset dispatch, one cull dispatch,
  one pose-key compaction dispatch, one indirect pose dispatch, and one indirect mesh
  dispatch. CPU code does not inspect visibility or pose counts before submission.

## Metrics

- Animation catalog hash and bytes; rig, clip, sample, bone, hierarchy-depth, skin-stream,
  and influence counts; packed layout sizes and fixed GPU capacities.
- Candidate, visible, rejected, animated, static, active-pose, reused-pose, evaluated-bone,
  meshlet, emitted-vertex, emitted-triangle, and skin-influence counts with oracle deltas.
- Reset, cull, pose-compact, indirect-pose, and indirect-mesh submission counts.
- GPU cull/classify, pose compact/evaluate, mesh skin/render, and total P50/P95/P99 times.
- Palette allocation and write bytes, pose-bitset and active-list high-water marks, and
  unique-pose worst-case memory.
- Time, center, process identity, color/object-ID/diagnostic hashes, semantic joins,
  sampled palette deltas, renderer errors, device removal, and cleanup state.

## Acceptance criteria

- The animation catalog reproduces its complete byte hash, validates parent-before-child
  hierarchy order, contains exactly 8 clips and 64 samples per clip, supports the 128-bone
  maximum, and stores four normalized packed influences per vertex.
- Every workload sample keeps GPU counters exactly equal to the CPU oracle, sampled GPU
  palettes within the pre-registered tolerance, and all writes within fixed capacities.
- Shared mode evaluates no more than `8 * phaseCount` poses and reports fewer evaluated
  bones than animated objects times bone count whenever at least two objects reuse a pose.
- Fully unique mode evaluates exactly one pose per animated visible object, remains within
  the 25,600-pose capacity at 128 bones, and completes without validation or device errors.
- The canonical radius-2, 100-percent animated, 64-bone workload retains 25,600 candidates,
  the accepted static visibility/LOD/meshlet counts, performs non-zero palette and
  four-influence work, and records exactly the fixed five-command submission shape.
- Time `0` and `11` produce different color captures while preserving object-ID semantics;
  returning to time `0`, cached revisit, and restart reproduce exact counters, color hash,
  raw object-ID hash, diagnostic hash, and bounded semantic joins.
- Animated-fraction, bone-count, pose-diversity, movement, and geometry-LOD sweeps report
  bounded resources and P50/P95/P99 GPU distributions without CPU submission changes.
- Existing `runseal :meshlet-scene`, `runseal :cooked-region`, `runseal :async-region`,
  `runseal :resident-stream`, and `runseal :guard` regressions pass. Final Sidecar status
  contains no target or broker process.

## Environment

The final report records revision, Windows build, adapter and driver, feature level,
mesh-shader tier, shader model, Agility SDK, DXC, Rust toolchain, debug-layer state,
catalog hashes, workload sweeps, GPU distributions, visual hashes, process identities,
and cleanup.

## Reproduction

```powershell
runseal :skeletal-crowds
```

## Results

The canonical run passed on the RTX 4070 Ti SUPER reference adapter with D3D12 feature
level 12_1, mesh-shader tier 1, shader model 6.9, Agility SDK 1.619.4, and NVIDIA driver
32.0.16.1074. Correctness used a debug-layer workbench; timing used an independently
stamped release workbench with the debug layer disabled. The final timing pass preheated
the unique 128-bone workload for 2,000 ms, warmed each workload for 250 ms, and recorded
32 samples per distribution.

The animation catalog reproduced SHA-256
`cc075037175990f29083ad1fc63823c1a77002d7aeccfbc429eee4f54de22a6e` and occupied
3,169,920 GPU bytes. The unchanged meshlet catalog reproduced SHA-256
`9553748209f9de17e9b524b1c21080404f32df57be62959714b58db1121f0a4e`.
Execution storage was bounded at 158,005,616 bytes, including a 157,286,400-byte palette
capable of 25,600 unique 128-bone poses.

The canonical shared workload retained 25,600 candidates, 18,928 visible objects,
69,270 meshlets, 2,399,960 emitted vertices, 3,316,944 emitted triangles, and 9,599,840
skin influences. It compacted 512 active poses, reused them for 18,416 characters, and
evaluated 32,768 bones. Every GPU aggregate exactly matched the CPU oracle and every
sampled palette remained within absolute tolerance `0.00002`.

Release timing observations were:

| Workload | P50 GPU total | P95 | P99 |
| --- | ---: | ---: | ---: |
| 100% animated, shared, 64 bones, 64 phases, automatic LOD | 4.50 ms | 12.94 ms | 13.61 ms |
| 100% animated, shared, 128 bones, 64 phases, automatic LOD | 4.63 ms | 12.97 ms | 13.98 ms |
| 100% animated, unique, 128 bones, automatic LOD | 6.30 ms | 12.72 ms | 21.17 ms |
| 100% animated, shared, 64 bones, forced LOD 0 | 5.21 ms | 6.56 ms | 10.10 ms |
| 100% animated, shared, 64 bones, forced LOD 2 | 2.32 ms | 2.45 ms | 10.80 ms |

The unique workload evaluated 18,928 poses and 2,422,784 bones, wrote 116,293,632 palette
bytes, and stayed within all fixed capacities with the same five-command submission.
Time 0 and 11 changed color and object-ID coverage while retaining 20 valid region
semantic joins and no unknown IDs. Returning to time 0, cached revisit, and process
restart reproduced exact color, raw object-ID, diagnostic, counter, and catalog hashes.
Both correctness and benchmark Sidecar namespaces were empty after cleanup.

## Conclusion

Accepted. GPU animation classification, pose sharing, bounded hierarchical palette
evaluation, and four-weight meshlet skinning can sustain the representative crowd
without CPU work enumeration or submission growth. Shared work scales with active pose
diversity, while the fully unique path exposes a bounded and measurable upper workload.

The release distributions contain desktop scheduling and power-state tails, especially
in the unique-pose P99. They are retained as observations rather than hidden by averages;
the experiment accepts the architecture and bounded load behavior, not a frame-time SLA.

## Promotion

ADR 0013 promotes GPU pose classification, pose sharing, bounded hierarchical palette
evaluation, fixed five-stage submission, and meshlet skinning contracts. Synthetic clip
generation, stable-key animation assignment, integer operator time, the 157 MB allocation
policy, and sweep controls remain laboratory inputs.
