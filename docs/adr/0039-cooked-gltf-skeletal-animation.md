# ADR 0039: Cooked glTF Skeletal Animation

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

Experiments 0034 and 0035 made the pinned Fox visible with source geometry and material, but
its vertices were rigidly bound to the fixture root. The renderer already had fixed GPU pose
reuse, hierarchy evaluation, four-influence meshlet skinning, surface reconstruction, and
conservative occlusion. The missing boundary was an independently authored rig whose
hierarchy differs from the procedural 128-bone fixture rig.

Experiment 0036 proves whether that source rig can enter the same execution path without
changing the signed object schema or creating an imported-object renderer.

## Decision

- Fox JOINTS_0 and WEIGHTS_0 participate in the canonical geometry vertex identity. The cook
  validates 24 parent-first joints at depth at most seven, maps indices into that order, and
  deterministically quantizes four weights to bytes summing to 255.
- `animation-catalog` verifies the pinned JSON/BIN, one skin, exact inverse binds, and three
  named linear clips at build time. It uniformly samples each clip at 64 loop phases and
  emits a strict `WLSKN001` payload; runtime code decodes only that payload.
- The animation resources contain two fixed banks. Rig 0 is the unchanged 128-bone/eight-clip
  fixture bank. Rig 1 is the 24-joint Fox rig padded to the same fixed 128-bone/eight-clip
  shape. Source clips occupy slots 0 through 2; slots 3 through 7 alias them as
  `[Survey, Walk, Run, Survey, Walk]`.
- Archetypes 0 through 6 select rig 0 and archetype 7 selects rig 1 in the same cull, pose,
  meshlet skin, surface, and occlusion path. Shared pose identity is
  `rig * 512 + clip * 64 + phase`, giving a fixed 1,024-key domain.
- The imported object profile explicitly authors archetype 7, material 63, Walk clip 1, a
  deterministic phase, and zero variation. Runtime rig selection follows authored archetype;
  clip/phase/variation are never derived from stable identity or physical order.
- Imported vertices are skinned in normalized source space before the existing instance-height
  Y scale. Imported normals use the corresponding inverse order. This preserves joint pivots
  across varied instance heights while retaining the accepted geometry and object schema.
- Imported conservative-bound proof evaluates only bone counts that contain the complete
  24-joint source rig (32/64/128). The historical fixture proof retains 16/32/64/128, and the
  live canonical renderer remains fixed at 64 bones.

## Consequences

- The accepted Fox workload now has articulated legs, spine, head, and tail with exact GPU/CPU
  palette and surface agreement. It no longer relies on rigid root motion.
- Pose compaction remains bounded and data-driven across both banks. Imported Walk crowds use
  64 shared poses for 10,538 visible objects rather than one palette per object.
- Animation catalog GPU residency grows to 6,326,464 bytes, but descriptors, resource count,
  palette stride, indirect submission count, and runtime publication behavior remain fixed.
- Runtime binaries still do not parse glTF, select fallback skeletons, or retain source paths.
- This decision does not define clip blending, state transitions, root motion, duration-aware
  playback, compression/streaming, or a general multi-asset rig format. Those require new
  experiments rather than compatibility branches here.

## Evidence

Experiment 0036 passed the direct 516.9-second workflow. All 10,538 visible imported objects
used rig 1/Walk with 64 active poses and a maximum GPU/CPU palette delta of
`2.3283064e-10`. Tick 0/16 color, PNG, and object-ID attachments all differed while spatial,
identity, material, grounding, publication, and fixed-dispatch evidence remained exact.

The exhaustive 8,042,496-vertex-pose proof measured 1.107228 maximum radial extent within the
1.2 bound and vertical pads below 0.25. Manual inspection found intact, recognizably articulated
Fox silhouettes. All traversal, rollback, resource plateau, and 16-cycle lifecycle gates passed.
Generated evidence is ignored under
`out/captures/0036-cooked-gltf-skeletal-animation/`.
