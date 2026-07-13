# Experiment 0023: Signed Terrain Storage

Status: Accepted

- Related ADRs: [ADR 0026](../../docs/adr/0026-signed-terrain-storage.md)

## Hypothesis

A versioned sparse terrain pack keyed by signed 64-bit regions can make cooked content
identity independent of the bounded local alias: one immutable source namespace plus
one signed region is sufficient for cache residency, while local region IDs remain only
active placement/projection data. Rebinding the same global window to a different local
origin can therefore publish a new mapping with zero I/O and zero GPU upload without
changing canonical terrain content or payload residency.

## Scope

This experiment adds a distinct terrain pack V2. Its sorted index uses signed `(x,z)`
keys, canonical offsets, fixed payload sizes, and per-payload SHA-256. The hash of the
header and complete index is the immutable source namespace and therefore includes all
keys and payload hashes.

The 4 KiB terrain payload keeps the accepted height and material offsets used by the
GPU. Its first 16 bytes identify the signed region; dimensions and height units are
implicit constants of V2. A background worker reads by exact signed key and projects
the decoded tile to the requested local ID for CPU validation. GPU placement continues
to come from the bounded active mapping, not payload identity.

V1 remains byte- and behavior-compatible. A V1 signed schedule retains its accepted
`{global region, local content ID}` key because its payload selection is alias-based. A
V2 signed schedule uses `{source namespace, global region}`. V2 does not authorize local
scheduling, object content, atomic composition, camera traversal, or automatic rebase.

## Workload

1. Reproduce Experiment 0022 unchanged with the V1 source.
2. Cook deterministic V2 packs around zero, plus/minus 2^40, a 32-region corridor, and
   one fixed signed window that can be represented by local centers 64 and 96.
3. Validate canonical round-trip; sorted unique signed keys; negative and far regions;
   payload self-identity; edge continuity; source namespace determinism; malformed
   metadata, key, offset, checksum, and payload rejection.
4. Open V2, publish the fixed global window with local center 64, and validate exact
   source/global/local assignments, CPU/GPU terrain oracles, and attachments.
5. Rebind that same global window through local center 96 and compensate the local
   camera by 32 regions. Require 25 retained entries, zero reads/uploads/evictions, a
   changed local mapping, unchanged source/global content evidence, and independently
   exact terrain oracles at both aliases. Revisit both aliases repeatedly.
6. Move across eight adjacent signed centers. Require 20 retained and five exact signed
   reads/uploads per move, bounded cache ownership, exact edges/LOD, and deterministic
   revisit hits.
7. Open a second pack with the same signed keys but variant payloads. Require a distinct
   namespace and 25 misses instead of stale hits. Reopen the first pack and require its
   resident namespace to hit without I/O.
8. Hold V2 I/O and copy independently. Require the old complete snapshot and attachments
   until publication. Reject an absent signed key before worker submission and roll
   back a corrupted payload before GPU copy.
9. Reject local and composition scheduling against V2 without reservations or cache
   mutation. Restart and reproduce namespaces, global content, mappings, and attachments.
10. Run release adjacent and alias-rebind distributions. Report I/O, upload, copy,
    publication, composed terrain GPU, and operator observation costs.

## Controlled Variables

- Signed keys remain `i64`; payloads remain 4 KiB; active radius remains two; region
  side remains 16 meters; local format-V1 mapping remains 128 by 128.
- Terrain cache capacity remains 50, active capacity remains 25, worker/request/
  completion capacity remains one, and copy publication remains transactional.
- Height/material encoding, GPU descriptors, shaders, active mapping, stable semantic
  IDs, LOD, fixed submission, and perception attachments remain unchanged.
- The source namespace is content-derived and cannot be supplied by the caller.
- Correctness uses the debug namespace. Release timings use validation-disabled
  benchmark mode and make no speedup claim.

## Metrics

- Pack version, signed keys, source namespace, index/payload/file bytes and hashes,
  edge comparisons, malformed-input errors, and deterministic recook equality.
- Cache namespace/global keys, local assignments, retained/uploaded/evicted/resident
  counts, requested signed keys, I/O/upload bytes, transaction IDs, and mapping hashes.
- Canonical content hashes, projected CPU tile hashes, CPU/GPU edge and LOD oracles,
  attachment hashes, old-snapshot hold evidence, and joins from projected semantic IDs
  back to signed regions.
- I/O, schedule, copy GPU, copy-to-publication, pending, terrain GPU, operator, process,
  validation, device-removal, and lifecycle evidence.

## Pass Criteria

- V2 accepts the complete signed domain, rejects malformed or noncanonical data, and
  deterministically binds every payload hash into one source namespace. V1 tests and
  Experiment 0022 remain unchanged.
- Every V2 request is addressed by exact signed key. No local alias selects cooked
  content, and no signed key is converted to `f32`, `u32`, slot, or semantic identity.
- Rebinding one global window from local center 64 to 96 retains all 25 slots, reads and
  uploads zero bytes, changes only local placement/projection, preserves canonical
  content/geometry evidence, and passes CPU/GPU oracles at both aliases. Repeated alias
  switches do not grow residency.
- Adjacent signed movement retains 20 and reads/uploads five payloads. Resident revisits
  read/upload zero. Cache, request, completion, and copy capacities remain bounded.
- Equal signed keys in a different source namespace do not hit stale slots. Returning to
  a still-resident prior namespace hits only that namespace.
- Missing/corrupt content and held I/O/copy never expose a partial snapshot or mutate
  committed cache state. Unsupported local/composition use rejects transactionally.
- Restart, debug/release validation, Flavor, Sidecar lifecycle, fixed GPU submission,
  semantic attachments, and existing terrain oracles pass without device loss.

Exact cross-alias pixels are not a criterion. The accepted terrain path still submits
absolute local `f32` positions and local semantic IDs, so compensated aliases may change
raster bits and object-ID attachments. Camera-relative terrain transforms and global
semantic indirection require separate experiments before automatic rebase is valid.

## Evidence

The canonical workflow is:

```powershell
runseal :signed-terrain-storage
```

Generated evidence remains ignored under
`out/captures/0023-signed-terrain-storage/`.

## Results

- The complete workflow passed in 538.0 seconds, including the unchanged Experiment
  0022 compatibility chain. Debug correctness and validation-disabled release runs
  reported no device removal or renderer error.
- Two identical cooks produced the same 978,944-byte, 235-region pack and file SHA-256
  `dcad3aa19a31876f9eaabdc18bce5694e6e56c15595a738d1215d4ce989d542c`.
  Its source namespace was
  `e22708e1c9669079cc71e71a3adf940cf00d5b9420b8ca860f414b904bc9fa96`.
  A payload variant produced namespace
  `2457fda470f1192464c99112b9b40bd357abd59943369156d79d6d1cf4639b17`.
- Cooking checked 408 signed neighbor edges and 13,464 samples with zero mismatch.
  Format tests covered signed zero crossings, far and negative keys, deterministic
  round-trip, malformed headers, offsets, keys, checksums, padding, and payload binding.
- The initial signed window uploaded 25 payloads and 102,400 bytes. Rebinding it from
  local center 64 to 96 retained all 25 entries and slots with zero I/O, upload, or
  eviction. Eight repeated rebinds remained at 25 resident entries. Canonical content
  SHA-256 remained
  `c49f514b7c9ed4b8fe37b047b7faba911d5017195cd37f9f31b9f78b33eb4cec`.
- Each adjacent move retained 20 and read/uploaded five payloads or 20,480 bytes. The
  cache saturated at 50 entries. Returning after eight moves retained 10 and reloaded
  15, which matches the deterministic bounded-LRU contract rather than an unbounded
  revisit assumption.
- Switching namespaces retained zero and uploaded 25. Returning to the still-resident
  first namespace retained 25 with zero I/O and upload. Missing keys, local/composition
  V2 schedules, malformed indexes, and corrupted payloads left the committed probe and
  attachments unchanged; restoring the payload allowed retry and publication.
- I/O and copy holds kept the complete old color, object-ID, diagnostic, canonical
  content, and slot evidence visible until publication. Restart reproduced canonical
  content and the base attachments exactly.
- Across 32 release adjacent samples, schedule median/P95/P99 was
  25.112/33.088/33.385 ms, I/O total was 0.062/0.069/0.070 ms, copy GPU was
  0.002688/0.003200/0.003488 ms, and terrain GPU was
  0.079872/0.103424/0.104448 ms.
- Across 32 release alias rebinds, I/O bytes remained zero. Copy GPU median/P95/P99 was
  0.000928/0.001408/0.001696 ms and terrain GPU was
  0.138240/0.351232/0.352256 ms. Operator observation includes frame polling and was
  112.967/130.468/147.997 ms.
- The compensated alias color hashes differed (`4d42bacf...` versus `15ecf38d...`) while
  both independent CPU/GPU terrain oracles passed. This confirms that pixel-invariant
  automatic rebase needs a separate camera-relative terrain and semantic-ID decision.
- `runseal :guard` passed all workspace tests and clippy, 41 Deno checks, Flavor with
  zero deny findings, and both debug/release Sidecar doctor and plan checks.

## Conclusion

The hypothesis passes. One immutable source namespace plus one signed region is a
sufficient canonical terrain cache identity. Bounded local IDs can remain publication
and GPU-placement projections without selecting cooked content, and alias rebinding can
reuse canonical GPU residency without I/O.

The result removes the content-integrity blocker for origin rollover but does not make
rollover visually invariant. Automatic rebase remains blocked on camera-relative
terrain transforms and global-to-local semantic projection.

## Promotion

Promote terrain format V2, exact signed worker lookup, source-aware canonical cache
keys, and alias-remapped publication as accepted workbench capabilities. Keep format
V1, local scheduling, and composition unchanged until later experiments replace them.
