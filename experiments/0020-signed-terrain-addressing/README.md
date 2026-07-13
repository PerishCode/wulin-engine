# Experiment 0020: Signed Terrain Addressing

Status: Accepted

- Related ADRs: [ADR 0023](../../docs/adr/0023-signed-terrain-addressing.md)

## Hypothesis

A signed 64-bit global terrain key can own logical residency while an explicit bounded
alias maps it to the unchanged format-V1/GPU/semantic local region ID: anchors at plus
or minus 2^40 regions preserve exact global identity, adjacent movement retains 20 of
25 cache entries and uploads five, equal local aliases at different global anchors do
not false-hit, and all accepted terrain output and fixed GPU work remain unchanged.

## Scope

This experiment adds a requested global terrain window above the accepted standalone
terrain streamer. A window contains a signed global origin, signed global center,
active radius, and fixed local origin `(64,64)`. Checked integer subtraction maps each
global region to a format-V1 local region ID. The 50-slot terrain cache identifies a
global entry by its signed global key and local content binding; the pack reader and GPU
continue to receive the bounded local ID.

Legacy `terrain.schedule` remains unchanged. Global scheduling is an explicit Sidecar
control and does not replace camera traversal. The published snapshot reports both the
signed global mapping and the local active mapping so cache identity and GPU placement
cannot be confused in evidence.

This experiment does not change terrain format V1, terrain payloads, GPU descriptors,
mesh/LOD dispatch, terrain semantic IDs, object residency, atomic terrain/object
composition, camera traversal, automatic rebase, predictive prefetch, authored world
partitioning, network coordinates, collision, or navigation. The local region ID still
serves storage lookup, GPU placement, and semantic projection; separating those three
remaining roles requires a later gate.

## Workload

1. Reproduce Experiment 0013's canonical standalone terrain publication, probe, and
   capture through the unchanged legacy schedule.
2. Schedule a global window whose origin and center are `(0,0)` and whose local alias
   center is `(64,64)`. Require exact 25-entry global/local mapping evidence and the
   canonical local GPU output.
3. Schedule origins/centers `(2^40,-2^40)` and `(-2^40,2^40)`. Require distinct exact
   global hashes, identical local mapping and attachments, and 25 uploads rather than a
   false cache hit on the reused local aliases.
4. At a far origin, move the global center by one region on X and then Z. Require 20
   retained regions, five uploads, five pack reads, and bounded cache occupancy.
5. Revisit the far origin center while the 30-region union remains resident. Require 25
   retained regions, zero uploads/reads, exact logical mapping, and revisit attachments.
6. Hold terrain I/O while scheduling a neighboring global center. Require the complete
   old signed mapping and attachments to remain published until one frame-boundary
   commit exposes the complete new mapping.
7. Request a center whose local aliases are absent from the deterministic sparse pack.
   Require synchronous rejection before worker submission, no cache mutation, no
   publication, and unchanged attachments.
8. Exercise exact signed overflow/boundary rejection and a changed origin that maps to
   the same local aliases. The latter must upload all 25 regions because global keys are
   different.
9. Restart and reproduce the far-origin baseline. Then run 32 release adjacent/revisit
   cycles and report schedule, I/O, copy, publication, and combined GPU distributions.

## Controlled Variables

- Terrain payload, format version, local pack IDs, local GPU placement, semantic IDs,
  shaders, descriptors, reverse-Z, camera poses, LOD settings, attachments, and
  1280x720 extent remain unchanged.
- Global coordinates are signed 64-bit integers. Mapping subtracts global origin before
  any conversion to local `u32`; overflow and out-of-pack aliases are errors.
- Global local-origin alias is fixed at `(64,64)` for this experiment. Active radius is
  bounded by the accepted terrain capacity and remains at two in canonical evidence.
- Physical capacity remains 50 slots and active capacity remains 25. One I/O request
  and one copy transaction may be pending.
- Correctness uses the debug namespace. Timings use the release namespace with
  validation disabled and make no speedup claim.

## Metrics

- Global origin/center/radius, local alias config, exact global active mapping, global
  mapping hash, local active mapping/hash, and mismatch/duplicate counts.
- Retained, uploaded, evicted, protected, resident, and free region counts; physical
  slot assignments; requested local IDs; payload/read bytes; and cache revisit counts.
- I/O read/verify/total, schedule, copy GPU, copy-to-publication, pending, combined GPU,
  and operator-observed publication distributions.
- Color, PNG, object-ID, diagnostic, terrain payload, position, seam, LOD, semantic,
  process, validation, and device-removal evidence.

## Pass Criteria

- Anchors at zero and plus/minus 2^40 serialize exactly, produce distinct global hashes,
  and map bijectively to the expected 25 bounded local IDs with no direct global-to-f32
  or global-to-u32 conversion.
- Equal local aliases at different global anchors upload 25 regions and produce
  byte-identical local GPU probes and attachments. They never false-hit because local
  IDs happen to match.
- Adjacent movement reports exactly 20 retained and five uploaded/read regions. Revisit
  reports 25 retained, zero uploads, zero payload bytes, and no pack read.
- Held I/O and copy preserve one complete old snapshot until one immutable publication.
  Missing aliases and signed arithmetic failures reject transactionally without worker,
  cache, GPU, or attachment mutation.
- Cache residency never exceeds 50, active mapping never exceeds 25, and one pending
  request cannot allocate work proportional to global coordinate magnitude.
- Legacy Experiment 0013 and affected traversal/composition compatibility pass without
  changing format, shaders, GPU submission, semantic IDs, or canonical hashes.
- Debug/release validation, restart, Flavor, and Sidecar lifecycle pass without device
  loss, hidden fallback, unbounded growth, or residual process.

## Evidence

The canonical workflow is:

```powershell
runseal :global-terrain
```

Generated evidence remains ignored under
`out/captures/0020-signed-terrain-addressing/`.

## Results

The canonical workflow passed on 2026-07-13 in 389.1 seconds. It passed Experiment
0013 and Experiment 0018 unchanged; Experiment 0018 recursively passed its complete
0017 -> 0016 -> 0015 compatibility chain.

- The deterministic pack contained 197 local terrain regions. Legacy scheduling kept
  the canonical 25-entry active mapping and payload hashes. Zero, `(2^40,-2^40)`, and
  `(-2^40,2^40)` produced global mapping hashes
  `89bacb8c112deb1a426110bb3d33bbbb21959446e9d0ab3d7a9453ef390c6437`,
  `c094fd79322493121fcbe881565e44f1504c46390ff665a96620a023334e3b1c`,
  and `e9cc23c044c9eb4af35101a9d8d158c73ec0e2caa28ffe1dfe24101adf942bc7`.
- All three equal local aliases preserved one local probe, color hash
  `1c044011973c4df5ffc2f0f92967f8595281ecb892073d991c0f9adaa6b7d1aa`,
  PNG hash `e04905e70f2548c848a186eb82a4d961fc949754fed5d67e255e1c9f0dc2f77f`,
  object-ID hash `37e32108bf3aaba3b1e37e2488be5221c96a1a95b8359224e5b2dcb915c10aa8`,
  and diagnostic hash
  `2dace648099e6eb234d6def4fc3c5274244c69eaf32c7fd6255a43eecd112127`.
- Each one-region X or Z move retained 20 regions and read/uploaded five regions
  (`20,480` bytes). The union revisit retained 25 and read/uploaded zero. Resident
  count never exceeded 50.
- I/O hold kept generation 9 visible until generation 10 published. Copy hold kept
  generation 10 visible until generation 11 published. Their held global mappings and
  color/object-ID attachments were unchanged.
- A missing local alias was rejected as `stream_failed`; an out-of-window alias and
  exact signed subtraction overflow were rejected as `invalid_global_terrain_config`.
  No rejection changed the published mapping, cache transaction, or attachments.
- A previously unseen global origin mapped to the same local aliases with zero retained
  and 25 uploaded regions. Restart changed process identity and reproduced the exact
  far-anchor logical mapping and attachments.
- Across 32 release adjacent moves, schedule time was
  25.0263/25.5799/25.6904 ms median/P95/P99; copy GPU time was
  0.002432/0.003104/0.003296 ms; verified I/O total was
  0.0602/0.0787/0.0990 ms. Combined terrain GPU time was
  0.039936/0.048128/0.050176 ms.
- Across 32 release resident revisits, payload/read/verify bytes and times were zero.
  Schedule time was 24.8868/33.5640/33.7484 ms. Operator-observed adjacent publication
  was 123.3016/135.4659/136.5502 ms.

These are laboratory control-path distributions and include Sidecar/frame-boundary
observation where stated. They are not a normal-frame speedup claim.

## Conclusion

The hypothesis passes. Signed global region identity can own bounded terrain residency
while the accepted format-V1 reader, GPU placement, semantic IDs, payloads, and fixed
submission continue to use an explicit local alias. Integer mapping remains exact at
plus or minus 2^40 regions, cache overlap follows spatial identity, and equal local IDs
cannot create a false hit at another global anchor.

The experiment accepts a terrain-only signed cache identity and local content-binding
adapter. It does not accept signed object residency, global atomic composition,
automatic traversal/rebase, a new world format, or direct global coordinates on GPU.

## Promotion

Promote `GlobalTerrainConfig`, checked signed-to-local window mapping, composite terrain
cache identity, typed Sidecar scheduling, and global/local probe evidence as the input
boundary for the next signed atomic composition experiment. Preserve format V1 and
local GPU/semantic contracts until that experiment replaces them explicitly.
