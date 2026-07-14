# ADR 0031: Cooked Canonical Object Storage

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0028 gives canonical generated objects exact signed cache identity, region-local GPU
payloads, and stable seeds. ADR 0030 proves that the same cache can prepare one traversal
target speculatively. Object payloads are still generated on the render thread, however,
so no accepted contract yet proves offline-authored object bytes, exact signed lookup, or
bounded object I/O independent from terrain storage.

Experiment 0028 tests storage replacement only. It must preserve the accepted 1,024
`InstanceRecord` payload, 50-slot cache, 25-entry snapshot, copy queue, composition,
projection, grounding, animation, semantics, traversal, and prefetch behavior.

## Decision

- `region-format` V2 is a distinct signed-key object pack with a 96-byte canonical header,
  64-byte sorted index entries, 4 KiB alignment, fixed 20,480-byte chunks, and per-chunk
  SHA-256 verification. Writing remains offline-only through `region-cooker`.
- The object cache source namespace is SHA-256 over the complete header and sorted index,
  including payload checksums. Any key, metadata, source identity, or payload change
  therefore creates a distinct cache source.
- The header also carries an authored stable-seed namespace. Stable seeds derive from
  that namespace plus exact signed region, not from the pack source namespace. This
  avoids a hash cycle because stable seeds are part of payload checksums, while allowing
  cooked bytes to remain identical to the accepted generated fixture.
- `canonical-object-fixture` owns only the deterministic fixture math shared by cooker,
  generated compatibility, and CPU oracle. It is not an asset schema or runtime source.
- Opening a cooked object pack selects object source independently from terrain source.
  No open pack preserves the generated path and existing serialized status exactly.
- Canonical composition reserves the existing object cache before one bounded worker
  request. The worker reads and verifies only missing signed chunks, then submits them
  through the existing object copy queue. Publication remains one matched terrain/object
  pair; no second cache, descriptor mapping, or GPU contract is introduced.
- One worker, one request slot, one completion slot, one active transaction, existing
  protected slots, and latest-demand backpressure are fixed capacities. Object I/O and
  copy gates are diagnostic controls over the same transaction.
- Missing or corrupt object data cancels before copy, records a source diagnostic, and
  leaves the old pair published. Completed work from the terrain half is immutable cache
  population and may be reused by retry.

## Consequences

Cooked and generated arbitrary-Q8 base payloads are byte-identical even though their
cache source namespaces differ. Adjacent movement reads five object chunks, diagonal
movement reads nine, revisit and alias rebinding read zero, and source replacement cannot
cross-hit. Completed prefetch still makes demand `25/0` in both halves.

Object and terrain packs may change independently while atomic publication remains the
only visible state transition. Cooked sourcing can be disabled to restore generation,
and status for the new source is absent until a pack is opened.

This decision accepts deterministic canonical object storage and bounded streaming. It
does not accept a general asset database, legacy importer, heterogeneous object schema,
persistent gameplay identity, collision, navigation, networking, or mod content.

## Evidence

- [Experiment 0028](../../experiments/0028-cooked-canonical-objects/README.md) records
  recursive compatibility, codec rejection tests, generated/cooked byte equality,
  movement, independent source replacement, I/O/copy promotion, rollover, failures,
  disable/restart, and release distributions.
- The full workflow passed in `1062.9 s`; 32 reactive and 32 prepared release crossings
  retained fixed capacities and five object reads per preparation.
- Reactive/prepared object I/O P95 was `0.1992/0.2097 ms`; combined composition GPU P95
  was `0.139264/0.143360 ms`. No validation, lifecycle, oracle, semantic, or device loss
  occurred.

## Reproduction

```powershell
runseal :cooked-canonical-objects
```

The command writes the ignored report to
`out/captures/0028-cooked-canonical-objects/acceptance.json`.
