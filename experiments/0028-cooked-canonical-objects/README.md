# Experiment 0028: Cooked Canonical Objects

Status: Accepted

- Related ADRs: [ADR 0031](../../docs/adr/0031-cooked-canonical-object-storage.md)

## Hypothesis

The accepted canonical object GPU payload and cache do not require runtime procedural
generation. A versioned object pack keyed by exact signed region, combined with one
bounded background reader and an index-derived object source namespace, can replace
generated materialization while preserving composition, prefetch, identity, semantic,
grounding, animation, and fixed GPU submission contracts.

## Scope

`region-format` gains a distinct signed-key V2 pack for the existing 1,024-record,
20,480-byte region payload. Its header and complete sorted index define the object source
namespace. Every index entry binds one signed region to one aligned fixed-size payload
and checksum. The header carries a separate authored stable-seed namespace so record
identity can remain byte-compatible without forming a hash cycle with the complete-index
source namespace. Writing remains offline-only through `region-cooker`; the workbench is
a read-only consumer.

The cooker materializes the accepted arbitrary-Q8 canonical fixture offline so runtime
records remain byte-compatible with the existing GPU and CPU oracle. This deterministic
fixture proves authored storage and lookup, not a general asset schema or legacy import.

Opening an object pack selects its source independently from terrain source. Canonical
composition reserves object cache slots before issuing one bounded worker request,
validates only missing chunks, submits verified uploads through the existing copy queue,
and publishes only with the matched terrain half. With no object pack open, the accepted
generated source and all V1 paths remain unchanged.

## Workload

1. Reproduce Experiment 0027 and its complete compatibility chain with no object pack
   open. Generated object source, status shape, hashes, and timings remain unchanged.
2. Cook the same signed object fixture twice. Require byte-identical files, metadata,
   sorted index, source namespace, region payloads, stable seeds, and file hash. Reject
   malformed header, duplicate/out-of-order keys, range overflow, padding, checksum, and
   unsupported version/payload-schema values.
3. Open one object pack and publish a canonical terrain/object pair near
   `(2^40,-2^40)`. Require exact equality with generated object positions, stable seeds,
   GPU payload bytes, grounding, animation, LOD, geometry, semantics, CPU/GPU oracles,
   and rendered attachments.
4. Move through adjacent, diagonal, revisit, alias-rebind, and origin-rollover targets.
   Require object I/O/upload counts of `5`, `9`, `0`, `0`, and the accepted rollover
   behavior while resident capacity remains 50.
5. Switch only object pack namespace while terrain source remains fixed. Require terrain
   `25/0`, objects `0/25`, one atomic pair, and no cross-source cache hit. Switch only
   terrain source and require the inverse independence.
6. Hold object I/O and object copy across demand and prefetch. Require complete old pair,
   one latest target, exact pending promotion, and no early object mapping or semantic
   visibility. Completed prefetch must still make demand `25/0` in both halves.
7. Request missing and corrupt object chunks. Require pre-copy rollback, no mixed pair,
   no retry churn, valid immutable terrain cache reuse, deterministic recovery after
   reopening a valid pack, and unchanged camera/basis.
8. Disable cooked object sourcing, restart, and reproduce both generated and cooked base
   frames. Run 32 adjacent reactive cooked controls and 32 cooked/prepared crossings in
   release mode with fixed probe/capture settings.

## Controlled Variables

- `InstanceRecord`, records per region, region payload bytes, canonical region-local
  coordinates, stable-seed meaning, 50-slot object cache, and 25-entry active snapshot
  remain unchanged.
- Terrain source, terrain worker, pair publication, projection, prefetch, rollover,
  descriptors, root constants, dispatches, indirect execution, shaders, and attachments
  remain unchanged.
- Object V2 writing is offline-only. Runtime uses one worker, one request slot, one
  completion slot, one active transaction, and existing copy/backpressure capacities.
- Signed keys and source namespaces stay CPU-only. Pack records contain no local alias,
  physical slot, projected semantic region, terrain source, or camera state.
- Correctness uses the debug workbench. Release uses validation-disabled benchmark mode
  and makes no speedup claim.

## Metrics

- Pack metadata, file/index/source hashes, signed keys, offsets, checksums, payload bytes,
  index-read bytes, chunk-read bytes, read/verify times, and worker queue capacities.
- Terrain/object source namespaces, retained/uploaded/evicted/resident counts, cache
  slots, stable-seed/payload hashes, reservation/copy/publication fences, and rollback.
- Pair token, traversal/prefetch state, basis/origin/camera, semantic inverse joins,
  grounding/contact, animation/LOD/geometry, all attachment hashes, and mismatch counts.
- Object I/O, copy, pending, prefetch completion, pair publication, combined GPU,
  capture, operator observation, validation, process, and device status distributions.

## Pass Criteria

- Experiment 0027 passes unchanged without an object pack. V1 and generated source paths
  remain byte-compatible and omit cooked-object status.
- Signed V2 object pack round-trip and malformed-input tests pass. Source namespace is
  derived from the complete canonical header/index and changes when source identity does.
- Cooked and generated base content is exact at payload, stable-seed, oracle, semantic,
  and attachment levels. No runtime procedural generation occurs for cooked uploads.
- Cache movement, source independence, prefetch 25/0 demand, rollover, holds/latest-wins,
  missing/corrupt rollback, disable, and restart meet their declared bounds.
- Worker/request/completion, cache, copy, descriptor, root-constant, and GPU submission
  capacities remain fixed. Validation, Flavor, Sidecar lifecycle, and device status pass.

## Evidence

The canonical workflow will be:

```powershell
runseal :cooked-canonical-objects
```

Generated evidence will remain ignored under
`out/captures/0028-cooked-canonical-objects/`.

## Results

The complete recursive workflow passed in `1062.9 s`. Experiment 0027 and its prior
compatibility chain passed unchanged before the signed object codec and runtime gates.
The accepted object pack contained 1,295 signed regions and 26,521,600 payload bytes;
its stable-seed namespace remained the arbitrary-Q8 fixture identity while its complete
header/index produced an independent source namespace.

Fresh generated and cooked runs produced the same object `uploadedSha256`, stable seeds,
positions, grounding hashes, animation/LOD/geometry oracles, semantic joins, contact
residuals, color, PNG, object-ID, depth, and diagnostic attachments. Cooked reports used
`payloadSource: cooked-pack`, reported `generationMs: 0`, and read no payload at open.

Adjacent, diagonal, revisit, and alias movement read/uploaded `5`, `9`, `0`, and `0`
object chunks. Object-only source replacement produced terrain `25/0` and objects `0/25`;
terrain-only replacement produced terrain `0/25` and objects `25/0`. Object I/O and copy
holds promoted the exact pending prefetch, and completed preparation made both demand
halves `25/0`. Missing and corrupt object requests each failed once before copy, retained
the complete old pair, reused valid terrain cache work, and recovered deterministically.

Across 32 reactive and 32 prepared release crossings, every object preparation read five
chunks. Object I/O median/P95/P99 was `0.1557/0.1992/0.2159 ms` and
`0.1502/0.2097/0.2186 ms`; pair publication was `0.5124/1.1348/1.2834 ms` and
`0.3604/0.5180/0.5623 ms`. Combined composition GPU time was
`0.117760/0.139264/0.143360 ms` and `0.117760/0.143360/0.144384 ms`. Validation,
capacity, lifecycle, oracle, semantic, and device-removal checks passed.

## Conclusion

Accepted. Runtime procedural generation is not required by canonical object residency or
GPU execution. A distinct signed V2 pack, complete-index cache namespace, authored
stable-seed namespace, one bounded reader, and the existing reservation/copy/publication
path preserve all accepted behavior without adding a cache tier or general asset system.
