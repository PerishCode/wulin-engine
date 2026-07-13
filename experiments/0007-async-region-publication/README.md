# Experiment 0007: Asynchronous Region Publication

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md),
  [ADR 0009](../../docs/adr/0009-resident-region-storage.md),
  [ADR 0010](../../docs/adr/0010-asynchronous-region-publication.md)

## Hypothesis

The accepted resident-region model can publish deterministic region payloads through a
dedicated D3D12 copy queue without CPU fence waits or ordinary-frame stalls, while the
direct queue continues rendering one immutable published snapshot and physical storage,
in-flight transactions, staging memory, and eviction remain explicitly bounded.

## Scope

The experiment adds a separate asynchronous resident mode, 50 individually state-owned
default-heap region slots, a shader-visible SRV descriptor table, a dedicated copy queue
and fence, one bounded persistent upload arena, one in-flight transaction, direct-queue
release barriers for reused non-active slots, copy-fence polling, and frame-boundary
publication of 25 active physical slot indices through root constants.

A deterministic operator-only copy gate may hold the copy queue behind an unsignaled
fence. It exists to make the pending state observable and to prove that direct rendering
continues while transfer completion is impossible. It is not a runtime scheduling
feature.

The experiment retains Experiment 0006's deterministic in-memory record generator. It
excludes file/network I/O, cooked asset formats, decoding, compression, background CPU
workers, multiple concurrent requests, cancellation, priorities, bandwidth throttling,
animation, meshes, materials, LOD, occlusion, sparse resources, and ECS.

## Workload

The world remains 128x128 regions with 1,024 20-byte records per region. The active
radius remains 2, producing 25 active regions and 25,600 candidates. Physical capacity
is 50 regions: the smallest capacity that can preserve any 25-region published snapshot
while a completely disjoint 25-region snapshot uploads.

The canonical sequence is:

1. Schedule center `[64,64]`, poll without blocking until published, and reproduce the
   Experiment 0006 resident frame and perception evidence.
2. Arm the deterministic copy gate and schedule adjacent center `[65,64]`.
3. While the copy fence is provably incomplete, reject a second request as busy, observe
   at least 30 additional presented frames, and reproduce the old `[64,64]` snapshot.
4. Release the gate, poll until `[65,64]` publishes, follow the camera, and probe the new
   snapshot.
5. Publish `[65,65]`, then return to cached `[64,64]` with zero instance bytes copied.
6. Teleport to `[96,96]`: preserve all 25 currently published slots until publication,
   upload 25 regions, evict exactly 10 non-active cached regions, and fill 50 slots.
7. Restart through Sidecar, reproduce the initial generated-data checksum and resident
   visual/semantic evidence, then disable the mode and reproduce calibration baselines.

Artifacts overwrite the ignored
`out/captures/0007-async-region-publication/` collection.

## Controlled variables

- Region dimensions, record layout, generator, camera, culling, indirect draw, colors,
  and object-ID semantics remain identical to Experiment 0006.
- The synchronous 49-slot resident mode remains available as an unchanged regression.
- Each physical region is a separate default-heap resource so copy-destination state is
  never applied to a resource referenced by the published snapshot.
- The active mapping is 25 physical slot indices in root constants. Region identity
  remains in each persistent instance record.
- The copy queue waits for a direct-queue release fence only when reused non-active slots
  require `NON_PIXEL_SHADER_RESOURCE -> COPY_DEST` transitions. The CPU does not wait.
- Publication occurs only after CPU polling observes the copy fence complete. The direct
  queue then transitions uploaded slots to shader-resource state before first use.
- At most one stream transaction exists. Upload arena and command allocator reuse occur
  only after its copy fence completes.
- Initial and teleport transfers are 512,000 instance bytes; adjacent movement transfers
  102,400 bytes; a cached revisit transfers zero bytes.

## Metrics

- Transaction ID and stage, requested and published centers, gate state, direct release
  fence, copy fence, and observed completion values.
- Retained, uploaded, evicted, protected, resident, and free region counts.
- Generated and copied instance bytes, upload checksum, generation time, schedule-return
  time, pending duration, and publication latency.
- Frame index at schedule, while held, and after publication; old-snapshot color and raw
  object-ID hashes while copy completion is impossible.
- GPU visible count, dispatch, indirect draw count, and timing distributions after each
  publication.
- Renderer/device errors, process identity, restart hashes, and final Sidecar process
  counts.

## Acceptance criteria

- Region payloads are 50 separate default-heap resources addressed through a bounded
  descriptor table; upload memory and in-flight transaction count are fixed.
- A held adjacent request returns successfully while its copy fence is incomplete. Its
  schedule call does not execute a CPU fence wait.
- While held, the published center remains `[64,64]`, the native frame index advances by
  at least 30, and color/object-ID evidence remains exactly the initial snapshot.
- A second request while held returns `stream_busy` without modifying planned or
  published cache state.
- Releasing the gate publishes `[65,64]` without renderer error or device removal.
- Adjacent publications retain 20, upload 5, evict 0, and copy 102,400 instance bytes.
- Cached `[64,64]` publication retains 25, uploads 0, evicts 0, and copies zero bytes.
- Teleporting to `[96,96]` uploads 25, evicts exactly 10, reports 50 resident regions,
  and never chooses one of the 25 currently published slots for reuse.
- Every published load snapshot reports 25 active regions, 25,600 candidates, dispatch
  `[25,4,1]`, one indirect draw, and `0 < visible < 25,600`.
- Initial, cached revisit, and restart evidence reproduce exactly; restart reproduces
  the initial generated-upload checksum.
- Disabling asynchronous resident mode reproduces accepted calibration color and raw
  object-ID hashes.
- The existing `runseal :resident-stream` regression and repository guard both pass.
- Final Sidecar status contains no target or broker process.

## Environment

The final report records revision, adapter, debug-layer state, queue/fence progression,
resource and staging bounds, transaction reports, frame progression, GPU probes, hashes,
process identities, and cleanup.

## Reproduction

Run from the repository root:

```powershell
runseal :async-region
```

## Results

Accepted results on 2026-07-12 using D3D12 feature level 12_1 with the debug layer on
an NVIDIA GeForce RTX 4070 Ti SUPER:

| Transaction | Retained | Uploaded | Evicted | Resident | Bytes | Schedule | Publication |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial `[64,64]` | 0 | 25 | 0 | 25 | 512,000 | 9.52 ms | 40.07 ms |
| Held adjacent `[65,64]` | 20 | 5 | 0 | 30 | 102,400 | 3.27 ms | 3,025.91 ms |
| Adjacent `[65,65]` | 20 | 5 | 0 | 35 | 102,400 | 1.97 ms | 41.41 ms |
| Cached `[64,64]` | 25 | 0 | 0 | 35 | 0 | 0.24 ms | 33.21 ms |
| Teleport `[96,96]` | 0 | 25 | 10 | 50 | 512,000 | 9.43 ms | 42.02 ms |

The held adjacent schedule returned with copy fence 2 incomplete behind gate fence 1.
A concurrent request returned `stream_busy`. Before release, 32 additional frames were
presented while the published center remained `[64,64]`; color, raw object-ID, diagnostic
PNG, and semantic sample evidence all reproduced exactly. The 3-second publication time
is the intentional gate hold, not a CPU wait in the schedule call.

Every non-initial transaction protected all 25 published slots. The teleport reused only
10 non-active cache slots and reached the fixed 50-slot capacity. Cached publication
copied zero instance bytes. Initial and restart uploads reproduced SHA-256
`280cb2eea7fc3e23743c6bd74f9b986ceaf00cb742a3b8214f130c6f9ea501f2`.
The 50 payloads and fixed upload arena each contain 1,024,000 bytes. Individually
committed default-heap resources consume 3,276,800 bytes after 64 KiB alignment, making
the state-ownership tradeoff explicit rather than hiding allocation overhead.

Each published state recorded 16 probes of 64 GPU iterations. Visible counts were
18,928, 18,928, 18,932, 18,928, 18,934, and 18,928 for initial, adjacent X, adjacent Z,
revisit, teleport, and restart. Median total GPU time ranged from approximately 0.05 to
0.10 ms. All probes retained 25 regions, 25,600 candidates, dispatch `[25,4,1]`, and one
indirect draw.

Initial, held, cached-revisit, and restart frames reproduced color SHA-256
`9bd075106177...`, raw object-ID SHA-256 `8431f1c795ec...`, and diagnostic PNG SHA-256
`91c9fad2b269...`, with no unknown IDs. Disabling the mode reproduced calibration color
`8f0fc6e9a49b...` and raw ID `b132c850f029...`. No renderer error, device removal, or
final Sidecar process remained.

## Conclusion

Accepted. Dedicated copy-queue transfer and frame-boundary polling preserve an immutable
published snapshot without CPU fence waits or direct-frame stalls. A 50-slot cache is
the minimum safe physical bound for overlapping arbitrary old and requested 25-region
windows, and explicit single-transaction backpressure prevents hidden growth.

## Promotion

ADR 0010 promotes immutable asynchronous publication, per-slot resource-state ownership,
double-active-set capacity, direct-to-copy fence ordering, frame-boundary completion
polling, and bounded backpressure. Deterministic in-memory generation and the one-slot
scheduler remain experiment constraints. The next streaming experiment may introduce a
real cooked format and background I/O without changing these queue/publication rules.
