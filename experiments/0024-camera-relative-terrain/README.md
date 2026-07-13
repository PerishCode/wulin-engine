# Experiment 0024: Camera-Relative Terrain Projection

Status: Accepted

- Related ADRs: [ADR 0027](../../docs/adr/0027-camera-relative-terrain-projection.md)

## Hypothesis

Canonical signed terrain can render and expose perception independently of its bounded
local alias without sending signed coordinates to the GPU. One frame-local projection
derived from ordered active rows/columns, plus a camera translated by the alias center's
exact integer-meter offset, is sufficient to make color, PNG, object-ID, diagnostic,
LOD, and terrain oracle evidence byte-identical for the same global window at every
legal local center.

## Scope

This experiment changes only V2 terrain projection. Format V1 continues to derive
positions, LOD patch coordinates, and object IDs from local `region_id` exactly as
accepted. V2 content identity remains `{source namespace, signed region}` and its active
assignments continue to expose local IDs for publication compatibility.

For V2, active row/column and radius define terrain positions relative to the global
window center. The CPU translates camera position and target by
`(local active center - 64) * 16` meters before constructing the existing reverse-Z
view-projection matrix. GPU patch/LOD coordinates use the same centered active grid.

V2 object IDs use a bounded frame-local semantic region projected into the existing
128 by 128 ID range around center `(64,64)`. A probe-owned table joins active index,
render offset, semantic region ID, local alias ID, and signed global region. Object IDs
remain `R32_UINT`; signed coordinates remain CPU-side identity and are never hashed or
truncated into persistent GPU IDs.

V2 composition, generated objects, automatic traversal/rebase, cooked global objects,
prefetch, and authored world partitioning remain out of scope.

## Workload

1. Reproduce Experiment 0023 unchanged, including its complete compatibility chain.
2. Open the accepted V2 pack around `(2^40,-2^40)` and publish one fixed signed window
   through local centers 2, 64, 96, and 125 by changing only the global origin.
3. Compensate each local camera by the exact alias offset. Require all 25 canonical
   slots and content hashes to remain resident with zero I/O/upload after the first
   publication.
4. At each alias, validate exact centered render offsets, semantic region IDs, signed
   joins, CPU/GPU edges, geometry, fixed submission, and frame attachments. Require
   byte-identical color pixels, PNG, raw object-ID, diagnostic PNG, and sampled IDs.
5. Enable terrain LOD and repeat all aliases. Require identical camera patch, LOD hash,
   LOD counts, geometry, adjusted edges, attachments, and GPU aggregate/oracle evidence.
6. Move the same global window through a forward/reverse alias sweep and revisit every
   extreme repeatedly. Require bounded residency and no alias-dependent read/upload.
7. Publish adjacent signed global windows under different aliases. Require the same
   canonical projection shape, exact global joins, 20 retained/five uploads, and exact
   CPU/GPU oracles without semantic collisions or unknown IDs.
8. Hold V2 I/O and copy during adjacent movement. Require the old projection table and
   attachments until one frame-boundary publication. Restart and reproduce all base
   evidence.
9. Run 32 release fixed-window alias rebinds and 32 adjacent-global moves. Report
   projection, schedule, I/O, copy, publication, terrain GPU, capture, and operator
   observation distributions.

## Controlled Variables

- Terrain pack V2, source namespace, 4 KiB payload, exact signed lookup, 50 cache slots,
  25 active entries, one I/O request, one copy transaction, and immutable publication
  remain unchanged.
- Root constants, descriptors, mesh/compute dispatch counts, render targets, reverse-Z,
  height/material encoding, lighting, and active radius remain bounded and fixed.
- The canonical GPU mode is derived from committed V2 source identity, not caller input.
- Signed regions stay `i64` on CPU. GPU positions, patch coordinates, and semantic IDs
  are bounded projections with explicit inverse tables.
- Correctness uses the debug workbench. Release uses validation-disabled benchmark mode
  and makes no speedup claim.

## Metrics

- Source/global/local assignments, slots, active indices, centered region offsets,
  semantic region IDs, inverse joins, projection hashes, and collision/unknown counts.
- Camera alias offset, translated camera bits, view-projection hash, LOD camera patch,
  LOD hash/counts, terrain geometry, edge oracle, and fixed submission evidence.
- Color pixel/PNG, raw object-ID, diagnostic PNG, sample IDs, visible global regions,
  and exact cross-alias comparisons.
- Retained/uploaded/evicted/resident counts, I/O/upload bytes, schedule, copy GPU,
  copy-to-publication, pending, terrain GPU, capture, operator, validation, process,
  device-removal, and lifecycle evidence.

## Pass Criteria

- Experiment 0023 and every V1 attachment/status contract pass unchanged. V1 does not
  enter canonical projection and omits all new optional evidence.
- For one fixed signed window, local centers 2, 64, 96, and 125 preserve all 25 source/
  global slots and content with zero I/O/upload after initial publication.
- Compensated aliases produce exactly equal view-projection bits, centered positions,
  semantic IDs, projection tables, CPU/GPU terrain and LOD oracles, color pixels, PNG,
  object-ID bytes, diagnostic PNG, and requested sample evidence.
- Every emitted V2 terrain object ID resolves through exactly one active projection
  entry to its signed region. There are no collisions, unknown IDs, direct signed-to-
  `u32` conversions, or dependence on physical slots/local aliases.
- Adjacent global movement retains 20 and uploads five exact payloads while projection
  identity remains bounded. I/O/copy holds expose only the complete old projection and
  attachments until publication.
- Cache/request/copy/root-constant/submission sizes remain fixed. Restart, debug/release
  validation, Flavor, Sidecar lifecycle, and device status pass.

## Evidence

The canonical workflow is:

```powershell
runseal :camera-relative-terrain
```

Generated evidence remains ignored under
`out/captures/0024-camera-relative-terrain/`.

## Results

- The complete workflow passed in 623.2 seconds, including Experiment 0023 and its
  recursive compatibility chain. Debug correctness and validation-disabled release
  runs reported no renderer error or device removal.
- Local centers 2, 64, 96, and 125 produced one canonical projection SHA-256
  `d45249b2bc77e778f5497f55511b6054001f87f520d8caa860304cfc11154a0a`
  and one view-projection SHA-256
  `016d993922e9d8e3eee7793baa1b316cde95acfae6702d34c20602544c7e5756`.
  All 25 table entries inverted exactly with zero semantic collisions or mismatch.
- Eight full-resolution alias publications retained all 25 slots with zero reads,
  uploads, evictions, or residency growth. Color, PNG, raw object-ID, and diagnostic
  SHA-256 remained respectively `4d42bacf...`, `2877707e...`, `a7f2f907...`, and
  `fe479ced...`; sampled IDs and visible signed-region joins were exact.
- Four LOD alias extrema shared camera patch `(260,261)`, LOD SHA-256
  `9d66047854d3a198a4ea0de3feffb06ddc473b2b193abf4e6dd75051ecb21d98`,
  counts `25/144/231`, geometry, adjusted edges, color, object-ID, diagnostic, and all
  CPU/GPU oracles exactly.
- Eight adjacent signed windows retained 20 and read/uploaded five 4 KiB payloads each.
  Canonical semantic IDs and render offsets remained fixed while every ID joined the
  current signed window. I/O and copy holds preserved the old projection table and all
  attachments until publication; restart reproduced the base frame exactly.
- Across 32 release alias samples, I/O/upload bytes stayed zero. Copy GPU median/P95/P99
  was 0.001120/0.001504/0.001664 ms, terrain GPU was
  0.141312/0.354304/0.359424 ms, and full perception capture was
  189.305/196.009/201.382 ms.
- Across 32 release adjacent samples, I/O remained exactly 20,480 bytes with total I/O
  median/P95 0.0736/0.1198 ms. Copy GPU median/P95/P99 was
  0.009248/0.009984/0.009984 ms and terrain GPU was
  0.359424/0.379904/0.413696 ms. Operator publication medians were 111.251 ms for alias
  rebind and 125.438 ms for adjacent movement, including frame polling.

## Conclusion

The hypothesis passes. A CPU-owned bounded projection is sufficient to make canonical
V2 terrain rendering, LOD, and perception byte-invariant across the complete legal
local alias range. Signed identity remains in the CPU inverse table, while the GPU
continues to consume the accepted bounded mapping and unchanged shader path.

Terrain no longer blocks automatic origin rollover. Generated-object content,
camera-relative object rendering, object semantics, and V2 atomic composition remain
separate blockers and must pass their own experiments before rebase can be enabled.

## Promotion

Promote V2 centered terrain projection, exact camera translation, projected LOD oracle,
and frame-local semantic inverse evidence as accepted workbench capabilities. Preserve
V1 passthrough and keep V2 composition disabled.
