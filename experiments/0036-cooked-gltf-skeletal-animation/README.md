# Experiment 0036: Cooked glTF Skeletal Animation

Status: Accepted

## Hypothesis

The pinned Fox skin, 24-joint hierarchy, inverse-bind data, and three source clips can be
cooked offline into a second fixed rig bank and produce natural articulated deformation
through the existing GPU pose, meshlet skin, surface, and occlusion path without changing
the signed object schema or introducing a second renderer.

## Scope

This experiment extends the accepted Fox geometry payload with its exact four-influence
vertex bindings and extends `animation-catalog` with one imported rig. Both fixture and
imported rigs occupy fixed 128-bone, eight-clip, 64-sample banks in the existing buffers.
Archetypes 0 through 6 retain the existing fixture bank byte-for-byte; archetype 7 selects
the imported bank. Shared pose keys add one rig bit, increasing the fixed key domain from
512 to 1,024 while retaining one compact/indirect pose dispatch and one skin path.

Source clips `Survey`, `Walk`, and `Run` occupy imported clip slots 0, 1, and 2. Slots 3
through 7 are deterministic aliases `[Survey, Walk, Run, Survey, Walk]` so the existing
three-bit authored clip field remains total. The imported cooker profile explicitly authors
Walk, a deterministic phase offset, and zero variation for every imported object.

General glTF skins, multiple rigs per asset, clip blending, transitions between clips,
root-motion policy, source-duration playback, animation compression, dynamic rig streaming,
schema changes, and Wulin content are out of scope.

## Workload

1. Verify the pinned JSON/BIN hashes, one skin, 24 joints, parent-first depth at most seven,
   one inverse-bind accessor, four source influences, and three named linear clips.
2. Include joints and weights in geometry deduplication, remap them to canonical parent-first
   joint indices, and quantize weights deterministically to four bytes summing to 255.
3. Uniformly sample each source clip at 64 loop phases. Conjugate final skin palettes into
   the accepted normalized object space, derive parent-relative matrices, pad each clip to
   128 bones, and encode a strict canonical imported-rig payload.
4. Append the imported fixed bank after the unchanged fixture bank. Select the bank from
   authored archetype in cull/pose work and include the bank in shared pose identity.
5. Compare GPU palette samples, skinned surface samples, counters, occlusion bounds, and
   captures against CPU evaluation at controlled ticks 0 and 16.
6. Re-run physical reorder, time controls, hold/failure, traversal, rollover, resource
   plateau, lifecycle, direct-workflow, and repository guard gates.

## Controlled Variables

- Signed schema-3 object authority, exact identity, material 63, normalized geometry,
  grounding, LODs, composition, renderer clock, and fixed submission remain unchanged.
- Fixture rig bytes and fixture vertex bindings remain unchanged. The imported rig is a
  fixed second bank in the same four animation resources; no runtime glTF or fallback rig
  is permitted.
- Palette capacity and stride remain 128 bones per visible pose. The pose-key bitset and
  active-key buffer may grow only to the declared 1,024-key bound.
- The imported profile explicitly owns clip, phase, and variation; runtime selection may
  not derive them from stable identity or physical order.

## Metrics

- Source/cooked hashes, joint count/depth/order, clip names/durations/key counts/aliases,
  sample count, imported binding count, nonzero influence distribution, and weight sums.
- Fixture-bank hash, imported-bank hash, complete animation-catalog hash/GPU bytes, rig-bank
  offsets, pose-key capacity, active/reused poses, evaluated bones, and palette bytes.
- CPU/GPU palette and surface-sample deltas, exhaustive imported animated bounds, controlled
  tick capture hashes, changed pixels, and visible articulated silhouette checks.
- Dispatch/resource counts, traversal continuity, device state, handle/private-byte plateau,
  and process cleanup.

## Acceptance Criteria

- Any source hash, skin/joint/accessor, hierarchy, clip, influence, interpolation, or payload
  mismatch fails construction. Repeated cooks are byte-identical.
- Every imported cooked vertex has four in-range influences summing exactly to 255. Source
  joints map exactly once into a parent-first 24-joint order and maximum depth is at most
  seven.
- Fixture rig/binding bytes remain exact. Imported animation uses rig bank 1, Walk clip 1,
  all 64 phase offsets, and zero variation independent of physical record order.
- GPU and CPU palette/surface samples agree within the existing tolerances. Tick 0 and tick
  16 captures differ in geometry/attachments while retaining object identity, material,
  grounding, and publication state; manual inspection shows leg, spine, head, and tail
  articulation rather than rigid whole-object motion.
- The exhaustive 3-clip x 64-phase imported deformation remains inside a measured
  conservative bound. No false occlusion, invalid query, hierarchy mismatch, compaction
  mismatch, device removal, or lifecycle leak occurs.
- One cull, one compact, one indirect pose, one indirect mesh, and one surface path remain;
  all Experiment 0035 regressions and the full repository guard pass.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0036-cooked-gltf-skeletal-animation/`.

## Results

The direct workflow passed in 516.9 seconds. The verified source contained 24 parent-first
joints at maximum depth 7 and linear `Survey`, `Walk`, and `Run` clips with 83, 18, and 25
keys over 3.4166667, 0.7083333, and 1.1583333 seconds. The geometry cook retained 434
vertices and produced 434 exact bindings with nonzero-influence distribution
`[0, 202, 218, 12, 2]`; every binding summed to 255. The geometry payload, binding stream,
and imported rig hashes were respectively
`b0eb4940ee63a34e0b64569774ade165b767a458d5806b0239cf90dcf759c077`,
`de8831585bbb3a13504a049d106258c8819fb990e3908408239b03554baff319`, and
`fea223a83fc8d799c6ef794358f98aa5b524a8a0b7d92a80d9ca4c8fa0429ec1`.

The fixture and imported fixed-bank hashes were
`bf4eb3fddf98f18eb191f2d5ed3a4a5b4dcb9efe399f6375d843faf62fee80e8` and
`1ca9897100f0f1b5909dcc0cb892f827483b87f924dfcd325d516cd5cc645b71`.
The complete animation catalog occupied 6,326,464 GPU bytes. All 10,538 visible imported
objects were authored as Walk/rig 1 with zero variation. They compacted to 64 active poses,
reused 10,474 poses, evaluated 4,096 bones, and wrote 196,608 palette bytes while retaining
144,605 meshlets and 5,082,336 triangles. The sampled GPU palette differed from the CPU
result by only `2.3283064e-10`; surface channel delta was zero.

Tick 0 and tick 16 produced distinct color, PNG, and object-ID hashes:
`0a4945e7...`/`fc855276...`, `0daad87c...`/`f38b43a7...`, and
`ea3d222c...`/`2984cb32...`. Manual inspection confirmed intact Fox silhouettes in both
frames with changed leg, head/neck, body, and tail articulation rather than rigid whole-body
motion. No skin explosion or detached geometry was visible.

The exhaustive bound oracle tested 8,042,496 vertex poses. Maximum radial extent was
1.107228 inside the 1.2 imported bound with 0.019259 minimum slack after variant margin;
vertical pads were 0.137071 below and 0.089549 above, both within 0.25. Imported proof uses
32/64/128-bone evaluations because the source rig requires 24 joints; the retained fixture
proof still includes 16 bones, and the live runtime remains fixed at 64.

All time, reorder, hold/failure, restart, rollover, 32 reactive crossings, and 32 prepared
crossings passed. The 64-publication resource baseline was 531 handles and 402,796,544
private bytes; peak handles stayed 531, and the final sample was 516 handles and 403,517,440
bytes. All 16 lifecycle cycles stopped cleanly without device removal.

## Conclusion

Accepted. The pinned Fox now uses its source skin and Walk deformation through the same fixed
GPU pose, meshlet, surface, and occlusion path as the fixture rigs. Rig identity is part of
shared pose identity, and the imported profile explicitly owns clip, phase, and variation.
The runtime still has no glTF parsing, fallback rig, or second renderer.

## Promotion

Promoted verified joint/weight geometry cooking, strict imported-rig cooking, dual fixed rig
banks, rig-aware bounded pose compaction, and imported normalized-space skinning. ADR 0039
records the durable boundary. Clip blending/transitions, root-motion policy, source-duration
playback, animation compression/streaming, general multi-rig assets, and Wulin content remain
later gates.
