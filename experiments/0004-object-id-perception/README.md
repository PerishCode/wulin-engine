# Experiment 0004: Object-ID Perception

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0005](../../docs/adr/0005-capture-collection-contract.md),
  [ADR 0006](../../docs/adr/0006-spatial-and-depth-convention.md),
  [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md)

## Hypothesis

The workbench can produce a deterministic per-pixel semantic object-ID buffer in the
same draw and depth domain as the accepted color frame, analyze an explicitly bounded
screen region, and correlate visible pixels with the scene registry through Sidecar
without desktop capture or heuristic image recognition.

## Scope

The experiment adds one `R32_UINT` object-ID render target, a second pixel-shader output,
GPU-to-CPU ID readback, exact region validation, ID histograms and bounds, semantic joins,
a deterministic diagnostic PNG, and one typed `perception.capture` event.

It excludes world partitioning, streaming, GPU culling, occlusion queries, depth
readback, normal/material buffers, arbitrary selection shapes, continuous perception,
mouse UI, ECS, assets, animation, and generalized render-graph abstractions.

## Workload

The canonical workload starts the workbench through stable Sidecar, pauses continuous
rendering, validates the eight-object scene registry, and captures:

1. The complete 1280x720 default-camera viewport twice in one process.
2. The default camera with analysis constrained to `[560, 240, 160, 200]` in top-left
   pixel coordinates.
3. The complete viewport from the accepted alternate camera.
4. The complete default-camera viewport after Sidecar restart.

Every perception capture includes fixed sample points at `[0, 0]` and `[640, 360]`.
Artifacts overwrite the ignored
`out/captures/0004-object-id-perception/` collection.

## Controlled variables

- Experiment 0003 scene revision, object transforms, camera poses, shader lighting, and
  reverse-Z contract remain fixed.
- Pixel origin is top-left; `+X` points right and `+Y` points down in image space.
- Regions are integer half-open rectangles `[x, x + width) x [y, y + height)`.
- Full-frame region is `[0, 0, 1280, 720]`; the bounded region is
  `[560, 240, 160, 200]`.
- ID `0` means no semantic object. Drawn object IDs are the stable nonzero IDs declared
  by `calibration-v1`.
- Object IDs are stored as tightly packed little-endian `u32` values after row-pitch
  removal.
- Color and object ID are outputs of the same indexed draw calls and share the same
  reverse-Z depth test.

## Metrics

- Full ID-buffer byte count and SHA-256.
- Full-frame and requested-region ID histograms.
- Per-ID pixel bounds in full-frame top-left coordinates.
- Semantic ID, name, kind, and pixel count for every visible object in the region.
- Exact IDs at the fixed sample points.
- Diagnostic ID PNG byte count and SHA-256.
- Region coordinates, process identity, frame index, scene revision, camera state, and
  renderer/device status.
- GPU submission/readback, row copy, analysis, hashing, encoding, and artifact timing.
- Final Sidecar target and broker process counts.

## Acceptance criteria

- The graphics PSO has two render targets: `R8G8B8A8_UNORM` color and `R32_UINT` object
  ID. The ID target clears to zero every frame.
- Every scene draw writes its registry ID from draw constants; no color-derived or
  post-hoc classification is used.
- ID and color targets use the same draw submission, viewport, scissor, geometry, and
  reverse-Z depth buffer.
- The exact ID artifact contains 921,600 little-endian values and no ID outside zero or
  the eight-object registry.
- The requested half-open region is validated against the render-target extent and only
  its pixels contribute to the region histogram.
- `[0, 0]` resolves to background ID `0` and default-camera `[640, 360]` resolves to
  `block.occluder` ID `110`, proving a stable visible occlusion result.
- Region semantic entries exactly join IDs to the versioned scene registry.
- Repeated default-camera captures have identical raw ID and diagnostic PNG hashes and
  identical histograms.
- The alternate camera has a different raw ID hash from the default camera.
- The default camera after Sidecar restart reproduces the original raw ID hash,
  diagnostic PNG hash, histogram, and fixed samples.
- Every manifest reports no unknown IDs, device removal, or renderer error.
- Final Sidecar status contains no target or broker process and `runseal :guard` passes.

## Environment

The manifest and acceptance report record the actual revision, process, renderer,
camera, region, registry, hashes, timings, and semantic evidence.

## Reproduction

Run from the repository root with Sidecar 0.5.1 or newer installed from stable:

```powershell
runseal :object-id
```

## Results

Accepted results on 2026-07-12:

| Capture | Process | Frame | Region objects | Region background | Raw ID SHA-256 prefix | Diagnostic PNG prefix | GPU/readback |
| --- | ---: | ---: | ---: | ---: | --- | --- | ---: |
| Default 1 | 15532 | 13 | 8 | 513,161 | `b132c850f029` | `43526cdb1957` | 2.25 ms |
| Default 2 | 15532 | 14 | 8 | 513,161 | `b132c850f029` | `43526cdb1957` | 2.55 ms |
| Bounded | 15532 | 15 | 7 | 6,488 | `b132c850f029` | `43526cdb1957` | 2.13 ms |
| Alternate | 15532 | 16 | 8 | 452,135 | `76ae2fc8e0c7` | `ca34f709bd91` | 3.73 ms |
| Default restart | 23140 | 13 | 8 | 513,161 | `b132c850f029` | `43526cdb1957` | 2.68 ms |

Every capture contained 921,600 exact `u32` values and 3,686,400 tightly packed bytes.
No unknown ID appeared. The fixed background sample resolved to `0`; the center sample
resolved to `block.occluder` ID `110` in every default-camera capture. The bounded region
contained IDs `1`, `10`, `11`, `12`, `100`, `101`, and `110`, and its 32,000-pixel count
matched the declared half-open rectangle exactly.

Default raw ID bytes, diagnostic PNG, full histogram, semantic joins, and samples were
identical in one process and after restart. The alternate camera changed both raw and
diagnostic hashes. Manual inspection confirmed the diagnostic silhouettes and occlusion
relationships match the color frame. Final Sidecar status contained no process.

CPU analysis took approximately 124-146 ms and complete artifact preparation took
approximately 814-868 ms in the development build. These synchronous costs belong to an
explicit evidence operation and are not a runtime frame-performance gate.

## Conclusion

Accepted. The renderer now exposes an exact, deterministic semantic interpretation of
visible pixels and bounded screen regions. The result is produced by draw-time IDs and
shared depth, not color classification or desktop image inference.

## Promotion

ADR 0007 promotes the object-ID attachment, zero-background rule, pixel coordinates, and
typed perception capture to repository policy. Readback and CPU analysis remain
workbench evidence code. Future large-scene work must prove GPU-side reduction or query
behavior before promoting a runtime perception implementation.
