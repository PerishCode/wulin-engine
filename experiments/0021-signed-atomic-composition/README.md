# Experiment 0021: Signed Atomic Composition

Status: Accepted

- Related ADRs: [ADR 0024](../../docs/adr/0024-signed-atomic-composition.md)

## Hypothesis

A shared signed 64-bit global region window can own both terrain and generated-object
cache identity while one explicit bounded local alias continues to drive format-V1
terrain, object fixture generation, GPU placement, and semantic IDs: terrain and object
halves publish one matched global/local snapshot, equal aliases at different far anchors
cannot false-hit either cache, adjacent movement retains 20 of 25 entries in both
caches, and all accepted composed GPU output remains unchanged.

## Scope

This experiment promotes Experiment 0020's signed window mapping into a shared streaming
address owner and adds an explicit `composition.global.schedule` path. One global window
is mapped once to the local 128 by 128 address space, then supplied to terrain and object
reservations. Both caches identify entries by signed global key plus local content
binding. Composition stores and validates complete ordered global/local assignments
before committing both staged halves at one frame boundary.

The object half remains the accepted deterministic generated composition fixture. This
experiment does not claim signed cooked-object pack lookup or a persistent object world
format. Legacy `composition.schedule` and camera traversal remain unchanged and use the
existing local cache namespace. Global scheduling is manual and rejects active camera
traversal.

This experiment does not change terrain format V1, region format V1, terrain payloads,
instance records, GPU descriptors, object `region_id`, stable GPU keys, semantic IDs,
shaders, grounding, LOD, submission shape, automatic traversal, render-origin policy,
prefetch, collision, navigation, networking, or authored partitioning.

## Workload

1. Reproduce Experiment 0020 and the accepted local atomic composition workflow.
2. Publish a global composition window at zero, `(2^40,-2^40)`, and
   `(-2^40,2^40)` using equal local aliases. Require distinct exact global pair hashes,
   25 terrain uploads, 25 object uploads, and byte-identical local probes/attachments.
3. At a far origin, move one region on X and then Z. Require both caches to retain 20
   regions and upload five. Revisit while the union remains resident and require 25
   retained, zero uploads, and unchanged pair identity.
4. Hold the terrain I/O half, terrain copy half, and object copy half independently.
   Require one complete old pair and attachments until both halves stage and one atomic
   publication exposes the complete new global/local pair.
5. Request a global window whose local terrain aliases are absent from the sparse pack.
   Require synchronous rejection, cancellation of both reservations, no generated object
   submission, no cache/publication mutation, and unchanged attachments.
6. Reject local-window range and signed-overflow failures before reservation. Change to
   an unseen global origin with equal local aliases and require zero false hits in both
   caches.
7. Restart and reproduce a far-anchor pair. Run 32 release adjacent/revisit cycles and
   report pair publication, terrain I/O/copy, object generation/schedule/pending, copy
   fence/hold, and combined GPU distributions.

## Controlled Variables

- Global coordinates are signed 64-bit integers. One checked shared mapping produces
  the ordered global/local region list consumed by both halves.
- Local origin remains `(64,64)`, active radius remains two, and each cache remains 50
  physical slots with 25 active entries.
- Terrain payloads and generated object records are determined only by local content
  IDs. Global identity never enters GPU buffers or semantic IDs.
- One terrain I/O, one terrain copy, one object copy, and one composition pair may be in
  flight under their accepted bounded contracts.
- Correctness uses the debug namespace. Timings use the release namespace with
  validation disabled and make no speedup claim.

## Metrics

- Pair global origin/center/radius, local config, ordered global/local mapping/hash,
  terrain/object assignment hashes, duplicate/mismatch counts, tokens, generations,
  and publication counts.
- Per-half retained, uploaded, evicted, protected, resident, free, payload/instance
  bytes, physical slots, and transaction identities.
- Terrain I/O/read/verify, terrain copy, object generation/schedule/pending, object copy
  fence/hold, pair publication, combined GPU, and operator-observed median/P95/P99
  distributions.
- Local composition, grounding, LOD/contact, color, PNG, object-ID, diagnostic, process,
  validation, and device-removal evidence.

## Pass Criteria

- Both halves expose the same exact 25-entry signed/global-local mapping at zero and
  plus/minus 2^40. Equal local aliases at different anchors upload 25 entries in each
  cache and produce distinct logical hashes with byte-identical local output.
- Adjacent movement reports exactly 20 retained and five uploaded entries in both
  caches. Resident revisit reports 25 retained and zero uploaded/read/generated bytes.
- No frame or status exposes terrain from one global window with objects from another.
  Each independent hold keeps the complete old pair until one matched commit.
- Missing terrain aliases and arithmetic/range failures leave both committed caches,
  pair counters, mappings, and attachments unchanged. No object work is submitted after
  terrain preflight rejects the request.
- Terrain and object residency never exceed 50 each, active mappings never exceed 25,
  and coordinate magnitude does not affect allocation or submission shape.
- Experiment 0020 and legacy composition/traversal compatibility pass without changing
  formats, payloads, shaders, GPU submission, semantic IDs, grounding, or canonical
  attachments.
- Debug/release validation, restart, Flavor, and Sidecar lifecycle pass without device
  loss, hidden fallback, unbounded growth, or residual process.

## Evidence

The canonical workflow is:

```powershell
runseal :global-composition
```

Generated evidence remains ignored under
`out/captures/0021-signed-atomic-composition/`.

## Results

The canonical workflow passed on 2026-07-13 in 457.4 seconds. It passed Experiment 0020
unchanged; that workflow recursively passed Experiment 0018 and its complete 0017 ->
0016 -> 0015 compatibility chain.

- The deterministic sparse pack contained 207 local terrain regions and had file hash
  `76a37084e1aa37b8331d66aa899cdac3f68992db82edfa79131385f6b63685e8`.
  The debug layer was enabled for correctness and disabled for release measurement;
  neither path reported device removal.
- Zero, `(2^40,-2^40)`, and `(-2^40,2^40)` produced exact global pair hashes
  `89bacb8c112deb1a426110bb3d33bbbb21959446e9d0ab3d7a9453ef390c6437`,
  `c094fd79322493121fcbe881565e44f1504c46390ff665a96620a023334e3b1c`,
  and `e9cc23c044c9eb4af35101a9d8d158c73ec0e2caa28ffe1dfe24101adf942bc7`.
- Equal local aliases preserved one local composition probe, color hash
  `579f3800a4f9603e5298919a7da34fe66a04f54d6bbcd464666dfd67449c158a`,
  PNG hash `8e12ebe39b1aadea32c5efb75fbcd9c7f7af92f49dc81ffeec21e76f99095b70`,
  object-ID hash `20d8017adf2bcde946eff8a8f834d563d9f703b0a4a3bf142fd06478c27fcc75`,
  and diagnostic hash
  `a52a77905b8c395ead2df585025e1f1d018ff372c44d1a3cbd63aa9ff66e3795`.
- Each one-region X or Z move retained 20 and uploaded five entries in both caches:
  `20,480` terrain bytes and `102,400` generated-object bytes. Resident revisits
  retained 25 and uploaded zero bytes in both halves. Neither cache exceeded 50 slots.
- Terrain I/O and terrain copy holds exposed `terrain=in-flight, instance=staged`;
  object copy exposed `terrain=staged, instance=in-flight`. Every held probe and
  attachment remained the complete old pair until one matched publication.
- Missing terrain alias `12094` was rejected as `stream_failed`. An out-of-window alias
  and exact signed subtraction overflow were rejected as
  `invalid_global_composition_config`. Pair token, both reservations, copy fence,
  committed transactions, probes, and attachments remained unchanged.
- An unseen global origin with the same local aliases retained zero and uploaded 25 in
  both caches. Restart changed process identity and reproduced the exact far mapping,
  local probe, and attachments.
- Across 32 release adjacent moves, terrain schedule time was
  25.2569/33.7726/58.1009 ms median/P95/P99; copy GPU time was
  0.009024/0.009568/0.009952 ms and verified I/O total was
  0.0625/0.0767/0.0872 ms. Generated-object preparation was
  0.0224/0.0316/0.0339 ms and schedule time was 0.2316/0.2779/0.4632 ms.
- Pair publication was 25.6015/34.2733/58.6251 ms, combined composed GPU time was
  0.214016/0.260096/0.270336 ms, and operator-observed publication was
  192.5076/230.8213/252.0948 ms. Across resident revisits, both payload byte counts were
  zero; terrain schedule was 25.0778/33.6471/33.6652 ms and object schedule was
  0.0913/0.1353/0.1565 ms.

These are laboratory control-path distributions and include Sidecar/frame-boundary
observation where stated. They are not a normal-frame speedup claim.

## Conclusion

The hypothesis passes. One exact signed global window can own terrain and generated
object cache identity while both halves continue to consume bounded local content IDs
and publish one atomic snapshot. Coordinate magnitude does not change cache capacity,
payload shape, GPU submission, semantic projection, or local composed output.

The experiment accepts shared signed identity for the deterministic generated-object
composition fixture. It does not accept cooked-object global lookup, a persistent world
format, automatic global traversal/rebase, or direct signed coordinates in GPU buffers.

## Promotion

Promote `GlobalRegionConfig` as the shared streaming address owner, signed/content-bound
object cache keys, typed global composition scheduling, matched global/local pair
validation, and physical-slot-independent pair evidence. Preserve explicit manual
scheduling and all local format/GPU/semantic contracts until a later experiment replaces
them.
