# Experiment 0006: Resident Region Streaming

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md),
  [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md),
  [ADR 0009](../../docs/adr/0009-resident-region-storage.md)

## Hypothesis

The accepted region-addressed GPU work model can consume persistent instance records
from a bounded default-heap region cache such that adjacent active-window movement uploads
only newly entered regions, revisiting cached regions uploads no instance data, distant
movement performs deterministic bounded eviction, and ordinary render/probe frames
perform no upload work.

## Scope

The experiment adds a separate resident load mode, a 49-slot GPU region cache,
20-byte persistent instance records, a 25-entry active-region mapping, deterministic CPU
test-data generation, LRU retention/eviction, explicit default-heap copies, resident
compute compaction and indirect rendering, upload reports, Sidecar stream controls, and a
Runseal movement/restart workload.

It excludes file/network I/O, asset decoding, compression, asynchronous copy queues,
upload rings shared with shipping frames, sparse resources, background streaming,
multiple instance archetypes, animation, LOD, occlusion, memory defragmentation, and ECS.

## Workload

The configured world remains the Experiment 0005 maximum 128x128 region address space.
Each region contains 1,024 persistent upright-quad records. The camera follows the active
center with fixed offset `[0, 30, 30]`, targets the center at ground level, and retains a
60-degree vertical FOV. Returning to `[64, 64]` therefore restores the canonical load
camera `[0, 30, 30]` targeting `[0, 0, 0]`.

The canonical sequence is:

1. Configure resident mode at center `[64, 64]`, radius 2: upload 25 regions.
2. Move to `[65, 64]`: retain 20 active regions and upload 5 new regions.
3. Move to `[65, 65]`: retain 20 active regions and upload 5 new regions.
4. Return to `[64, 64]`: all active regions are cached; upload no instance records.
5. Probe and capture without movement: perform no upload work.
6. Teleport to `[96, 96]`: upload 25 regions and evict exactly 11 to stay within 49
   resident slots.
7. Restart through Sidecar, configure `[64, 64]` again, and reproduce the initial
   generated-data checksum plus resident color/object-ID evidence.
8. Disable resident mode and reproduce calibration color and object-ID baselines.

Artifacts and the final report overwrite the ignored
`out/captures/0006-resident-region-streaming/` collection.

## Controlled variables

- Maximum world address is 128x128 regions; region dimensions remain 16x16 meters.
- Active radius is 2: 25 active regions and 25,600 candidate instances.
- Cache capacity is exactly 49 regions and 50,176 instance records.
- One instance record is 20 bytes: position `float3`, height `float`, and global region
  ID `uint`.
- One region occupies 20,480 instance bytes. The active mapping contains 25 records of
  8 bytes each and occupies 200 bytes.
- Initial instance upload is 512,000 bytes plus the 200-byte active mapping.
- An adjacent move uploads 102,400 instance bytes plus the active mapping.
- A fully cached revisit uploads only the 200-byte active mapping.
- Uploads are explicit synchronous operator transactions on the direct queue. The
  experiment records this limitation rather than treating it as production streaming.
- Resident data is read as shader resources by the accepted GPU compaction and indirect
  draw path. The CPU never builds a visible list or draw argument.
- Region proxy IDs retain the Experiment 0005 procedural semantic range.

## Metrics

- Requested center, active region count, retained, newly uploaded, evicted, resident, and
  free region counts for every stream transaction.
- Instance bytes, mapping bytes, and total bytes copied per transaction.
- Deterministic SHA-256 of generated records uploaded by each transaction and of the
  initial active resident dataset.
- CPU generation, command recording, queue submit/wait, and total transaction duration.
- GPU visible count, compaction/draw/total distributions, indirect draw count, and
  dispatch shape after streaming.
- Color, raw object-ID, diagnostic PNG, region semantic samples, process identity,
  renderer errors, device removal, and final Sidecar process counts.

## Acceptance criteria

- Resident buffers use default-heap resources; upload resources are staging inputs and
  are not used as per-frame shader data.
- Initial configuration reports 25 uploaded, 0 retained, 0 evicted, 25 resident regions,
  512,000 instance bytes, 200 mapping bytes, and 512,200 total bytes.
- Each adjacent move reports 20 retained, 5 uploaded, 0 evicted, and 102,600 total bytes.
- Returning to `[64, 64]` reports 25 retained, 0 uploaded, 0 evicted, and exactly 200
  mapping bytes.
- An unchanged stream request and ordinary probes/captures report zero instance bytes;
  render frames never regenerate or upload instance records implicitly.
- Teleporting to `[96, 96]` reports 25 uploaded, 11 evicted, 49 resident regions, and
  512,200 total bytes; cache capacity is never exceeded.
- Active candidate count, dispatch `[25, 4, 1]`, one indirect draw, and
  `0 < visible < 25,600` remain valid after every movement.
- Returning to the initial cached window reproduces its color, raw object-ID, and
  diagnostic PNG hashes before restart.
- After restart, initial generated-data checksum and resident color/object-ID evidence
  reproduce exactly.
- Region perception reports no unknown IDs and a fixed visible sample resolves through
  the procedural region registry.
- Disabling resident mode reproduces Experiment 0003 color and Experiment 0004 raw
  object-ID hashes.
- Every transaction, probe, and manifest reports no device removal or renderer error.
- Final Sidecar status contains no target or broker process and `runseal :guard` passes.

## Environment

The final report records the actual revision, adapter, cache layout, transfer volumes,
checksums, timings, GPU probe evidence, frame hashes, process identities, and cleanup.

## Reproduction

Run from the repository root:

```powershell
runseal :resident-stream
```

## Results

Accepted results on 2026-07-12 using D3D12 feature level 12_1 with the debug layer on
an NVIDIA GeForce RTX 4070 Ti SUPER:

| Transaction | Retained | Uploaded | Evicted | Resident | Instance bytes | Total bytes | Transaction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial `[64,64]` | 0 | 25 | 0 | 25 | 512,000 | 512,200 | 10.150 ms |
| Adjacent `[65,64]` | 20 | 5 | 0 | 30 | 102,400 | 102,600 | 4.022 ms |
| Adjacent `[65,65]` | 20 | 5 | 0 | 35 | 102,400 | 102,600 | 3.939 ms |
| Revisit `[64,64]` | 25 | 0 | 0 | 35 | 0 | 200 | 0.801 ms |
| Unchanged `[64,64]` | 25 | 0 | 0 | 35 | 0 | 200 | 2.205 ms |
| Teleport `[96,96]` | 0 | 25 | 11 | 49 | 512,000 | 512,200 | 10.527 ms |

The initial and restarted uploads produced the same SHA-256
`280cb2eea7fc3e23743c6bd74f9b986ceaf00cb742a3b8214f130c6f9ea501f2`.
Cached and unchanged requests produced the empty instance-upload hash and copied only
the 200-byte active mapping. Cache capacity remained 49 after the teleport.

Each state recorded 16 probes of 64 GPU iterations. Median GPU compaction was
0.00266-0.00352 ms and median total GPU time was 0.0591-0.0895 ms. Every probe retained
25 active regions, 25,600 candidates, dispatch `[25, 4, 1]`, and one indirect draw.
Visible counts were 18,928, 18,928, 18,932, 18,928, 18,934, and 18,928 for initial,
the two adjacent moves, revisit, teleport, and restart respectively; each state was
internally stable.

Initial, cached-revisit, and restart frames reproduced color SHA-256
`9bd075106177...`, raw object-ID SHA-256 `8431f1c795ec...`, and diagnostic PNG SHA-256
`91c9fad2b269...`. Sample `[600,600]` resolved to `load.region.064.065` ID `73921`, no
unknown IDs were reported, and color/ID coverage matched. Disabling resident mode
reproduced calibration color `8f0fc6e9a49b...` and raw ID `b132c850f029...`.

All transactions, probes, and captures completed without renderer error or device
removal. Sidecar restart changed the native process identity and final cleanup left no
target or broker process.

## Conclusion

Accepted. A bounded default-heap region cache supplies the existing GPU compaction path
without per-frame instance generation or upload. Transfer volume follows newly entered
regions, a cached revisit costs only the active mapping, and deterministic eviction caps
resident storage through distant movement.

## Promotion

ADR 0009 promotes bounded resident region storage, active-to-physical slot indirection,
transactional cache publication, and explicit upload accounting. Synchronous direct-
queue waiting and synthetic CPU generation remain experiment constraints. A later
streaming stage may introduce real cooked data and asynchronous copy while preserving
these accepted invariants.
