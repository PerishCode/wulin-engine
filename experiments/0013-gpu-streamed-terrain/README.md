# Experiment 0013: GPU Streamed Terrain

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0016](../../docs/adr/0016-gpu-streamed-terrain.md)

## Hypothesis

A globally addressed integer height lattice can be cooked, read, copied, published, and
expanded into terrain mesh patches with bounded storage and fixed CPU submission while
preserving exact geometry continuity across every active region boundary. Pending or
failed terrain transactions can leave the previous immutable frame snapshot unchanged.

## Scope

The experiment owns an independently versioned terrain pack whose fixed 4 KiB region
payload contains a 33x33 signed 16-bit height field and a 32x32 byte material field for
one 16-meter, 32x32-cell region. Heights are generated and quantized from global sample
coordinates, so adjacent payloads duplicate one byte-identical shared edge. The pack is
offline-written, indexed, aligned, checksummed, read on one bounded background worker,
and copied through a dedicated queue into a protected 50-slot default-heap cache.

The workbench renders the published radius-0, radius-1, or radius-2 snapshot through one
fixed 400-group amplification/mesh submission. Each active region expands into sixteen
8x8-cell mesh patches; inactive groups emit no mesh work. A GPU oracle compares every
sample of every active horizontal and vertical region edge using the exact shader decode
path. Terrain color and semantic region IDs participate in deterministic frame capture,
bounded-region perception, and probe readback.

Terrain LOD, crack stitching between different resolutions, clipmaps, virtual texturing,
material blending, authored terrain, editing, collision, navigation, object grounding,
object/terrain composition, terrain occlusion, and broad GPU compatibility are excluded.
The existing instance `region-format` V1 remains unchanged; this experiment must not
reinterpret its records or promote a generic asset container without evidence.

## Workload

1. Cook deterministic terrain packs for the same sparse centers `[64,64]`, `[65,64]`,
   `[65,65]`, and `[96,96]` used by accepted cooked-region streaming.
2. Round-trip every payload and reject malformed headers, indices, padding, checksums,
   noncanonical coordinates, and mismatched neighboring edges.
3. Open the pack, publish radius 2 at `[64,64]`, enable terrain rendering, and validate
   the 25 active mappings, 400 fixed patch groups, generated geometry aggregates, all
   GPU edge comparisons, semantic joins, samples, and attachments against a CPU oracle.
4. Sweep active radii `0`, `1`, and `2` at fixed cameras centered on a region interior,
   an X boundary, a Z boundary, and a four-region corner.
5. Move `[64,64] -> [65,64] -> [65,65]`, revisit `[64,64]`, and teleport to `[96,96]`.
   Record retained, uploaded, evicted, protected, resident, and free slots plus exact
   active ordering and payload hashes.
6. Hold background I/O and copy completion independently. Continue rendering the exact
   old snapshot, then release and publish only the complete new mapping at a frame
   boundary.
7. Corrupt one requested payload before copy submission. Reject the transaction, roll
   back its reservation, keep the previous snapshot and attachments unchanged, restore
   the pack, and complete a valid retry.
8. Restart through Sidecar and reproduce the canonical mapping, edge proof, semantic
   evidence, and attachment hashes.
9. Measure optimized cached-revisit and cold-teleport I/O, verification, copy, publish,
   terrain amplification/mesh/raster, and total frame distributions with preheat,
   per-workload warm-up, and 32 samples.

## Controlled variables

- World side is 128 regions; each region is 16 meters and contains 32x32 half-meter
  cells, 33x33 height samples, and 32x32 material cells.
- Height values are canonical signed 16-bit integers at 1/256 meter. Payload byte order,
  layout, zero padding, index ordering, alignment, and SHA-256 coverage are fixed.
- Cache capacity is 50 slots and active capacity is 25 regions. Reservation cannot
  overwrite a slot referenced by the current frame snapshot or an in-flight copy.
- Active region order is row-major by logical region coordinate and does not depend on
  physical cache slots, I/O completion order, or prior movement.
- Rendering always submits one fixed 400-group amplification/mesh dispatch. Each active
  region owns sixteen fixed 8x8-cell patches with 81 vertices and 128 triangles; groups
  outside the active mapping emit zero mesh work.
- The GPU seam oracle compares 33 samples for each active horizontal or vertical shared
  edge. It is dispatched at a fixed maximum shape and ignores absent neighbors.
- Reference adapter, 1280x720 extent, reverse-Z projection, cameras, generated pack,
  material palette, background, presentation mode, and semantic-ID assignment are fixed.
- Correctness uses the debug-layer Sidecar manifest. Timing uses the release benchmark
  manifest with validation explicitly disabled.

## Metrics

- Pack version, index and payload sizes, file and per-region hashes, read and verify
  bytes/times, queue depth, failure stage, and reservation rollback state.
- Cache capacity, committed/allocation bytes, active/retained/uploaded/evicted/protected/
  resident/free slots, upload bytes, copy fence, publication generation, and high-water
  marks.
- Active mapping hash, payload hash, CPU neighboring-edge comparisons, GPU neighboring-
  edge comparisons, mismatch count, first mismatch, and decoded height extrema.
- Fixed amplification groups, emitted patches, vertices, triangles, inactive groups,
  overflow, and exact GPU/CPU aggregate agreement.
- Terrain GPU time and total frame P50/P95/P99, CPU schedule and publication times, I/O
  and checksum times, copy time, and cached-revisit versus cold-teleport distributions.
- Raw color and object-ID hashes, diagnostic edge image hash, semantic region counts,
  bounded-region joins, registered samples, process identities, debug messages, and
  final Sidecar namespace cleanup.

## Acceptance criteria

- The terrain pack round-trips byte-identically, every generated neighboring edge is
  equal before writing, and malformed metadata, payload, padding, checksum, or edge data
  is rejected by the owning validation stage.
- Every published active mapping is row-major, contains exactly `(2r+1)^2` unique logical
  regions, references initialized immutable slots, and matches the CPU snapshot oracle.
- CPU and GPU compare every shared-edge sample in radius 1 and radius 2 snapshots with
  zero mismatch. Registered boundary and corner cameras show no background pixel between
  adjacent terrain regions and reproduce deterministic color, object-ID, diagnostic,
  semantic, and sample evidence.
- GPU emitted patch, vertex, and triangle aggregates exactly match the CPU oracle with
  zero inactive emission and overflow. CPU records exactly one fixed 400-group terrain
  mesh submission and one fixed-shape edge-oracle dispatch regardless of radius, cache
  history, world position, or visible pixel count.
- I/O and copy holds render byte-identical old attachments until complete publication.
  Corruption fails before copy submission, cancels the complete reservation, preserves
  the old mapping and attachments, and permits a valid retry without restart.
- Storage, descriptors, channels, upload arenas, readback, and in-flight transactions
  remain explicitly bounded. CPU does not read GPU counts before terrain submission.
- Movement, revisit, teleport, corruption recovery, and Sidecar restart reproduce exact
  mappings, hashes, oracle results, and expected cache movement without validation error,
  device removal, hidden fallback, unbounded growth, or residual process.
- Experiments 0007-0012 and `runseal :guard` pass after the final implementation.

## Environment

The final report records revision and dirty state, Windows build, adapter and driver,
feature level, shader model, Agility SDK, DXC, Rust toolchain, debug-layer state, pack and
payload hashes, resource allocation sizes, cameras, movement evidence, failure evidence,
timing distributions, process identities, and cleanup.

## Reproduction

```powershell
runseal :terrain
```

The command recooks the canonical pack, runs debug-layer correctness and release timing
through isolated Sidecar namespaces, injects I/O, copy, and checksum failures, verifies
restart determinism, cleans both namespaces, and writes the ignored structured report to
`out/captures/0013-gpu-streamed-terrain/acceptance.json`.

## Results

- The canonical 60-region pack is 249,856 bytes with 245,760 payload bytes and SHA-256
  `4a7ad08da2eb9a2adf2c90a9b0402886196dd70330098f60ccd43be9d5110571`.
  Pre-write validation compared 3,234 samples over 98 neighboring edges with zero
  mismatch. Pack round-trip and malformed header, checksum, padding, and edge tests pass.
- Radius 0, 1, and 2 emitted 16, 144, and 400 patches from the same single 400-group
  mesh dispatch. Radius 2 emitted 32,400 vertices and 51,200 triangles. Every aggregate
  exactly matched the CPU oracle; inactive groups emitted no geometry.
- Radius 1 compared 396 samples over 12 shared edges and radius 2 compared 1,320 samples
  over 40 shared edges. CPU decode and the fixed `[25,2,1]` GPU seam dispatch reported
  the same edge counts and zero mismatch at interior, X-edge, Z-edge, and corner cameras.
- The canonical mapping and payload hashes are
  `42adea7d457e6094829661910fb22122b8069ff56570f22d94129970df47c449`
  and `5353840d77c05d7e7e0e17232e06a5cc2bc2461b86b25ba32c3f2e9c5774c460`.
  Color, object-ID, and diagnostic attachment hashes reproduced after Sidecar restart,
  and the center semantic joined to `terrain.region.064.064` with no unknown IDs.
- Adjacent movement retained 20 regions and uploaded 5; the cached revisit retained all
  25 and uploaded zero; the teleport uploaded 25 and evicted 10 at the fixed 50-slot
  capacity. Active ordering remained row-major independently of physical slots.
- Independent I/O and copy holds continued presenting byte-identical old attachments.
  A corrupted final requested payload failed after 98,304 read bytes and before copy,
  left no reservation or pending transfer, preserved the old snapshot, and completed a
  valid 25-region retry after restoration.
- Logical payload storage is 204,800 bytes, copy timing readback is 16 bytes, and 50
  committed 4 KiB buffers consume 3,276,800 allocation bytes on the reference device.
  This is bounded and accepted for the architecture experiment, but it is not a
  promoted final terrain allocation policy.
- Thirty-two cached revisits each retained 25 regions, read and uploaded zero payload
  bytes, and reproduced the empty upload hash. Thirty-two independently restarted cold
  teleports each read, verified, and copied all 25 regions and 102,400 bytes with the
  same mapping and payload evidence. Cold I/O total measured 0.2381 ms median and 0.3511
  ms P95; copy-queue GPU time measured 0.005856 ms median and 0.006368/0.006656 ms
  P95/P99; copy submission to frame publication measured 0.0514 ms median and
  0.0692/0.0788 ms P95/P99.
- In the optimized 32-sample canonical workload after 250 ms warm-up, seam validation
  measured 0.00512 ms median and 0.007168/0.008192 ms P95/P99; terrain raster measured
  0.049152 ms median and 0.050176 ms P95/P99; combined GPU time measured 0.054272 ms
  median and 0.057344/0.058368 ms P95/P99. These values characterize this fixed
  laboratory workload only.
- Experiments 0007-0012 passed after implementation. `runseal :guard` passed workspace
  clippy, release tests, frozen Deno checks, Flavor with zero deny issues, and both
  Sidecar profile checks. No validation error, device removal, fallback, or residual
  process was observed.

## Conclusion

Accepted. A globally addressed fixed-resolution integer lattice can move from an offline
pack through bounded asynchronous I/O and copy publication into one fixed GPU terrain
submission while preserving exact shared edges, semantic perception, and the previous
immutable snapshot under delay or failure. CPU submission shape is independent of
radius, cache history, world position, emitted geometry, and visible pixels.

This result does not accept cross-resolution continuity or a production terrain system.
The next terrain experiment must introduce a falsifiable LOD transition and crack-free
stitching contract rather than weakening the exact same-resolution edge invariant.

## Promotion

- Promote `crates/terrain-format` as the independently versioned fixed terrain pack,
  payload codec, validation, and indexed-reader boundary.
- Promote `tools/terrain-cooker` as the offline-only deterministic writer and global
  lattice fixture generator for this accepted format.
- Keep terrain streaming, cache resources, GPU mesh expansion, seam probes, and Sidecar
  controls workbench-owned until a later experiment establishes a broader engine API.
- Do not promote committed-per-region allocation, fixed 400-group overdispatch, terrain
  LOD, crack stitching, material blending, collision, grounding, composition, or
  occlusion from this result.
