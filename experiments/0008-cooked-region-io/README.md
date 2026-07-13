# Experiment 0008: Cooked Region I/O

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md),
  [ADR 0009](../../docs/adr/0009-resident-region-storage.md),
  [ADR 0010](../../docs/adr/0010-asynchronous-region-publication.md),
  [ADR 0011](../../docs/adr/0011-cooked-region-storage.md)

## Hypothesis

A versioned, indexed cooked region pack can supply only the missing payloads for the
accepted 50-slot asynchronous resident cache from one bounded background I/O worker,
without changing immutable GPU publication semantics or stalling direct rendering
while file reads are intentionally held.

## Scope

The experiment introduces the first reusable canonical runtime format crate, a
deterministic offline region cooker, a sparse immutable pack, strict header/index/chunk
validation, one background reader, one reserved cache transaction, and typed Sidecar
controls for cooked scheduling, status, and an operator-only I/O gate.

The existing asynchronous transfer remains the only owner of cache commit, protected
slot reuse, copy queue submission, and frame-boundary publication. Its planning path is
split into reservation, payload materialization, copy submission, and publication so
generated and cooked payloads share one state machine.

The I/O gate exists only to make a pending file request observable. It is not a runtime
throttling mechanism.

Compression, decoding, memory mapping, multiple workers, multiple in-flight requests,
cancellation, priorities, prefetch, bandwidth shaping, loose files, remote I/O,
production assets, animation, meshes, materials, LOD, occlusion, ECS, and legacy game
formats are excluded.

## Cooked format

The little-endian `WLRGN001` pack has a fixed 64-byte header and a sorted, unique table
of 56-byte index entries. Each entry records region ID, record count, aligned absolute
payload offset, payload byte count, flags, and SHA-256. Payloads contain exactly 1,024
canonical 20-byte instance records. The first payload and every chunk are 4 KiB aligned.

The reader rejects unknown magic or version, non-canonical sizes, unknown flags,
unsorted or duplicate IDs, invalid alignment, overlapping/out-of-file ranges, truncated
data, checksum mismatches, and records whose embedded region ID disagrees with the
index.

The canonical pack contains the sorted union of radius-2 windows centered at `[64,64]`,
`[65,64]`, `[65,65]`, and `[96,96]`: 60 regions and 1,228,800 payload bytes. Generated
files and reports remain under ignored `out/cooked/0008-cooked-region-io/`.

## Workload

1. Cook the canonical sparse pack twice and require byte-identical file hashes.
2. Open and validate its header and complete index before starting payload requests.
3. Arm the I/O gate and schedule initial center `[64,64]` from an empty cache.
4. While the worker is provably held before payload reads, reject a second request as
   `stream_busy`, advance at least 30 direct frames, and preserve calibration output.
5. Release the gate, read exactly 25 chunks, submit them through the existing copy queue,
   publish `[64,64]`, and reproduce Experiment 0007 visual and semantic evidence.
6. Schedule adjacent `[65,64]` and `[65,65]`; each reads exactly five chunks. Return to
   cached `[64,64]` and read zero payload bytes.
7. Teleport to `[96,96]`; read 25 chunks, protect all 25 published slots, evict exactly
   ten non-active cached regions, and publish with 50 resident slots.
8. Restart through Sidecar, reopen the pack, reproduce the initial payload checksum and
   frame evidence, then disable the mode and reproduce calibration baselines.

## Controlled variables

- World dimensions, active radius, records per region, record layout, generated source
  semantics, camera, shaders, descriptor table, culling, indirect draw, object IDs, and
  visual colors remain identical to Experiment 0007.
- The offline cooker is the only writer. Runtime code opens packs read-only.
- The pack index is loaded synchronously once; region payloads are read only by the
  background worker using indexed absolute offsets.
- The request channel, completion channel, reserved cache transaction, GPU transfer,
  upload arena, and copy transaction each have capacity one.
- A cache reservation does not mutate the published or committed cache. I/O or checksum
  failure cancels it without GPU submission.
- Only after all requested chunks validate may the reservation enter the accepted copy
  queue and frame-boundary publication path.

## Metrics

- Pack version, region count, index/payload/file bytes, alignment, file SHA-256, and
  deterministic recook hash.
- Transaction ID and stage, requested IDs, retained/uploaded/evicted/protected/resident
  counts, chunk count, payload bytes, seek count, I/O duration, queue duration, checksum
  duration, copy fences, and total publication latency.
- Worker/channel/reservation/GPU in-flight capacities and observed high-water marks.
- Frame index before and during the I/O hold, published center, visual/object-ID hashes,
  GPU probes, renderer/device errors, process identity, and cleanup state.

## Acceptance criteria

- The format crate round-trips canonical records and rejects every malformed class
  listed in the format section through focused tests.
- Two independent cooker runs produce byte-identical packs and SHA-256 values.
- Runtime reports 60 indexed regions, 1,228,800 payload bytes, 4 KiB alignment, and no
  payload read during pack open.
- A gated initial request returns immediately, remains in the I/O stage with zero bytes
  read, rejects another request as `stream_busy`, and advances at least 30 direct frames
  without renderer error.
- Gate release reads 25 chunks/512,000 bytes and publishes the accepted initial frame,
  object-ID, upload checksum, and GPU workload evidence.
- Adjacent requests each read five chunks/102,400 bytes; cached revisit reads zero
  chunks/bytes; teleport reads 25 chunks/512,000 bytes and evicts exactly ten regions.
- File-derived records reproduce Experiment 0007 generated-upload checksums for matching
  transactions, proving the cooked path did not alter runtime semantics.
- I/O/checksum failure is observable, does not submit copy work, and leaves the last
  published snapshot and committed cache unchanged.
- Existing `runseal :async-region`, `runseal :resident-stream`, and `runseal :guard`
  regressions pass. Final Sidecar status contains no target or broker process.

## Environment

The final report records revision, adapter, debug-layer state, pack metadata and hashes,
I/O and queue bounds, transaction reports, frame progression, GPU probes, visual hashes,
process identities, and cleanup.

## Reproduction

Run from the repository root after implementation:

```powershell
runseal :cooked-region
```

## Results

Accepted results on 2026-07-13 using D3D12 feature level 12_1 with the debug layer on
an NVIDIA GeForce RTX 4070 Ti SUPER:

| Transaction | Chunks | Bytes | Read | Verify | Schedule return | Publication |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial `[64,64]` | 25 | 512,000 | 0.14 ms | 13.64 ms | 36.23 ms | 2,559.07 ms |
| Adjacent `[65,64]` | 5 | 102,400 | 0.03 ms | 2.52 ms | 29.28 ms | 28.23 ms |
| Adjacent `[65,65]` | 5 | 102,400 | 0.05 ms | 2.53 ms | 33.41 ms | 35.29 ms |
| Cached `[64,64]` | 0 | 0 | 0 ms | 0 ms | 25.91 ms | 36.42 ms |
| Teleport `[96,96]` | 25 | 512,000 | 0.15 ms | 13.99 ms | 41.38 ms | 33.00 ms |

Two independent cooker runs produced the same 1,232,896-byte file and SHA-256
`3a520a6de45477f850c222930ef7bc7f16644cd9a45fbcadd3610797c1550290`.
Runtime open read the 64-byte header and 3,360-byte index, reported zero payload bytes,
and exposed one worker, one request slot, one completion slot, and one transaction.

The initial I/O gate held the worker for approximately 2.53 seconds, which accounts for
the initial publication duration. The schedule call returned, a concurrent request was
rejected as `stream_busy`, and the direct frame index advanced exactly 30 observed frames
while payload bytes remained zero. Calibration color and raw object-ID hashes remained
`8f0fc6...` and `b132c8...`.

All cooked transactions reported `payloadSource = cooked-pack` and `generationMs = 0`.
Their upload hashes exactly matched Experiment 0007's generated path, including initial
`280cb2...`, adjacent X `965b15...`, adjacent Z `2bf29f...`, empty revisit `e3b0c4...`,
and teleport `9df08f...`. Initial, revisit, restart, and failure-preserved frames reproduced
color `9bd075...`, raw object IDs `8431f1...`, and diagnostic PNG `91c9fa...`.

The corrupt pack changed the file hash while preserving its valid header and index. Its
first teleport chunk failed SHA-256 after one seek and 20,480 attempted bytes. No copy
was submitted: completed fence remained 1, next fence remained 2, reservation and pending
state returned to null, published center remained `[64,64]`, and visual evidence remained
exact. Initial and teleport GPU probes retained 25 regions, 25,600 candidates, one
indirect draw, and visible counts 18,928 and 18,934. No renderer error, device removal,
or final Sidecar process remained.

## Conclusion

Accepted. A strict indexed pack and one bounded background reader feed only reserved
missing regions into the existing asynchronous GPU publication path. I/O can remain
pending without blocking direct frames, and validation failure rolls back before any
copy or cache commit.

## Promotion

ADR 0011 promotes the explicit cooked format, offline-only writing, header/index open,
on-demand chunk validation, reserve/materialize split, bounded worker channels, payload
source metrics, and pre-copy rollback. Version 1's fixed records and single-worker policy
remain experiment constraints rather than a general asset architecture.
