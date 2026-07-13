# Experiment 0012: GPU Conservative Occlusion

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md),
  [ADR 0013](../../docs/adr/0013-gpu-skeletal-crowd-execution.md),
  [ADR 0014](../../docs/adr/0014-gpu-surface-visibility-resolve.md)

## Hypothesis

The accepted deterministic visibility winner can become a bounded reverse-Z hierarchy
for the next compatible frame. A fixed-shape GPU classify, prefix, and stable-scatter
pipeline can conservatively remove fully hidden skeletal objects before mesh execution
without CPU-visible object, depth, or indirect-count enumeration. Compatible frames
must reproduce the exact unculled visibility, color, and semantic-ID attachments while
submitting measurably less meshlet, vertex, triangle, and skin-influence work.

## Scope

The experiment extends only the accepted surface mode. The original skeletal cull,
animation classification, pose compaction, palette evaluation, and CPU oracle remain the
source workload. A new bounded filtered-visible list and indirect argument are produced
after pose evaluation and before visibility mesh execution.

The accepted `R32G32_UINT` rasterizer-ordered winner is reduced into a full fixed-size
`R32_UINT` mip chain. Reverse-Z uses minimum reduction so every hierarchy value is the
farthest depth in its source footprint; background zero therefore prevents unsafe
culling. Object queries project a fixture AABB proven to contain all generated static
and animated vertices, expand its screen rectangle conservatively, choose a mip whose
footprint covers the rectangle, and reject only when the AABB's nearest possible depth
is strictly behind all covered hierarchy samples plus the registered bias.

History is usable only when the complete GPU execution signature that affects position,
LOD, residency, animation, and camera projection is bit-identical to the frame that
built it. First use, explicit reset, camera changes, animation changes, LOD changes,
resident publication changes, region movement, resize, mode disable, and process restart
must bypass occlusion. A bypass frame still rebuilds history. No reprojection,
speculative temporal tolerance, CPU visibility list, or fallback draw is permitted.

General occluder selection, two-phase current-frame occlusion, meshlet-level bounds,
dynamic object motion, camera reprojection, depth prepasses, terrain, portals, software
rasterization, ray queries, authored bounds, and broad GPU compatibility are excluded.

## Workload

1. Preserve the canonical cooked radius-2 snapshot at `[64,64]`, 100-percent animated
   shared poses, 64 bones, 64 phases, 64 materials, mip 0, and automatic geometry LOD.
2. Run surface visibility with occlusion disabled and capture the canonical attachment,
   semantic, sample, skeletal-counter, and color evidence.
3. Build the complete reverse-Z hierarchy from the deterministic winner and validate
   every mip dimension and reduction value against a CPU reconstruction.
4. Enable occlusion on an exactly compatible frame. Dispatch exactly 100 classification
   groups, one 128-thread prefix group, and 100 stable-scatter groups. Consume the GPU
   source count, preserve the source survivor order in a bounded filtered list, and issue
   one indirect visibility mesh dispatch from the filtered count.
5. Compare the culled frame byte-for-byte with the unculled baseline while recording
   rejected candidates and eliminated meshlet, vertex, triangle, and skin-influence
   counts. Resolve remains exactly 14,400 groups over 921,600 pixels.
6. Sweep active radii `0`, `1`, and `2`; forced LOD `0`, `1`, and `2`; animation time
   `0`, `11`, and `0`; shared and fully unique poses; and canonical camera plus one
   registered high-occlusion camera.
7. Explicitly reset history, change camera, change animation time, move to `[65,64]`,
   revisit `[64,64]`, and restart through Sidecar. Validate one bypass frame followed by
   one compatible queried frame for each transition.
8. Measure optimized disabled, history-build/bypass, and compatible-query distributions
   with preheat, per-workload warm-up, and 32 samples.

## Controlled variables

- Adapter, dimensions, reverse-Z projection, camera definitions, cooked bytes, resident
  publication, meshlet/animation/surface catalogs, semantic IDs, winner ordering,
  material resolve, and generated geometry remain fixed.
- The hierarchy has the complete D3D12 mip chain for 1280x720 `R32_UINT`, uses exact
  integer depth bits, and reduces each valid 2x2 source footprint with `min`.
- The fixture object bound and screen expansion are constants shared by HLSL and the CPU
  oracle. Acceptance requires an exhaustive generated-catalog proof that every static
  and swept animated vertex lies inside the registered object-space bound.
- Occlusion compaction order may vary. Candidate identity, candidate-to-filtered mapping,
  deterministic fragment winner, and final attachments must not depend on append order.
- The history signature is the complete skeletal GPU constant image. Material count,
  material mip, and background color do not affect geometric compatibility.
- The hierarchy build has one fixed mip-0 dispatch and one fixed dispatch for every
  remaining mip. Classification and stable scatter always use 100 groups and prefix
  always uses one group. No dispatch is sized from a CPU readback.
- Correctness uses the debug-layer Sidecar manifest. Timing uses the release benchmark
  manifest, explicit preheat, per-workload warm-up, and 32 samples.

## Metrics

- Hierarchy format, dimensions, mip count, byte count, per-mip dimensions, complete-byte
  hash, reduction mismatch count, build dispatch count, and build time.
- History enabled, valid, queried, bypass reason, current and producer signatures,
  explicit reset count, and compatible-frame count.
- Source visible, survivor, occluded, tested, bypassed, overflow, and candidate-mask
  counts with exact GPU/CPU oracle agreement.
- Source and submitted LOD, meshlet, vertex, triangle, animated, and skin-influence
  counts; absolute and percentage work eliminated.
- Fixed classification, prefix, scatter, indirect visibility, hierarchy, and resolve
  dispatches; source/filtered order hashes; stable-compaction mismatch count; descriptor
  count; capacity high-water marks; and total execution bytes.
- GPU cull/classify, pose, occlusion query, visibility raster, resolve, hierarchy build,
  and total P50/P95/P99 times from optimized execution.
- Raw visibility, resolved color, raw object-ID, diagnostic, hierarchy, candidate-mask,
  and sample hashes across disabled, compatible, invalidated, revisit, and restart runs.
- Renderer errors, device removal, debug messages, process identities, and final Sidecar
  namespace cleanup.

## Acceptance criteria

- The complete hierarchy exactly matches CPU min reduction for every texel of every mip,
  has a deterministic hash, contains no unbounded resource or descriptor allocation,
  and submits a fixed number of build dispatches.
- The generated bound validator proves containment for every mesh catalog LOD and every
  registered skeletal animation configuration before GPU execution is accepted.
- Compatible queried frames match the disabled baseline exactly for raw visibility,
  resolved color, raw object ID, diagnostic color, semantic joins, six surface samples,
  and all original skeletal/pose counters.
- Canonical and high-occlusion workloads reject at least one object and eliminate a
  registered non-zero number of meshlets, vertices, triangles, and skin influences.
  The high-occlusion camera must eliminate at least 25 percent of source meshlets without
  changing any final attachment.
- GPU survivor/occluded counts, eliminated geometry aggregates, and candidate mask match
  the CPU oracle exactly. Survivor plus occluded equals source visible; overflow and
  invalid-query counts are zero.
- First use and every registered incompatible transition bypass all source visible
  objects, reports the exact invalidation reason, reproduces the uncullled frame, and
  builds history. The immediately following compatible frame queries history and passes
  the exact-output and oracle gates.
- Occlusion classification and stable scatter always dispatch 100 groups, prefix always
  dispatches one group, visibility remains one indirect mesh dispatch, hierarchy build
  count is mip-count fixed, and resolve remains one 14,400 group dispatch. Every GPU
  filtered record must equal the corresponding survivor subsequence of that frame's
  source list. CPU does not read source counts or hierarchy before submission.
- Debug correctness and optimized timing report no validation error, device removal,
  hidden fallback, overflow, unbounded growth, or residual Sidecar process.
- `runseal :surface-resolve`, `runseal :skeletal-crowds`, `runseal :meshlet-scene`,
  `runseal :cooked-region`, `runseal :async-region`, and `runseal :guard` pass.

## Environment

The final report records revision and dirty state, Windows build, adapter and driver,
feature level, shader model, Agility SDK, DXC, Rust toolchain, debug-layer state, accepted
catalog hashes, hierarchy and filtered-list resources, registered cameras and bounds,
all sweep evidence, optimized distributions, process identities, and cleanup.

## Reproduction

```powershell
runseal :occlusion
```

## Results

The canonical workflow passed in both Sidecar namespaces. Correctness used the D3D12
debug layer; optimized measurement explicitly disabled it. The hierarchy is an 11-mip
1280x720 `R32_UINT` chain containing 1,228,763 texels and 4,915,052 bytes. Every texel
of every mip matched CPU reverse-Z minimum reduction. The complete occlusion execution
allocation is 5,632,332 bytes and total bounded surface execution is 25,655,340 bytes.

The fixture-bound gate evaluated 7,667,712 vertex poses across all clips, phases,
16/32/64/128-bone settings, both registered heights, every generated vertex, and the
unique-pose margin. The affine radial bound retained 0.019259255 minimum slack; the
0.25 vertical pad contained both measured extremes.

At the canonical camera, 18,928 source objects produced 69,270 meshlets, 2,399,960
vertices, 3,316,944 triangles, and 9,599,840 skin influences. Conservative query
removed three fully hidden objects: 9 meshlets, 258 vertices, 360 triangles, and 1,032
skin influences. At the registered high-occlusion camera, 10,248 of 12,958 source
objects were removed. Meshlet submission fell from 53,270 to 13,362, a 74.916-percent
reduction; 1,449,496 vertices, 1,999,536 triangles, and 5,797,984 skin influences were
also eliminated.

GPU aggregate counters, the complete 25,600-entry candidate mask, and all eliminated
geometry totals exactly matched the CPU oracle. Every filtered 24-byte record matched
the survivor subsequence of that frame's source-visible list. Classification, prefix,
and stable scatter used fixed 100/1/100 group submissions, visibility remained one
indirect mesh dispatch, hierarchy construction used 11 direct dispatches, and resolve
remained 14,400 groups.

Disabled and queried frames produced byte-identical visibility, resolved color, raw
object-ID, and diagnostic attachments at both registered cameras. Canonical visibility
remained
`a58e18f0d7c304540a1cc459749fafbbf8d1cb6d6efb98d887347ddcc04d1965`;
canonical hierarchy was
`87250f1c1246d1d6f4bd81d5cb38eab4606d7111c2042a1cf9e4b9da13279922`.
High-camera hierarchy was
`b533aa4c247f00f2d65b70af98c28168acc26b7779217dd6ac6fdb0e79ea4dc9`.

First use, explicit reset, camera and animation changes, movement, cached revisit, and
Sidecar restart each produced the registered full bypass before the next compatible
query. Radius 0/1/2, forced LOD 0/1/2, shared and fully unique 128-bone poses, and time
0/11/0 retained exact outputs, oracle agreement, bounded storage, and zero overflow.

The optimized run measured classify/prefix/scatter P50 at 0.046080 ms for canonical and
0.071680 ms for the high-occlusion workload. Hierarchy construction P50 was 0.171008 ms
and 0.302080 ms. Total deterministic ROV-path time did not consistently track eliminated
geometry: canonical disabled/queried P50 was 3.106816/9.528320 ms and high-camera P50 was
8.213504/6.546432 ms, with wide distributions. Repeated implementations and SRV/UAV
read-path checks localized this to the accepted same-pixel ROV validation raster's
sensitivity to mesh/raster scheduling; it is not evidence of a general occlusion speedup
or slowdown. Only the exact submitted-work reduction and bounded query/build costs are
promoted. A production raster/shading performance claim requires a later workload that
does not use the deterministic ROV oracle path as its performance surface.

The first implementation used one global atomic survivor append. It passed correctness
but destroyed source locality, so acceptance replaced it with fixed-shape stable
compaction and added exact order readback. A fixed 0.5 m XZ bound failed the exhaustive
pose proof; a fixed worst-case bound then erased canonical query usefulness. The accepted
height-affine bound retains proof slack without hiding those failures.

All required Experiments 0007-0011 and `runseal :guard` passed with no validation error,
device removal, hidden fallback, overflow, unbounded growth, or residual Sidecar process.

## Conclusion

Accepted. A prior compatible deterministic winner can drive conservative, bounded GPU
occlusion with exact history invalidation, fixed CPU submission, stable GPU compaction,
and exact elimination of meshlet and skinning work. Current-frame reprojection and total
ROV raster speed are not accepted by this result.

## Promotion

[ADR 0015](../../docs/adr/0015-gpu-conservative-occlusion.md) promotes bounded reverse-Z
hierarchy construction, exact execution-signature invalidation, conservative object
query, stable filtered submission, and fixed indirect visibility ownership. Fixture
bounds, cameras, generated crowds, temporal history without reprojection, and ROV-path
timing remain experiment inputs.
