# ADR 0022: Camera-Relative Global Space

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0021 accepted camera-driven traversal inside the fixed 128 by 128 logical region
address space. The existing `u32` region ID simultaneously serves pack indexing, cache
identity, GPU stable keys, semantic identity, and procedural render placement. Replacing
all five contracts in one step would couple storage, streaming, perception, and rendering
without first proving that distant world coordinates can remain precise.

Directly converting a signed global position at plus or minus 2^40 regions to `f32`
would discard local movement. Experiment 0019 tests a narrower representation and
rendering boundary while leaving every accepted region format and cache contract intact.

## Decision

- CPU global XZ identity uses signed 64-bit region coordinates. A position combines that
  integer region with finite local meters normalized to the half-open interval `[-8,8)`.
- Region side is exactly 16 meters. Integer region coordinates are subtracted before the
  small difference is converted to `f32`, multiplied by 16, and combined with local
  meters. There is no large-float fallback.
- Calibration render conversion permits a maximum absolute region delta of eight. World
  relocation and render-origin rebase validate the camera, target, and all scene objects
  before publication. Failure leaves state and counters unchanged.
- Relocation moves scene anchor and render origin together. Rebase changes only render
  origin, keeping global positions fixed. Controls are mutable only in calibration mode.
- The calibration GPU path uses a camera-at-origin decomposition. Object and camera
  positions first pass through the bounded render-origin conversion; their common origin
  is then removed on the CPU, and the GPU receives camera-relative model translations and
  an orientation-only view matrix.
- Material semantic positions are separate from raster positions. Procedural calibration
  shading receives a stable scene-local offset, so origin rebases cannot move checker
  semantics even when render-space translations change.
- Existing scene-local camera controls and view/projection methods remain authoritative
  for terrain, composition, meshlet, skeletal, and resident modes. Their accepted
  unsigned region and format contracts do not inherit this experiment implicitly.
- Requested world probes generate exact CPU evidence only. They add no normal-frame GPU
  pass, resource, descriptor, or request-sized allocation.

## Consequences

Global region anchors at plus or minus 2^40 remain exact and produce distinct logical
hashes while equal local scenes render byte-identical color, PNG, object-ID, and
diagnostic artifacts. Nearby origin rebases change render-position hashes but preserve
attachments. The measured maximum absolute clip-space difference against the canonical
scene-local oracle is `0.000003814697265625`, below the accepted `0.0001` bound.

The CPU representation can now be consumed by a future signed-address streaming
experiment without forcing large world coordinates into GPU floats. The current
calibration path also establishes that camera-relative composition, rather than matrix
cancellation after GPU submission, owns rebase stability.

This decision does not change format V1, cooked pack indexes, cache slot keys, GPU stable
keys, semantic IDs, terrain generation, composition traversal, physics, animation roots,
network coordinates, or authored-world partitioning. It does not select when an engine
should rebase, prefetch distant regions, or migrate the fixed 128 by 128 address space.

## Evidence

- [Experiment 0019](../../experiments/0019-camera-relative-global-space/README.md)
  records far anchors, nearby rebases, 25,600 exact Q9 samples, transactional rejection,
  mode isolation, restart, compatibility, attachment hashes, and release distributions.
- Four anchors produced distinct global hashes and one shared render-position hash when
  origin equaled anchor. Four rebases produced distinct render hashes while all color and
  semantic attachments remained byte-identical.
- Experiments 0003 and 0018 passed; Experiment 0018 recursively passed its complete 0017
  through 0015 compatibility chain.

## Reproduction

```powershell
runseal :global-space
```

The command writes the ignored report to
`out/captures/0019-camera-relative-global-space/acceptance.json`.
