# Experiment 0014: GPU Terrain LOD Stitching

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0017](../../docs/adr/0017-gpu-terrain-lod-stitching.md)

## Hypothesis

A fixed-shape mesh-shader terrain submission can select heterogeneous patch resolution
entirely from GPU-visible scalar state, reduce emitted vertices and triangles, and
preserve exact geometric continuity by projecting every fine transition-edge vertex
onto the neighboring coarse edge. CPU submission, terrain payload, residency, and
semantic ownership can remain independent of the selected LOD distribution.

## Scope

The experiment consumes the accepted terrain pack and immutable radius-2 snapshot
without changing `terrain-format` V1, cooker output, cache capacity, copy publication,
or semantic IDs. Each of the 400 active 8x8-cell patches selects one of three levels:

| LOD | Sample step | Vertices | Triangles |
| --- | ---: | ---: | ---: |
| 0 | 1 cell | 81 | 128 |
| 1 | 2 cells | 25 | 32 |
| 2 | 4 cells | 9 | 8 |

Automatic selection uses the integer Chebyshev distance between a patch's global
address and the camera's global patch address. Registered near and middle radii produce
LOD 0, 1, and 2 bands. Adjacent patch distance differs by at most one, so ordered radii
must yield an adjacent LOD delta no greater than one without CPU lists or relaxation.
Forced levels exist only for controlled sweeps and baseline comparison.

At a heterogeneous edge, both patches use the coarser step as the authoritative edge
segmentation. Fine-only vertices evaluate the exact rational interpolation of the two
coarse source heights and interpolate their clip positions from the same transformed
endpoints. The coarse patch therefore owns the same geometric line even though it does
not emit the intermediate vertex. Corners remain original shared samples.

Terrain LOD residency, clipmaps, geomorph over time, temporal anti-aliasing, material or
normal continuity, anisotropic selection, horizon culling, occlusion, virtual texturing,
skirts, collision, navigation, grounding, object composition, authored error metrics,
and broad GPU compatibility are excluded.

## Workload

1. Disable LOD and reproduce the accepted Experiment 0013 canonical probe and color,
   object-ID, diagnostic, mapping, and payload hashes exactly.
2. Enable automatic LOD with near radius 2 and middle radius 6 over the canonical 20x20
   patch grid. Compare GPU LOD counts, emitted geometry, transition counts, adjusted
   vertices, and work reduction against an independent CPU oracle.
3. For all 760 horizontal and vertical neighboring patch edges, reconstruct both emitted
   edge polylines at nine finest-grid positions. Compare exact integer numerators and
   common denominators on CPU and GPU; require zero mismatch and maximum LOD delta one.
4. Force LOD 0, 1, and 2. Validate exact 400-patch aggregates, zero transitions, fixed
   submission, deterministic semantics, and monotonic vertex/triangle reduction. Forced
   LOD 0 must reproduce the disabled attachments byte-identically.
5. Sweep automatic bands `[0,2]`, `[2,6]`, and `[4,8]` at a fixed camera. Then move the
   camera across a patch interior, patch edge, region edge, and four-patch corner. Require
   exact CPU/GPU agreement while the LOD distribution and transition topology change.
6. Capture interior, X-transition, Z-transition, corner, and grazing views. Record raw
   attachments, semantic joins, bounded samples, and LOD diagnostics; no transition may
   expose background between adjacent active patches.
7. Move the streamed center `[64,64] -> [65,64] -> [65,65]`, revisit `[64,64]`, and
   teleport to `[96,96]` while LOD remains enabled. LOD classification must follow global
   addresses and camera state rather than physical cache slots or publication history.
8. Hold I/O and copy publication independently during an enabled LOD frame. The old
   terrain and LOD evidence must remain unchanged until complete publication.
9. Restart through Sidecar and reproduce the canonical automatic probe and attachments.
10. In the release namespace, warm each workload and collect 32 requested probes for
    disabled, automatic, forced LOD 1, and forced LOD 2. Report validation, raster, total,
    emitted geometry, transition, and work-reduction distributions separately.

## Controlled variables

- Terrain pack, 4 KiB payload, global sample lattice, 50 cache slots, 25 active regions,
  row-major mapping, copy protocol, 1280x720 extent, and semantic ID range remain those
  accepted by Experiment 0013 and ADR 0016.
- Active radius is 2, producing a 20x20 patch grid and 400 fixed amplification groups.
  CPU records one fixed mesh dispatch. Enabled LOD adds one fixed `[400,2,1]` stitch
  oracle dispatch; the accepted raw region-edge oracle remains `[25,2,1]`.
- The camera contributes one integer global patch address. CPU does not enumerate patch
  levels, transition edges, emitted vertices, or emitted triangles before submission.
- Automatic band radii are unsigned patch distances with `near < middle`. Forced LOD is
  either absent or one of 0, 1, and 2. Invalid settings are rejected before mutation.
- Edge interpolation uses signed height numerators over a shared power-of-two denominator.
  GPU mesh emission and both oracles use the same registered arithmetic, while CPU oracle
  implementation remains independent of shader output.
- Correctness uses the debug-layer Sidecar profile. Timing uses the release profile with
  validation disabled. Cameras, warm-up, sample count, and capture IDs are fixed.

## Metrics

- LOD 0/1/2 patch counts, maximum adjacent delta, same-level and transition edge counts,
  fine vertices projected, edge sample comparisons, mismatch count, and first mismatch.
- Fixed patch groups, emitted patches, vertices, triangles, inactive groups, baseline
  vertices/triangles, absolute reduction, and percentage reduction.
- CPU/GPU aggregate and exact edge-oracle agreement; mapping, payload, settings, camera
  patch, and LOD evidence hashes.
- Validation, mesh/raster, and total GPU P50/P95/P99 by forced and automatic workload.
- Color, object-ID, diagnostic, semantic, and sample evidence for disabled, transition,
  movement, held publication, and restarted states.
- Validation errors, device removal, resource bounds, process identities, and final
  Sidecar namespace cleanup.

## Acceptance criteria

- Disabled and forced LOD 0 reproduce Experiment 0013 canonical geometry, submission,
  mapping, payload, and all three attachment hashes exactly.
- Every active patch receives exactly one valid GPU-selected level. GPU level counts and
  emitted patch, vertex, and triangle aggregates exactly match the CPU oracle for every
  forced, automatic, camera, movement, teleport, held, and restarted workload.
- Every neighboring patch pair has LOD delta at most one. CPU and GPU reconstruct all
  6,840 edge positions for the canonical grid with the same transition count, adjusted
  vertex count, and zero geometric mismatch. No visual transition capture exposes a
  background crack or unknown semantic ID.
- Automatic LOD emits fewer vertices and triangles than the full-resolution baseline;
  forced levels are strictly monotonic. CPU submission remains one fixed 400-group mesh
  dispatch and fixed-shape validation dispatches independent of distribution and output.
- CPU does not read GPU LOD or geometry counts before submission. Readback occurs only
  for requested probes and remains bounded.
- Stream movement and physical slot changes do not alter classification for identical
  global patch and camera addresses. I/O/copy holds preserve the exact old LOD snapshot;
  restart reproduces canonical evidence.
- Experiment 0013, affected shared-path regressions, and `runseal :guard` pass after the
  final implementation without validation error, device removal, hidden fallback,
  unbounded growth, or residual process.

## Environment

The final report records repository revision and dirty state, Windows build, adapter and
driver, shader model, Agility SDK, DXC, Rust toolchain, debug-layer state, terrain pack
hash, settings, cameras, mapping and payload hashes, resource sizes, exact edge evidence,
geometry reductions, timing distributions, process identities, and cleanup.

## Reproduction

```powershell
runseal :terrain-lod
```

The command recooks the canonical terrain pack, exercises debug-layer correctness and
release timing through isolated Sidecar namespaces, cleans both namespaces, and writes
the ignored report to
`out/captures/0014-gpu-terrain-lod-stitching/acceptance.json`.

## Results

- Disabled and forced LOD 0 reproduced Experiment 0013 exactly: 400 patches, 32,400
  vertices, 51,200 triangles, canonical mapping hash `42adea7d457e6094829661910fb22122b8069ff56570f22d94129970df47c449`,
  payload hash `5353840d77c05d7e7e0e17232e06a5cc2bc2461b86b25ba32c3f2e9c5774c460`,
  and the same color, object-ID, and diagnostic attachment hashes.
- Canonical automatic `[2,6]` selected 25/144/231 patches at LOD 0/1/2. GPU and CPU
  both reported 7,704 vertices and 9,656 triangles, reductions of 76.222 and 81.141
  percent from the full-resolution baseline.
- The automatic workload contained 701 same-level and 59 transition edges. It projected
  158 fine vertices. CPU and GPU compared all 760 neighboring patch edges at 6,840
  positions with maximum adjacent delta one and zero mismatch.
- Forced LOD 0/1/2 emitted 32,400/10,000/3,600 vertices and
  51,200/12,800/3,200 triangles. All 400 patches received the forced level, all edges
  were same-level, and geometry decreased strictly.
- Band sweeps `[0,2]`, `[2,6]`, and `[4,8]`, patch and region boundary cameras, a
  four-patch corner, and a grazing camera all matched the independent oracle. Raw
  transition captures showed continuous terrain and no unknown semantic IDs.
- Independent I/O and copy holds preserved the exact old LOD snapshot. Adjacent moves,
  revisit, teleport, changed physical slots, and Sidecar restart preserved classification
  by global patch and camera address; restart reproduced automatic attachments exactly.
- In optimized 32-sample workloads after 250 ms warm-up, total GPU median/P95/P99 was
  0.05632/0.141312/0.167936 ms disabled,
  0.12288/0.161792/0.222208 ms automatic,
  0.234496/0.410624/0.412672 ms forced LOD 1, and
  0.247808/0.390144/0.401408 ms forced LOD 2. These timings include fixed validation
  and requested instrumentation and do not track emitted geometry monotonically.
- The real Sidecar control path rejected equal radii, an out-of-bound middle radius,
  and forced LOD 3 before settings mutation. Experiment 0013, affected shared-path
  regressions, and `runseal :guard` passed without validation error, device removal,
  fallback, unbounded growth, or residual Sidecar process.

## Conclusion

Accepted. Three GPU-selected patch resolutions can reduce emitted terrain work while
preserving exact cross-resolution geometry, immutable stream publication, semantic
ownership, and a CPU submission shape independent of the selected distribution.

This result accepts work elimination and crack-free geometric stitching, not a frame-time
improvement or a production terrain policy. The measured micro-workload is dominated by
fixed dispatch, validation, instrumentation, and pixels. Material/normal continuity,
temporal morphing, authored error, collision, composition, and broader terrain rendering
remain separate experiments.

## Promotion

- Accept the three-level integer distance classification and exact coarse-edge clip-space
  projection as the current workbench terrain LOD contract.
- Keep LOD selection, validation resources, mesh execution, and controls workbench-owned.
  Do not promote a general terrain engine boundary from this experiment.
- Preserve `terrain-format` V1, the cooker, streaming cache, publication protocol, and
  semantic range unchanged.
- Do not promote timing improvement, fixed 400-group overdispatch, experiment counters,
  distance bands, material/normal behavior, geomorphing, collision, or composition.
