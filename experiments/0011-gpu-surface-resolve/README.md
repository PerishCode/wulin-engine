# Experiment 0011: GPU Surface Visibility Resolve

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-13
- Related ADRs: [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md),
  [ADR 0012](../../docs/adr/0012-gpu-meshlet-scene-execution.md),
  [ADR 0013](../../docs/adr/0013-gpu-skeletal-crowd-execution.md),
  [ADR 0014](../../docs/adr/0014-gpu-surface-visibility-resolve.md)

## Hypothesis

A compact GPU visibility buffer can decouple accepted meshlet and skeletal visibility
from material shading. One fixed screen-space compute dispatch can reconstruct source
vertices, skinned surface attributes, deterministic material texture values, and final
color without CPU-visible object, primitive, pose, bone, or pixel enumeration. Geometry
and pose workloads may change while shade submission and maximum pixel work remain fixed.

## Scope

The experiment extends the accepted skeletal crowd path with a deterministic surface
catalog, an `R32G32_UINT` visibility target, barycentric raster output, a generated
64-layer material texture array, and an `R8G8B8A8_UNORM` compute-resolved color target.

Visibility word 0 stores deterministic `candidateIndex + 1` in its low 15 bits and a
global expanded surface-primitive index in its next 16 bits; bit 31 is reserved and zero.
Zero identifies background. Candidate identity is active-region ordinal times 1,024 plus
local instance index and is independent of atomic visible compaction. Word 1 stores the
first two `SV_Barycentrics` components as UNORM16 values; the third is reconstructed.
The surface catalog maps every accepted catalog primitive to three global vertex indices
and supplies one octahedral normal and UV pair per static catalog vertex. All payload
bounds are validated before GPU upload.

The existing `R32_UINT` semantic object-ID target remains independent and unchanged. The
accepted region pack, meshlet topology catalog, animation catalog, culling, LOD, shared
and unique pose paths, reverse-Z depth, copy queue, publication, capture, perception, and
Sidecar lifecycle remain regression contracts.

Generated fixtures, point-sampled explicit material mips, one directional laboratory
light, and deterministic stable-key material assignment are experiment inputs. General
materials, authored textures, filtering policy, tangent space, normal maps, PBR, HDR,
transparency, alpha testing, shadows, clustered lighting, post-processing, virtual
textures, mesh streaming, asset import, and compatibility fallbacks are excluded.

## Workload

1. Build a surface catalog from the immutable meshlet catalog without changing the
   accepted `WLMSH001` bytes or SHA-256. Require fewer than 65,536 expanded primitives
   and exactly one surface attribute per catalog vertex.
2. Build 64 deterministic materials and a 64-layer, 64x64 RGBA8 texture array with seven
   complete generated mip levels and a complete-byte catalog hash.
3. Publish the canonical cooked radius-2 snapshot at `[64,64]`, use 100-percent animated
   shared poses with 64 bones and 64 phases, and preserve Experiment 0010's visible set.
4. Execute the accepted reset, cull/classify, pose compact, indirect pose, and indirect
   mesh stages. The mesh/pixel stages write visibility, reverse-Z, and semantic IDs but
   do not evaluate a material color.
5. Issue one direct 8x8 compute shade dispatch over the fixed 1280x720 target. Reconstruct
   primitive vertices and barycentric attributes, apply the accepted skin palette where
   required, resolve stable-key material and explicit mip selection, and write every
   output pixel including deterministic background. A bounded candidate-to-visible map
   recovers the compact record and unique pose slot without changing payload identity.
6. Copy the resolved color to the swap-chain buffer, capture color/object-ID/visibility
   evidence, and compare selected GPU shade samples against a deterministic CPU oracle.
7. Sweep material counts `1`, `8`, and `64`; forced geometry LOD `0`, `1`, and `2`; active
   radii `0`, `1`, and `2`; shared and fully unique poses; and material mip levels `0`,
   `3`, and `6`.
8. Set animation time `0`, `11`, and `0`, move to `[65,64]`, revisit `[64,64]`, restart
   through Sidecar, and reproduce exact canonical artifacts.

## Controlled variables

- Reference adapter, capture dimensions, camera, swap chain, reverse-Z, region records,
  active mapping, cooked bytes, static meshlet topology, animation bytes, object IDs,
  animation tick, LOD thresholds, and Sidecar versions remain fixed.
- The surface catalog has fixed vertex and primitive order, octahedral encoding, UV
  convention, material records, texture bytes, mip offsets, and complete-byte hash.
- Visibility compaction order may vary. Candidate-addressed payloads and the bounded
  candidate-to-visible map ensure final visibility words, color pixels, object IDs,
  counters, material observations, and sampled shade results do not depend on it.
- Barycentric quantization is round-to-nearest UNORM16. CPU validation decodes the exact
  stored payload rather than comparing against unquantized raster coordinates.
- Equal-depth fragments select one deterministic reverse-Z and identity winner through
  a same-pixel rasterizer-ordered target before visibility, depth, and semantic ID are
  committed. Raster arrival order is not part of the output contract.
- Wrapped UV values at the exact upper boundary canonicalize to zero before explicit
  point sampling in both the GPU resolve and CPU oracle.
- The shade dispatch is always `ceil(1280/8) * ceil(720/8) = 14,400` thread groups and
  covers exactly 921,600 pixels, independent of geometry, pose, material, or occupancy.
- Correctness uses the debug-layer Sidecar manifest. Timing uses the accepted release
  benchmark manifest, explicit preheat, per-workload warm-up, and 32 samples.

## Metrics

- Barycentrics and rasterizer-ordered-view capability; visibility and color format
  support; catalog hashes and bytes;
  vertex, primitive, candidate-map, material, texture-layer, mip, descriptor, and
  fixed-capacity counts.
- Accepted candidate, visible, rejected, animated, pose, bone, LOD, meshlet, vertex,
  triangle, and influence counters with exact GPU/oracle agreement.
- Visibility written/background pixel counts, visibility payload hash, shaded pixel count,
  material-layer masks, shade dispatch groups, and all capacity high-water marks.
- Fixed reset, cull, compact, indirect-pose, indirect-visibility-mesh, direct-shade, and
  resolved-color copy submission counts.
- GPU cull/classify, pose compact/evaluate, visibility raster, surface resolve, copy, and
  total P50/P95/P99 times from optimized execution.
- Sample coordinates, packed visibility, stable object and primitive identity, decoded
  barycentrics, material/mip identity, GPU RGBA, CPU oracle RGBA, and channel deltas.
- Time, center, process identity, color/object-ID/visibility hashes, semantic joins,
  renderer errors, device removal, validation messages, and final cleanup state.

## Acceptance criteria

- The adapter reports barycentric and rasterizer-ordered-view support. `R32G32_UINT`
  supports render-target, shader-resource, and typed UAV load/store use;
  `R8G8B8A8_UNORM` supports typed UAV stores and copy to the swap-chain format.
  Unsupported capability is an explicit rejection, not a fallback.
- Surface generation reproduces complete catalog and texture hashes, preserves the
  accepted meshlet and animation hashes, and validates every primitive/vertex/material/
  mip reference within fixed 16-bit payload and descriptor capacities.
- Every workload retains exact accepted geometry/animation GPU-oracle counters. Every
  non-background visibility payload references a valid candidate, compact visible record,
  and primitive, and visibility-written plus background pixels equals 921,600.
- The shade stage dispatches exactly 14,400 groups and writes exactly 921,600 pixels for
  every radius, LOD, pose, material-count, and mip workload. CPU does not read visibility
  or indirect counts before submission.
- At least three canonical sample pixels resolve non-background geometry. Their object,
  primitive, barycentric, material, mip, and RGBA values match the CPU reconstruction;
  each quantized color channel differs by at most 2/255.
- Material sweeps observe no layer outside the configured count and observe every active
  layer when the canonical crowd is used. Forced LOD changes geometry and visibility
  hashes without changing shade dispatch. Shared and unique poses use the same resolve.
- Time 0 and 11 change color and visibility while preserving semantic joins. Returning to
  time 0, cached revisit, and Sidecar restart reproduce exact counters, visibility hash,
  color hash, raw object-ID hash, diagnostic hash, and sampled shade oracle.
- Debug correctness and release timing runs report no validation error, device removal,
  overflow, hidden fallback, or unbounded allocation. Both Sidecar namespaces are empty
  after cleanup.
- `runseal :skeletal-crowds`, `runseal :meshlet-scene`, `runseal :cooked-region`,
  `runseal :async-region`, `runseal :resident-stream`, and `runseal :guard` pass.

## Environment

The final report records Git revision and dirty state, Windows build, adapter and driver,
feature level, barycentrics, mesh-shader tier, shader model, Agility SDK, DXC, Rust
toolchain, debug-layer state, both catalog hashes, surface resources, workload sweeps,
GPU distributions, visual/visibility hashes, process identities, and cleanup.

## Reproduction

```powershell
runseal :surface-resolve
```

## Results

The canonical workflow passed in both Sidecar namespaces and produced one structured
acceptance report under ignored `out/`. The reference adapter reported D3D12 feature
level 12_1, mesh-shader tier 1, shader model 6.9, barycentrics, rasterizer-ordered views,
and all required typed format operations. Correctness used the debug layer; release
timing ran with validation disabled.

The immutable surface catalog contains 1,872 vertices, 3,648 expanded primitives, 64
materials, and a 64x64 seven-mip RGBA8 array. Its complete GPU image is 1,488,384 bytes
with SHA-256
`e9715635b9e9f2a7dd0089c35db3cb3ccd6ae87fc2119cc548ed2f37a4996989`.
The full surface execution allocation is bounded at 20,023,008 bytes.

The canonical radius-2 frame preserved all Experiment 0010 counters: 25,600 candidates,
18,928 visible objects, 512 active poses, 32,768 evaluated bones, 69,270 meshlets,
2,399,960 emitted vertices, 3,316,944 triangles, and 9,599,840 skin influences. One
indirect visibility dispatch produced 720,813 visible pixels and 200,787 background
pixels. One direct resolve submitted 160x90 groups and wrote all 921,600 pixels.

The canonical visibility hash was
`a58e18f0d7c304540a1cc459749fafbbf8d1cb6d6efb98d887347ddcc04d1965` and
the resolved color hash was
`050fb6b731e7966038cf8eb4f77454bac821304b20db1d191783719bda7c4f59`.
All six sampled pixels matched candidate, primitive, quantized barycentrics, stable key,
material, mip, exact texture texel, and packed RGBA reconstruction. Maximum channel
delta was zero against the pre-registered tolerance of 2/255.

The 32-sample optimized canonical run measured visibility P50 4.832256 ms, resolve P50
1.355552 ms, and total P50/P95/P99 6.346848/8.086496/8.820416 ms. The fully unique
128-bone case measured total P50/P95/P99 21.965760/30.969024/31.051264 ms. Cross-workload
material, mip, LOD, pose, and radius distributions contained desktop scheduling and
power-state noise, so they characterize this machine and do not establish architecture
thresholds. Every sweep retained fixed resolve work, exact CPU/GPU counters, valid
payloads, and zero sampled color error.

Implementation brought three failures to the surface before acceptance:

- Reusing the skeletal root constants exceeded D3D12's 64-DWORD root-signature limit.
  The resolve path now owns a narrow 28-DWORD contract.
- A forced-LOD sample disagreed at wrapped UV 1.0 because interpolated GPU values landed
  immediately below the boundary. The surface contract now canonicalizes that boundary
  and validates exact selected texels.
- Equal-depth fragments produced different raw visibility hashes across optimized
  repetitions despite identical aggregate counters and samples. A bounded
  rasterizer-ordered winner target now makes eight repeated forced-LOD captures
  byte-identical.

Material counts 1/8/64, mips 0/3/6, forced LODs 0/1/2, radii 0/1/2, shared and unique
poses, time 0/11/0, movement, cached revisit, and Sidecar restart all passed. Debug and
release processes reported no validation error, overflow, fallback, device removal, or
residual namespace. All required Experiments 0006-0010 and `runseal :guard` passed; one
transient empty release inspect response during the first skeletal regression retry did
not coincide with a process or device failure, and the unchanged canonical retry passed.

## Conclusion

Accepted. Compact candidate-addressed visibility plus a deterministic same-pixel winner
can decouple accepted skeletal meshlet execution from fixed-screen material resolution
without CPU enumeration or loss of semantic perception. Geometry, pose, and material
work now have independently measurable submission and storage bounds.

## Promotion

[ADR 0014](../../docs/adr/0014-gpu-surface-visibility-resolve.md) promotes the compact
visibility payload, deterministic winner, candidate map, normalized UV boundary,
surface reconstruction, and fixed-screen resolve contracts. `surface-catalog` owns the
deterministic reusable fixture boundary. Generated materials, textures, explicit mip
controls, point sampling, and laboratory lighting remain experiment fixtures.
