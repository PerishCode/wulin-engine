# Experiment 0019: Camera-Relative Global Space

Status: Accepted

- Related ADRs: [ADR 0022](../../docs/adr/0022-camera-relative-global-space.md)

## Hypothesis

Signed 64-bit global region coordinates can remain entirely CPU-owned while the GPU
receives only bounded origin-relative `f32` positions: relocating the same calibration
scene by at least plus or minus 2^40 regions and rebasing its render origin by nearby
integer regions preserves byte-identical color/object-ID attachments, exact local
transform evidence, and bounded conversion work without changing current region or
terrain formats.

## Scope

This experiment introduces a split global-space vocabulary above existing local
formats. A scene anchor and render origin are signed 64-bit XZ region coordinates. A
position consists of that integer region identity plus a bounded local `f32` offset.
Render conversion subtracts region coordinates as integers before multiplying the small
delta by the 16 meter region side and adding the local offset.

The existing camera control remains scene-local for compatibility. View and calibration
object model matrices must consume the converted render camera and object positions,
making the split conversion load-bearing rather than metadata-only. World relocation
and rebase controls are calibration-mode-only.

This experiment does not change format V1, cache keys, GPU stable keys, semantic IDs,
terrain generation, composition traversal, floating-origin policy, network coordinates,
physics, animation roots, or authored-world partitioning. It does not yet stream unique
content at global addresses.

## Workload

1. Reproduce Experiment 0003's default and alternate camera captures at anchor/origin
   `(0,0)` and preserve its accepted scene-local camera and object vocabulary.
2. Relocate the complete scene to anchors `(2^20,-2^20)`, `(2^40,-2^40)`, and
   `(-2^40,2^40)`. Keep render origin equal to anchor and require byte-identical local
   matrices, color pixels, object IDs, and PNG artifacts.
3. At a far anchor, hold all global positions fixed and rebase render origin by one and
   four regions on both axes. Require byte-identical attachments despite changed
   render-space camera and object translations, and bound clip-space error against the
   canonical scene-local oracle.
4. Reset origin to the scene anchor and require exact logical revisit evidence and
   attachments. Relocate back to zero and require the canonical baseline again.
5. Run a requested 25,600-position, 5 by 5 region oracle. Validate split normalization,
   integer region subtraction, bounded render values, exact Q9 fixture reconstruction,
   render hash stability across far anchors, and global hash variation.
6. Exercise exact positive/negative half-open region boundaries and reject an origin
   farther than the registered render-delta bound without mutating accepted state.
7. Restart the process and reproduce zero-anchor and far-anchor evidence.
8. Collect 64 requested release probes across controlled anchors/origins and report CPU
   conversion distributions separately from capture/readback latency.

## Controlled Variables

- Calibration geometry, camera poses, reverse-Z, object IDs, shaders, attachments,
  1280x720 extent, and presentation remain unchanged.
- Global region coordinates are signed 64-bit integers. Region side remains exactly 16
  meters. Integer region subtraction occurs before any `f32` conversion.
- Local XZ values are normalized to half-open `[-8,8)` region intervals. Exact positive
  boundaries move to the positive region; exact negative boundaries remain in the
  current half-open owner according to Euclidean normalization.
- The maximum render-time region delta is explicit and small. Out-of-range conversion
  is an error, not a large-float fallback.
- Relocation translates scene and origin together. Rebase changes only render origin;
  global camera and object positions remain fixed.
- Correctness uses the debug Sidecar namespace. Timings use the release namespace with
  validation disabled. Probes are requested-only CPU evidence and add no normal-frame
  GPU work.

## Metrics

- Scene anchor, render origin, relocation/rebase counts, signed range, region side, and
  maximum permitted/rendered region delta.
- Scene-local, split global, render-relative, and camera-relative positions; matrix
  hashes; global/render-position hashes; canonical/render clip-space hashes; and maximum
  absolute clip-space error.
- 25,600 sample count, Q9 denominator/range, normalization and reconstruction mismatch
  counts, non-finite count, and conversion allocation bytes.
- Color, PNG, and object-ID hashes, semantic ranges, unknown IDs, process identities,
  validation state, and device-removal state.
- Requested CPU conversion and capture/readback median/P95/P99 distributions.

## Pass Criteria

- Anchors at zero, plus/minus 2^20, and plus/minus 2^40 regions serialize exactly and
  produce distinct global hashes without any global-coordinate-to-`f32` conversion.
- Equal scene-local state with origin equal to anchor produces byte-identical render
  camera/object matrices, color pixels, object IDs, and PNGs at every anchor.
- Nearby render-origin rebases preserve byte-identical color/object-ID attachments and
  canonical clip-space evidence while global positions remain unchanged. Maximum
  absolute clip-space error remains at or below `0.0001`. Returning to the anchor
  reproduces exact local hashes.
- All 25,600 oracle positions normalize and reconstruct exactly on the registered Q9
  lattice, remain finite and inside the render bound, and allocate no per-sample runtime
  state outside the requested probe.
- Half-open boundary ownership matches the registered convention. An out-of-range
  render origin is rejected and leaves anchor, origin, counters, and attachments
  unchanged. No large-float fallback exists.
- Default world state preserves Experiment 0003 and affected camera-driven composition
  workflows. World controls reject non-calibration modes.
- Normal frame submission adds no GPU pass, resource, descriptor, or request-sized
  allocation. Release distributions are reported without a speedup claim.
- Debug/release validation, restart, Flavor, and Sidecar lifecycle pass with no device
  loss, hidden fallback, unbounded growth, or residual process.

## Evidence

The planned canonical workflow is:

```powershell
runseal :global-space
```

Generated evidence remains ignored under
`out/captures/0019-camera-relative-global-space/`.

## Results

The canonical workflow passed on 2026-07-13. It recursively passed Experiments 0003 and
0018; the latter passed its complete 0017 -> 0016 -> 0015 compatibility chain.

- Zero, `(2^20,-2^20)`, `(2^40,-2^40)`, and `(-2^40,2^40)` produced four distinct
  global-position hashes while preserving one render-position hash, camera-relative
  matrix hash, canonical clip-space hash, color hash, PNG hash, object-ID hash, and
  diagnostic hash.
- The requested 25,600-position Q9 oracle reported zero normalization,
  reconstruction, boundary, and non-finite mismatches with zero per-sample allocation.
- Rebases at offsets `(1,1)`, `(-1,-1)`, `(4,-4)`, and `(-4,4)` produced distinct
  render-position hashes while preserving global positions and all attachments. Maximum
  absolute clip-space error was `0.000003814697265625`, below the `0.0001` gate.
- An origin nine regions beyond an object was rejected as `invalid_world_space`; state
  and attachments did not change. A relocation while procedural load mode was active
  was rejected as `world_mode_required`.
- Restart changed the process identity and reproduced far-anchor logical and attachment
  evidence. Both debug and release namespaces stopped without a residual process.
- Across 64 requested release probes, conversion time was 1.2248 ms median, 1.2701 ms
  P95, and 1.4306 ms P99. Sidecar probe round trip was 29.7701/37.0831/37.5406 ms.
- Across 16 release captures, capture round trip was 133.3440/141.5675/141.5675 ms and
  GPU submission/readback was 3.3533/6.3213/6.3213 ms median/P95/P99.

These are laboratory control-path distributions, not normal-frame costs or a speedup
claim. The normal frame path adds no probe work.

## Conclusion

The hypothesis passes. Signed global regions can remain exact CPU state at distances far
beyond direct `f32` representation while the calibration GPU path consumes only bounded
camera-relative values. Integer region subtraction occurs before conversion, and a
camera-at-origin model/view decomposition removes render-origin translation before GPU
submission. Material semantic coordinates remain scene-local, separate from raster
coordinates, so rebase does not move procedural shading.

The experiment accepts the coordinate representation and calibration conversion
boundary. It does not accept a global streaming address space, format migration,
automatic render-origin policy, or general floating-origin system.

## Promotion

Promote `RegionCoord`, split positions, bounded render conversion, transactional world
controls, and camera-relative calibration transforms as the coordinate boundary for the
next signed-address streaming experiment. Preserve existing unsigned format/cache/GPU
keys until a separate experiment migrates them end to end.
