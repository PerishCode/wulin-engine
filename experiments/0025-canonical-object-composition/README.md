# Experiment 0025: Canonical Generated Object Composition

Status: Accepted

- Related ADRs: [ADR 0028](../../docs/adr/0028-canonical-generated-object-composition.md)

## Hypothesis

A versioned procedural object source keyed by exact signed region, combined with the
accepted centered frame projection, is sufficient to make generated-object residency,
animation, grounding, rendering, and perception independent of bounded local aliases.
The same projection can therefore publish one canonical V2 terrain/object pair without
changing GPU capacity or submission shape.

## Scope

This experiment adds one canonical mode selected only when composition consumes a
signed terrain V2 source. Legacy local and signed format-V1 composition keep their
existing cache keys, payloads, positions, stable keys, object IDs, probes, and
attachments exactly.

Canonical generated-object identity is `{object source namespace, signed region}`.
The source namespace identifies the procedural generator revision and fixture. A region
payload stores region-local positions and a deterministic stable seed derived on CPU
from the namespace and exact signed coordinate. It stores neither the local alias,
frame projection, physical slot, nor terrain source identity.

At publication, the accepted centered projection is shared by terrain and objects.
The existing 32-bit active mapping word packs the bounded instance slot, terrain slot,
and projected semantic region ID. Object shaders reconstruct bounded positions from
region-local payload coordinates, use the canonical seed for animation and appearance,
and emit frame-local region handles. A CPU table retains exact signed object identity
and inverts every emitted handle.

Cooked object packs, authored objects, public persistent object IDs, surface/occlusion
promotion, automatic origin rollover, prefetch, collision, navigation, and networking
remain out of scope.

## Workload

1. Reproduce Experiment 0024 unchanged, including the complete V1/V2 compatibility
   chain, and reproduce Experiment 0021 legacy signed composition attachments.
2. Open the accepted signed V2 terrain source around `(2^40,-2^40)`, select the
   `arbitrary-q8` generator, and publish one canonical terrain/object pair.
3. Rebind that fixed signed 25-region window through local centers 2, 64, 96, and 125.
   Require both caches to retain all 25 canonical slots with zero terrain I/O, terrain
   upload, or object upload after initial publication.
4. At every alias, require exact object source/content hashes, stable seeds, centered
   positions, semantic inverse entries, grounding, contact, animation, LOD, geometry,
   GPU/CPU oracles, color pixels/PNG, raw object-ID, diagnostic PNG, and samples.
5. Move through adjacent signed windows under changing aliases. Require both caches to
   retain 20 and upload five, unchanged retained object payload identity, bounded
   projection shape, exact signed joins, and matched terrain/object publication.
6. Switch between two signed terrain namespaces at one fixed signed window. Require
   terrain to miss by source while generated objects retain by their independent source;
   both halves must still publish atomically.
7. Hold terrain I/O, terrain copy, and object copy independently. Require the complete
   old pair, projection, oracles, and attachments until one frame-boundary publication.
8. Reject missing/corrupt terrain and invalid canonical mode before pair mutation.
   Restart and reproduce the base pair exactly.
9. Run 32 release alias rebinds and 32 adjacent moves. Report both half distributions,
   object generation/copy, terrain I/O/copy, pair publication, GPU composition, capture,
   validation, process, and device status.

## Controlled Variables

- Terrain V2 format/source namespace, exact signed lookup, 4 KiB terrain payload,
  50-slot terrain cache, and 25-entry terrain snapshot remain unchanged.
- Object records remain the accepted fixed-size runtime record. Object cache capacity
  remains 50 regions with 1,024 records per region and one bounded copy transaction.
- Terrain and skeletal root-constant counts, descriptors, render targets, mesh/compute
  dispatch counts, indirect submissions, reverse-Z, LOD settings, and animation settings
  remain fixed.
- Signed coordinates remain CPU identity. The GPU receives only bounded frame-local
  semantic regions and canonical stable seeds; neither is a persistent object ID.
- Correctness uses the debug workbench. Release uses validation-disabled benchmark mode
  and makes no speedup claim.

## Metrics

- Terrain/object source namespaces, signed/global/local mappings, physical slots,
  payload/stable-seed hashes, retained/uploaded/evicted counts, and bytes.
- Projected camera, semantic region/object IDs, exact inverse joins, collision/mismatch
  counts, centered positions, and projection hashes.
- Grounding position/numerator hashes, boundary comparisons, contact distributions,
  animation/pose/LOD/geometry counts, and CPU/GPU oracle mismatches.
- Color pixel/PNG, raw object-ID, diagnostic PNG, semantic classes, samples, and exact
  alias comparisons.
- Schedule, generation, I/O, copy GPU, pair publication, composition GPU, capture,
  operator observation, validation, process, device-removal, and lifecycle evidence.

## Pass Criteria

- Experiment 0024 and legacy Experiment 0021 pass unchanged. V1 snapshots omit all new
  optional source/projection evidence and retain byte-identical status and attachments.
- Canonical cache identity contains object source plus exact signed region and excludes
  local alias, physical slot, projected semantic ID, and terrain namespace.
- Fixed-window alias extrema retain all 25 regions in both halves with zero reads/uploads
  and exact payload, seed, projection, grounding, animation, geometry, and attachment
  evidence.
- Every emitted canonical object handle resolves through exactly one active entry to
  its signed region and source. There are no semantic/stable-seed collisions, unknown
  IDs, direct signed-to-`f32` conversion, or alias-dependent payload bytes.
- Adjacent movement retains 20 and uploads five in each cache. A terrain namespace
  switch uploads terrain only while retaining all canonical generated objects.
- Three independent holds expose only the complete old pair until atomic publication.
  Failure and restart are transactional and deterministic.
- Cache/request/copy/root-constant/submission sizes remain fixed. Debug/release
  validation, Flavor, Sidecar lifecycle, and device status pass.

## Evidence

The canonical workflow will be:

```powershell
runseal :canonical-object-composition
```

Generated evidence will remain ignored under
`out/captures/0025-canonical-object-composition/`.

## Results

- The complete canonical workflow passed in 724.6 seconds, including Experiment 0024
  and its recursive compatibility chain. Debug correctness and validation-disabled
  release runs used the reference RTX 4070 Ti SUPER and reported no renderer error or
  device removal.
- The base window produced object source SHA-256 `6007faad...`, object content
  `e78dc183...`, stable-seed table `2e9abc2e...`, terrain projection `d45249b2...`,
  grounding `c9ad7180...`, and position `016f5042...`. All 25 object entries joined the
  terrain projection exactly with zero source, seed, semantic, or payload mismatch.
- Eight debug alias rebinds through local centers 2, 64, 96, and 125 retained all 25
  terrain and object regions with zero I/O/upload. Color, PNG, raw object-ID, and
  diagnostic SHA-256 remained respectively `b9b4e0a7...`, `ac87bf5a...`, `395009b8...`,
  and `51c2223a...`; grounding, contact, animation, LOD, geometry, and CPU/GPU aggregates
  were exact.
- Eight adjacent windows retained 20 and uploaded five regions in each half: exactly
  20,480 terrain bytes and 102,400 object bytes. Every retained signed region preserved
  its stable seed. A terrain namespace switch uploaded 25 terrain regions while all 25
  generated-object regions remained resident with zero upload.
- Terrain-I/O, terrain-copy, and object-copy holds kept the complete old projection,
  grounding, skeletal oracle, color, and semantic attachments until atomic publication.
  Missing terrain and three standalone mode requests left the pair unchanged. Corrupt
  terrain failed and discarded the staged object snapshot; retry reused all 25 valid
  immutable object cache entries while terrain uploaded 25 repaired payloads.
- Across 32 release alias samples, transfer bytes remained zero. Combined terrain and
  skeletal GPU median/P95/P99 was 3.786/5.640/7.051 ms and pair publication was
  25.550/26.239/26.453 ms.
- Across 32 release adjacent samples, terrain I/O median/P95 was 0.0713/0.0863 ms,
  terrain copy GPU median/P95 was 0.00925/0.00960 ms, and object generation median/P95
  was 0.0559/0.0636 ms. Combined GPU median/P95/P99 was 3.800/6.941/7.001 ms and pair
  publication was 25.623/34.022/34.377 ms.

## Conclusion

The hypothesis passes. Canonical procedural objects can retain exact signed identity,
stable content, animation, grounding, rendering, and perception while consuming the
same bounded camera-relative projection as V2 terrain. One atomic pair now publishes
both halves without local aliases or physical slots leaking into content identity.

Generated objects and V2 composition no longer block automatic origin rollover.
Rollover scheduling and continuity remain unaccepted until their own experiment passes;
authored objects, persistent public identity, and broader world systems remain separate.

## Promotion

Promote canonical object source/cache identity, alias-independent region-local payloads,
shared terrain/object projection, exact CPU/GPU stable keys, frame-local semantic joins,
and V2 atomic composition as accepted workbench capabilities. Preserve every V1 path
and reject canonical snapshots outside atomic composition.
