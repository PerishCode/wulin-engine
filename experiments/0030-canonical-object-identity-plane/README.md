# Experiment 0030: Canonical Object Identity Plane

Status: Accepted

## Hypothesis

A canonical object can retain the same render and animation identity when records are
reordered if its region seed is combined with an explicit authored local ID instead of
the record ordinal. A parallel fixed-capacity GPU identity plane can add that invariant
without changing `InstanceRecord`, schema-1 behavior, cache capacity, steady-state CPU
submission, or the authority of bytes read from the GPU.

## Scope

`region-format` gains signed V2 payload schema 2. Each region still contains exactly
1,024 20-byte `InstanceRecord` values, followed by 1,024 unique `u32` local IDs. The
24,576-byte physical payload and its index checksum cover both planes. Schema 1 remains
readable and synthesizes ordinal IDs as format interpretation; its 20,480-byte physical
payload and checksum remain unchanged.

The cooker creates a schema-2 identity fixture by deterministically permuting the
accepted authority records while storing each record's original ordinal as its authored
local ID. A second permutation produces a distinct source with the same record/ID pairs.
The workbench and shared fixture code do not contain either permutation.

The async object cache retains 50 record resources and gains 50 bounded 4 KiB identity
resources. Schema-1 slots use ordinal identity and must not add per-transaction identity
I/O or copies in the recursive compatibility workflow. Schema-2 uploads publish record
and identity resources transactionally under the same cache key and pair token.

Every GPU path that derives stable keys, archetypes, yaw, animation clips, pose phases,
or material variation consumes the published local ID. Composition probes read back the
25 record pages and 25 identity pages. Schema-2 evidence reconstructs the exact physical
payload order and compares it with the signed pack index checksum. CPU oracles consume
the same GPU-read record/ID pairs.

## Workload

1. Reproduce Experiment 0029 and its complete recursive chain with schema 1. Require
   unchanged record hashes, 512,000-byte record readback, 25 record copies per probe,
   transaction byte counts, attachments, and all accepted status contracts.
2. Cook the schema-2 identity fixture twice and a second deterministic permutation.
   Require unique IDs `0..1023` per region, deterministic files, the accepted stable-seed
   namespace, distinct complete-index source namespaces, and exact malformed-input
   rejection for ID duplication, range, size, padding, and checksum failures.
3. Publish schema-1 authority at `(2^40,-2^40)`, then switch only objects to schema 2.
   Require terrain `25/0`, objects `0/25`, one atomic pair, and no early identity mapping.
4. Probe schema 2. Require 25 record pages, 25 identity pages, 25,600 unique `(region,
   local ID)` keys, zero pack checksum mismatch, and CPU/GPU grounding, skeletal,
   contact, semantic, material, occlusion, and attachment oracles with zero mismatch.
5. Compare schema-1 ordinal order, schema-2 first permutation, and schema-2 second
   permutation. Require equal identity-keyed position/ground/archetype/yaw/pose evidence
   and rendered attachments despite different payload/source hashes and record order.
6. Move adjacent, diagonal, revisit, alias, prefetch, and rollover targets. Hold object
   I/O and record/identity copy. Require the accepted cache/publication behavior and no
   mixed record/identity generation.
7. Inject missing, corrupt-record, corrupt-ID, and duplicate-ID sources; disable and
   restart. Require complete old identity authority, pre-publication rollback, exact
   recovery, and no stale identity descriptors.
8. Run 32 reactive and 32 prepared release crossings. Record record/identity I/O and
   copy bytes, pair publication, GPU work, fixed capacities, and probe-only readback;
   make no speedup claim.

## Controlled Variables

- `InstanceRecord`, record count, record-plane bytes, region/stable-seed derivation,
  signed keys, 50 cache slots, 25 active entries, terrain composition, camera projection,
  grounding, mesh/animation catalogs, and object-ID attachments remain unchanged.
- Schema-1 files, checksums, generated records, ordinal stable keys, copy counts, and
  status remain compatible. The new identity plane is not embedded in float bits or
  inferred from record order for schema 2.
- Local IDs are unique only within one exact signed region. They are content identity
  input, not a public gameplay/network ID and not a semantic attachment value.
- Identity resources, descriptors, upload space, readback space, worker fields, and
  transaction state have fixed capacities. No per-frame allocation or CPU payload cache
  is introduced.
- Variable record counts, sparse occupancy, authored archetype/rotation/scale, collision,
  navigation, networking, legacy import, and mod content remain out of scope.

## Metrics

- Payload schema, file/source/stable namespace, record-plane, identity-plane, combined
  payload, index, and active-page hashes.
- Per-region record/ID pair count, duplicate/missing ID count, ordinal mismatch count,
  stable-key collision count, and identity-keyed evidence hashes.
- Record and identity I/O/upload/readback bytes and copies, descriptor/resource counts,
  cache/transaction capacities, fences, gates, and publication generation joins.
- Grounding, skeletal, pose, archetype, yaw, material, occlusion, semantic, contact,
  color, object-ID, depth, diagnostic, and PNG evidence.
- Median/P95/P99 object I/O, copy/publication, frame, and combined GPU times.

## Pass Criteria

- Experiment 0029 passes unchanged for schema 1. Old packs and generated content retain
  ordinal identity and do not pay schema-2 transaction costs.
- Schema-2 pack validation binds 1,024 unique local IDs and both physical planes to the
  exact signed region and complete-index source namespace.
- Every schema-2 runtime stable key uses the GPU-published local ID. No shader or CPU
  oracle derives canonical identity from record ordinal.
- Reordering storage changes source/payload order but not identity-keyed render,
  animation, material, semantic, grounding, contact, or attachment results.
- Record and identity resources publish as one cache generation through movement,
  prefetch, rollover, holds, failures, disable, and restart. No mixed plane, stale tail,
  validation error, unbounded allocation, lifecycle leak, or device removal occurs.

## Evidence

The canonical workflow will be:

```powershell
runseal :canonical-object-identity
```

Generated evidence will remain ignored under
`out/captures/0030-canonical-object-identity-plane/`.

## Results

The complete recursive workflow passed in 1,452.4 seconds. Experiment 0029 and its
compatibility chain passed first. A fresh schema-1 publication uploaded 25 record pages
and retained the accepted 512,000-byte record contract while recording zero transaction
identity copies or bytes.

Each schema-2 order contained 1,295 signed regions, 31,825,920 payload bytes, and the
accepted stable-seed namespace `6007faad...b8`. Order A and B received distinct source
namespaces (`1ece86d4...39d8` and `a2d5e1dc...767f`) and distinct active payload/index
hashes. Both nevertheless produced identity-keyed hash `a3f77388...a5c7`, stable-key
hash `cd2d240e...4203`, and byte-identical color, object-ID, diagnostic, and PNG
attachments.

Every schema-2 probe decoded 25,600 record/ID pairs from 512,000 record bytes plus
102,400 identity bytes. All IDs were unique within their region, all 25 combined page
checksums matched the exact signed pack index, and the CPU/GPU grounding, contact,
skeletal, semantic, and attachment oracles passed. The fixed probe allocations were
524,288 record bytes and 131,072 identity bytes. A source switch copied 25 record and 25
identity pages under one publication; revisit and alias rebind copied none.

Adjacent and diagonal movement, object-I/O and copy holds, revisit, compensated alias
rebind, restart, and missing/corrupt-record/corrupt-ID/duplicate-ID rollback all retained
complete old authority and recovered without stale descriptors. Both 32-sample release
sweeps recorded 800 record and 800 identity page copies. Reactive/prepared object-I/O
median/P95/P99 was `0.1798/0.2396/1.1329 ms` and `0.2088/0.3083/0.3185 ms`; pair
publication was `124.5224/126.2930/133.8066 ms` and
`93.2638/94.7137/94.7240 ms`. No validation, capacity, oracle, lifecycle, or device
removal failure occurred.

## Conclusion

Accepted. Canonical object behavior is keyed by an explicit GPU-published authored
local ID rather than physical record ordinal. The bounded parallel identity plane makes
record reordering behaviorally invisible while preserving fixed cache capacity, one
atomic publication, fixed GPU submission, and GPU-read payload authority. Local IDs
remain region-local content identity inputs, not public gameplay or network IDs.
