# ADR 0037: Cooked glTF Geometry

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

The canonical runtime already proved GPU meshlets, LOD, skeletal evaluation, surface
resolve, conservative occlusion, streamed terrain, and authored object presentation, but
its eight mesh archetypes were generated fixtures. Even after presentation time began to
advance, the visible scene could still look like moving diagnostic solids rather than
formed objects.

Experiment 0034 tests the first external-content boundary without turning the renderer
into an asset importer: cook one redistributable glTF source into the existing catalog and
render it through the single accepted runtime path.

## Decision

- The Khronos Fox glTF sample at commit
  `5bad5aaa0bbb5d0f9cdc934e626f27d0df1e79b8` is the pinned source. Its JSON, binary, and
  texture SHA-256 values and CC0/CC BY 4.0 attribution are repository evidence.
- glTF and mesh simplification are build-time dependencies of `meshlet-catalog` only.
  The renderer cannot parse glTF, open source asset paths, or construct fallback geometry.
- The cooker accepts exactly one identity-transformed, non-morphed triangle primitive
  with positions and UV0. It verifies all source hashes, centers XZ, normalizes Y to
  `[0,1]`, deduplicates exact position/UV tuples, generates normals, and emits full,
  half, and quarter triangle LODs.
- The canonical `WLFOX001` payload contains source hashes, vertices, surface attributes,
  LOD indices, and measured simplification errors. Runtime catalog construction performs
  a strict decode and publishes it as archetype 7; archetypes 0 through 6 remain fixtures.
- Imported normals and UVs enter the existing parallel surface stream. Generated runtime
  materials remain authoritative; source texture and material import are deferred.
- Imported vertices use a rigid root-bone binding. Existing authored animation state may
  move the complete shape, but source skin, joints, and clips are not approximated as
  runtime authority.
- Conservative occlusion uses a validated 1.2-unit radial bound only for imported
  archetype 7. The seven established fixture bounds remain unchanged.
- Any new source shape, generalized scene support, runtime asset streaming, source
  materials, or source skeletal data requires a later experiment and format revision.

## Consequences

- The canonical scene can now show recognizable authored silhouettes while retaining one
  meshlet/skeletal/surface/occlusion submission path and presentation-driven selection.
- Build-time failure is intentional for a changed hash or unsupported glTF structure;
  silently accepting a different upstream source would break reproducibility.
- The source Fox proportions require a larger conservative radial extent than the narrow
  generated fixtures. Applying it only to archetype 7 preserves existing occlusion work
  elimination for all other content.
- This is a proven single-asset boundary, not a general asset pipeline. Texture appearance
  remains diagnostic and imported deformation is intentionally rigid.

## Evidence

Experiment 0034 passed the direct 478.4-second workflow. The source cooked to 434 vertices
and 576/288/144 triangles in 16/9/4 meshlets, with exact runtime source/cooked hash joins.
The all-imported frame observed only archetype bit 7 and matched every GPU/CPU, grounding,
contact, surface, occlusion, and fixed-dispatch oracle while producing a visibly
recognizable Fox crowd. Its conservative bound retained 0.019259 minimum radial slack.

The workflow also passed all temporal, held-pair, corruption, traversal, rollover,
64-publication resource, and 16-cycle lifecycle regressions. Generated evidence is ignored
under `out/captures/0034-cooked-gltf-geometry/`.
