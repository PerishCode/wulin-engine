# Experiment 0044: Exact Canonical Terrain Query

Status: Accepted

## Hypothesis

The canonical runtime can expose an exact, read-only CPU terrain-height query over the currently
published terrain snapshot, using signed global-region identity and half-open fixed-point local
coordinates, without source I/O, GPU access, render-LOD dependence, allocation, or any change to
the accepted publication, frame, failure, resource, and lifecycle behavior.

## Scope

This experiment adds one typed query position, one exact height/triangle result, and one `Runtime`
query method. A query addresses a signed `RegionCoord` and local X/Z Q9 coordinates in
`[-4096, 4096)`, representing the 16-meter region as a half-open domain. The result reports an
exact signed height numerator with denominator 65,536 and whether the point lies in the first
triangle, on the diagonal, or in the second triangle of its 0.5-meter terrain cell.

The implementation reads only the CPU tiles retained by the currently published canonical pair.
It exposes no cache slot, descriptor, render projection, camera, GPU resource, or source-streaming
handle. Normal, slope, material, bodies, contact state, simulation time, movement, gravity, jump,
camera behavior, actors, and gameplay are out of scope.

## Workload

1. Define the fixed-point position/result types and pure integer triangle sampler. Cover both
   triangles, the diagonal, negative heights, all half-open local edges, invalid coordinates,
   signed far regions, missing regions, mismatched tile identity, and arithmetic bounds.
2. Expose one read-only `Runtime` method delegated to the published terrain snapshot. Query before
   publication and outside the active window must fail explicitly; no fallback or clamping exists.
3. Compare a dense deterministic grid over every tile in the published 5x5 window with the
   existing exact CPU grounding oracle where their authored coordinates coincide. Record sample
   counts, triangle counts, exact hashes, and mismatch counts.
4. Hold each terrain/object I/O and copy gate and induce failed pair publication. Require every
   query to continue observing the old published snapshot and exact old hash until an atomic pair
   commit succeeds.
5. Repeat the grid through source reorder, revisit, alias, signed far rollover, 32 reactive and 32
   prepared traversal publications, and a fresh process. Require deterministic content results
   and exact identity changes only when global ownership changes.
6. Run the complete canonical GPU correctness, resource-plateau, prototype-host, and 16-cycle
   lifecycle gates.

## Controlled Variables

- Signed schema-3 object authority, terrain pack format, 50-slot terrain residency, triple-plane
  object residency, terrain-first atomic pair publication, and traversal policy remain unchanged.
- Terrain samples remain signed `i16` values in 1/256-meter units. Interpolation uses the existing
  cell diagonal and integer weights; no floating-point conversion occurs in the public query.
- Local Q9 coordinates are valid only in `[-4096, 4096)`. Positive region seams belong to the
  adjacent region at local `-4096`; invalid inputs are rejected rather than normalized or clamped.
- The query observes only the last committed terrain snapshot. Staged, in-flight, failed, or
  discarded terrain data is never visible.
- Each individual query performs a bounded scan of at most 25 active assignments and allocates
  no memory. It performs no file access, worker dispatch, GPU command, readback, fence wait, or
  synchronization operation.
- The existing render grounding sampler remains an independent experiment oracle during this
  proof; visual contact/LOD output does not participate in query authority.

## Metrics

- Focused test count and exact failure outcomes for invalid, unavailable, outside-window, and
  identity-mismatch queries.
- Dense-grid region/sample/triangle counts, query-result SHA-256, identity-keyed SHA-256, oracle
  mismatch count, and elapsed CPU time.
- Snapshot generation and exact query hashes before, during, and after held, failed, reordered,
  revisited, aliased, traversed, rolled-over, and restarted publications.
- Per-query heap allocation count, source-read count, GPU copy/readback count, fence-wait count,
  and synchronization count.
- Existing controlled GPU hashes, traversal results, resource plateau, and lifecycle evidence.

## Acceptance Criteria

- Public inputs and outputs are typed fixed-point values with explicit denominators. Both axes
  enforce the half-open local range exactly, and all interpolation arithmetic is integer and
  bounded without lossy global-coordinate conversion.
- Querying before publication, outside the published 5x5 window, or against an inconsistent
  assignment/tile snapshot fails explicitly. No previous-format, float, clamp, nearest-tile, I/O,
  GPU, or visual-LOD fallback path exists.
- The dense 76,800-sample grid contains exactly 25,600 first, diagonal, and second-triangle
  samples, has zero grounding-oracle mismatches, and reports zero per-query allocations, source
  reads, GPU copies/readbacks, fence waits, and synchronization operations.
- Held and failed pair operations preserve the previous query generation and hashes exactly.
  Successful pair publication changes terrain query visibility atomically with canonical render
  publication; reorder/revisit/restart reproduce exact content results.
- Reactive/prepared traversal and signed rollover preserve exact query correctness at every
  committed window, while global identity hashes follow the published region ownership.
- Focused tests, repository guard, all prior canonical GPU/prototype gates, resource plateau, and
  16 complete lifecycle cycles pass without validation or device-removal error.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0044-exact-canonical-terrain-query/`.

## Results

The 616-second direct workflow passed. Twenty-seven engine-runtime tests covered exact addressing,
both triangles, the diagonal, negative and extreme heights, half-open local bounds, adjacent-region
seams, far signed coordinates, outside-window lookup, and assignment/tile identity mismatch.
Pre-publication, local `-4097`, local `4096`, and center-plus-three-region queries all failed with
the exact `terrain_query_failed` contract; valid first/diagonal/second samples returned denominator
65,536 and numerators 130,176, 130,048, and 129,920. The adjacent-region `-4096` seam returned
94,720 without accepting the prior region's exclusive positive bound.

The controlled 5x5 probe executed 76,800 public-path queries in 117,490,200 ns. It observed exactly
25,600 first, diagonal, and second samples, height range `[-178624, 146048]`, and zero mismatches
against the independent arbitrary-Q8 grounding oracle. The result hash was
`5b26265ece4b58d650a6484e3e3688100197d5b753147783d9f1821a4fac13db`; the signed global
identity-keyed hash was
`58fb721098d409da3c50ab8534d2de063e251865753c54ecf446d34a4a339583`. Every query reported
zero allocation bytes, source reads, GPU copies/readbacks, fence waits, and synchronization.

Object source reorder, revisit, compensated alias, movement return, process restart, four held
I/O/copy gates, both corrupt-source rollbacks, prepared rollover, 32 reactive crossings, 32
prepared crossings, and all 16 lifecycle processes retained zero query-oracle mismatches. The
controlled color/PNG/object-ID/diagnostic hashes remained exact. The resource workload settled and
peaked at 527 handles with zero transient growth; publication 64 ended at 516 handles, 411,930,624
private bytes, and 18 threads.

## Conclusion

Accepted. The canonical runtime now exposes one exact read-only CPU height primitive over the last
committed terrain snapshot. Its signed identity, half-open Q9 domain, integer interpolation,
explicit failure behavior, and zero-I/O/GPU path are sufficient for the next spatial dependency;
it does not establish body, contact, normal, slope, material, time, or locomotion policy.

## Promotion

Promoted the typed position/height/triangle contract and pure integer sampler into
`crates/engine-runtime/src/terrain_query.rs`, added published-snapshot delegation through
`Runtime`, and added workbench-only single-query and dense oracle evidence. Retained CPU tile
storage under the canonical renderer and added no simulation or gameplay state.
