# Experiment 0034: Cooked glTF Geometry

Status: Accepted

## Hypothesis

The canonical runtime can render a recognizable externally authored glTF mesh through
the existing GPU meshlet, LOD, skeletal, surface, occlusion, and composition path while
keeping glTF parsing entirely offline and preserving deterministic fixed submission.

## Scope

This experiment imports the pinned Khronos Fox sample as archetype 7. A build-time
offline cooker reads the glTF 2.0 source, normalizes its geometry into the engine's
one-unit object space, creates three reducing LODs, partitions them into the accepted
64-vertex/126-primitive meshlet bounds, and emits a canonical binary payload embedded in
the workbench build. Runtime code decodes only that payload.

Archetypes 0 through 6 remain the accepted deterministic fixtures. Fox surface normals
and UVs enter the parallel surface stream, but the current generated material texture
array remains authoritative. Imported vertices use a rigid root-bone binding so existing
authored animation state may move the object without deforming its shape.

glTF textures, materials, skins, joints, clips, morph targets, multiple scenes or
primitives, runtime asset I/O/streaming, hot reload, legacy game formats, and Wulin
content are out of scope.

## Workload

1. Verify the pinned source files, upstream commit, hashes, attribution, and license.
2. Import one triangle primitive with positions and UV0. Reject unsupported scene,
   primitive, accessor, topology, non-finite, empty, or out-of-range inputs.
3. Normalize the bind geometry to `[0,1]` height with centered XZ, generate exact normals,
   and create deterministic full, half, and quarter triangle LODs with measured error.
4. Encode/decode the canonical payload twice and require byte-identical output, exact
   source joins, strict ranges, and mesh-shader bounds.
5. Select imported archetype 7 through cooked presentation authority and validate the
   existing GPU/CPU skeletal, surface, occlusion, semantic attachment, and capture oracles.
6. Re-run physical reorder, presentation mutation, time, hold/failure, traversal,
   rollover, resource plateau, and lifecycle gates through the direct workflow.

## Controlled Variables

- Signed schema-3 object authority, terrain, projection, exact grounding, clock,
  visibility, surface resolve, reverse-Z, and fixed submission remain unchanged.
- The renderer cannot open glTF files, inspect source paths, generate substitute geometry,
  or select a second catalog/rendering path.
- The source is fixed to upstream commit
  `5bad5aaa0bbb5d0f9cdc934e626f27d0df1e79b8`; file hashes are part of the cook contract.
- Imported geometry is one archetype in the existing eight-archetype table. Presentation
  data remains the only per-object archetype selector.
- Source skin, animation, materials, and texture are retained only for provenance and are
  not silently approximated as imported runtime authority.

## Metrics

- Source and cooked hashes, source vertex/triangle counts, normalized bounds, per-LOD
  triangle counts/errors, and per-LOD meshlet/vertex bounds.
- Canonical decode/re-encode identity and malformed-input rejection counts.
- Catalog CPU/GPU byte counts and hashes, imported vertex/primitive counts, rigid skin
  binding count, surface stream counts, and descriptor shape.
- GPU/CPU visibility, LOD, meshlet, triangle, skin, surface, occlusion, object-ID, and
  capture evidence for imported presentation.
- Content copy/publication counts, frame dispatches, D3D12 validation, handle/private-byte
  plateau, and process descendants.

## Acceptance Criteria

- The exact pinned source deterministically cooks to one canonical payload; any hash or
  structural mismatch fails before runtime construction.
- All three LODs retain valid triangles and strict reduction. Every meshlet stays within
  64 vertices and 126 primitives, and all references remain in range.
- Runtime geometry for archetype 7 comes only from the decoded cooked payload. It has the
  expected normalized bounds and source/cooked hashes and no procedural fallback.
- An imported-archetype source preserves stable identity, position, grounding, contact,
  publication, and fixed dispatch counts while changing catalog-bound GPU work and full
  visual evidence. GPU and CPU oracles remain exact.
- Rigid animation preserves imported vertex distances and time-only changes cause no
  source, cache, copy, or publication work.
- All Experiment 0033 regressions, the warmed resource bound, and lifecycle cleanup pass
  without validation or device errors.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0034-cooked-gltf-geometry/`.

## Results

The direct workflow passed in 478.4 seconds. The pinned source's 1,728 non-indexed source
vertices deduplicated to 434 canonical position/UV vertices. The three LODs contained
576, 288, and 144 triangles in 16, 9, and 4 meshlets; measured simplification errors were
0, 0.008710765, and 0.016450377. Normalized bounds were
`[-0.15934314, 0, -0.9788812]` through `[0.15934314, 1, 0.9788812]`. Source and cooked
hashes joined exactly in the runtime probe, and the meshlet catalog hash was
`e24aaf210a746aa281232e3bf1e2c26222cf5134b22224305ecf92189937c736`.

The imported presentation frame observed archetype mask 128 only. Its 10,538 visible
instances produced 144,605 meshlets, 9,053,328 emitted vertices, and 5,082,336 triangles;
GPU and CPU counts were exact. Compared with the mixed baseline, meshlets changed from
58,098 to 144,605 and triangles from 2,784,720 to 5,082,336 while identity, position,
grounding, contact, and terrain evidence remained byte-identical. The color and PNG hashes
changed to `c34995c79da249f4caf6ab19c204dad84f7249ec4144578c08d9c547a8976a0b` and
`9d979e1d44cecb9e0ba3d166565655a8b50f798552d8aeb461d72b0b699367d2`.

Manual inspection of `presentation-imported.png` confirmed recognizable Fox heads, ears,
bodies, legs, and tails across the crowd rather than generated solid silhouettes. Surface
sample error, grounding mismatch, invalid payload/query counts, hierarchy mismatch, and
stable-compaction mismatch were all zero. The imported 1.2 radial occlusion bound retained
0.019259 minimum slack across every tested clip, phase, bone count, height, and vertex.

All Experiment 0033 time, hold/failure, source reorder, traversal, prefetch, and rollover
gates passed. The 64-publication resource baseline was 531 handles and 397,770,752 private
bytes; peak handles were 532 and the final sample fell to 517 handles and 396,562,432
private bytes. All 16 lifecycle cycles left no process descendant or device removal.

## Conclusion

Accepted. One externally authored glTF geometry source now crosses a verified offline cook
boundary into the canonical runtime and renders as formed objects through the existing GPU
path. Runtime glTF parsing, fallback geometry, and a second renderer path remain absent.

## Promotion

Promoted the pinned-source offline glTF geometry cooker, strict canonical payload decode,
parallel imported surface stream, rigid root binding, and imported-archetype conservative
bound. ADR 0037 records the durable boundary. General scene import, source textures and
materials, source skeletons/clips, runtime streaming, and Wulin content remain later gates.
