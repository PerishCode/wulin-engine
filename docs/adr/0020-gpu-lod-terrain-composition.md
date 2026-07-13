# ADR 0020: GPU LOD Terrain Composition

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0017 accepted camera-selected terrain patch LOD and exact transition projection in
the standalone terrain path. ADR 0019 accepted exact arbitrary-position object
grounding against full-resolution terrain in composition. Composition deliberately
disabled terrain LOD because neither decision established which surface owns object
height or measured the visible contact error introduced by coarse terrain triangles.

Changing object ground with camera-selected LOD would make physical positions depend on
the viewer. Keeping exact ground without measuring the visible approximation could hide
floating or penetrating contacts. Experiment 0017 tests the smaller, explicit split.

## Decision

- Full-resolution terrain format V1 heights remain the physical and grounding source of
  truth. Terrain LOD changes only visible terrain geometry. Exact Q16 ground values and
  the mesh-consumed ground buffer do not depend on camera patch, selected terrain LOD,
  or emitted terrain work.
- Composition may retain the terrain renderer's existing LOD settings. LOD-disabled
  composition records the accepted three terrain operations; LOD-enabled composition
  adds the accepted fixed 400 by 2 by 1 transition-validation dispatch. Skeletal
  submission remains five fixed operations.
- A requested-only CPU contact oracle evaluates the actual selected terrain triangle at
  every frozen fixture position. Transition edges use the coarser adjacent patch exactly
  as the mesh shader does. Selected surface and residual values use signed Q18 meters
  with denominator 262,144.
- The canonical fixture accepts a maximum absolute contact residual of 0.125 meter.
  This threshold is a deterministic test-asset gate, not a general terrain error policy.
  Every probe reports the full residual distribution and exceedance evidence.
- Existing terrain LOD configuration remains renderer-owned and camera-dynamic. It does
  not participate in atomic terrain/instance pair publication because it does not
  change either resident payload or exact object ground.
- Physical terrain cache slots may change after movement and teleport. Revisit compares
  logical payload, LOD, contact, grounding, and skeletal evidence while independently
  validating current mappings and terrain/instance slot divergence.

## Consequences

Terrain render work can now be reduced inside the composed scene without camera-driven
object motion or a new runtime format. The canonical automatic distribution emits
7,704 vertices and 9,656 triangles instead of 32,400 and 51,200 while preserving the
exact 25,600-value ground hash and all grounded skeletal aggregates.

Visible contact is explicitly an approximation at coarse levels. The canonical
automatic maximum is 0.0896 meter, below the registered 0.125 meter bound. A future
authored-terrain or camera policy experiment must register its own error metric and may
require geomorphing, local refinement, or another visual treatment; this decision does
not generalize the current threshold.

The contact oracle is requested-only and CPU-owned because it validates already
accepted terrain selection and exact GPU grounding. It does not add a diagnostic GPU
dispatch or alter normal frame submission.

This decision does not accept camera-dependent ground, geomorphing, authored terrain,
general screen-space error, sampling outside the owning region, slope frames, feet or
IK, collision, navigation, or a reusable scene query.

## Evidence

- [Experiment 0017](../../experiments/0017-gpu-lod-terrain-composition/README.md)
  records disabled, automatic, forced LOD0/1/2, both pass orders, contact distributions,
  exact grounding, cameras, movement, revisit, teleport, restart, compatibility, and
  release timing evidence.
- Automatic LOD reproduces patch counts `[25,144,231]`, 59 transition edges, maximum
  adjacent delta one, and zero geometric mismatch.
- All 25,600 exact ground values preserve SHA-256
  `c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db`
  across every LOD and camera control.
- Disabled and forced LOD0 contact residuals are exactly zero. Automatic, forced LOD1,
  forced LOD2, movement, and teleport have zero 0.125 meter threshold exceedance.
- Terrain-first and object-first attachments are byte-identical within every controlled
  LOD workload and contain no unknown semantic ID.

## Reproduction

```powershell
runseal :lod-composition
```

The command writes the ignored report to
`out/captures/0017-gpu-lod-terrain-composition/acceptance.json`.
