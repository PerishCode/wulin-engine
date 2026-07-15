# ADR 0048: Remove the Calibration Compatibility Surface

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The canonical runtime and plain prototype now own one accepted content, frame, input, bootstrap,
and terrain-query path. The engine runtime still carries an earlier calibration scene containing
eight static objects, split floating-point world relocation/rebase controls, a dedicated vertex and
pixel shader pipeline, and inspect verbs that are absent from the current operator workflow.

The canonical renderer still borrows its depth and semantic ID targets from that calibration
renderer. This shared resource ownership makes the old renderer appear live even though canonical
frames never submit its geometry. Experiment 0045 must distinguish those current attachments from
the obsolete behavior that happens to own them.

## Decision

- Delete the calibration scene, geometry, shaders, draw pipeline, object inventory, split-world
  state/probes, runtime facade methods, and `scene.list_objects`/`world.*` inspect verbs without
  compatibility aliases.
- Retain a clear-only diagnostic idle shell. It owns no scene or semantic object and exists only as
  the pre-publication workbench frame outcome.
- Move the shared reverse-Z depth and `R32_UINT` semantic attachments to a neutral frame-target
  owner used directly by idle clearing, canonical passes, and capture readback.
- Keep `SceneState` as the narrow current camera owner. Move canonical semantic lookup into the
  existing ID-mapping owner, and move signed `RegionCoord` into its own module while preserving
  its exact API.
- Retain the fixed eight-slot animation bank and imported three-source-clip cook because they form
  the current region/GPU presentation contract. Altering that layout requires a separate measured
  experiment, not a cleanup alias.
- Extend the active-source guard with the deleted symbols and paths. Settled ADR and experiment
  history remains outside that compatibility scan.

## Consequences

- Idle workbench frames become intentionally empty except for clear color and cleared shared
  attachments; early calibration visuals and spatial controls disappear permanently.
- Canonical rendering and semantic capture no longer depend on an obsolete renderer owner while
  retaining identical resource formats, states, handles, and hashes.
- Signed global identity remains current and reusable without carrying float split-world policy.
- Removing the calibration pipeline should reduce live shaders, root signatures, pipeline state,
  upload buffers, source size, and runtime resources; acceptance forbids resource growth but does
  not prescribe an implementation-specific reduction count.

## Evidence

Experiment 0045 passed the 645.2-second direct workflow. Eight retired files, six inspect verbs,
seven runtime methods, two split-world types, eight calibration objects, and two shader stages were
deleted without an alias. The 1280x720 idle capture contained 921,600 background semantic values,
zero visible/unknown semantics, and a uniform configured clear color; all retired verbs returned
`unknown_event`.

The controlled canonical color, PNG, object-ID, diagnostic, shadow, and terrain-query hashes were
unchanged. Reactive/prepared traversal each completed 32 crossings, the 527-handle plateau had zero
transient growth, publication 64 ended at 516 handles/408,723,456 private bytes/18 threads, and all
16 lifecycle processes cleaned up without validation or device-removal error.

Generated evidence remains ignored under
`out/captures/0045-active-compatibility-removal/`.
