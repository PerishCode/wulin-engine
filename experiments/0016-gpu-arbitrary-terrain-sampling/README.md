# Experiment 0016: GPU Arbitrary Terrain Sampling

Status: Accepted

- Related ADRs: ADR 0019

## Hypothesis

The accepted atomic terrain/object pair can ground all 25,600 objects at deterministic
arbitrary positions inside their owning regions with exact GPU triangle interpolation,
zero cross-region boundary divergence, and no output-dependent CPU submission or new
terrain/instance runtime format.

## Scope

This experiment extends only the workbench-owned composition fixture. Standalone
instance generation and the accepted Experiment 0015 cell-center grounding encoding
remain unchanged.

Arbitrary positions lie on a 1/512 meter lattice. That lattice is exactly representable
by `f32` over the reference world extent and maps each 0.5 meter terrain cell to an
integer Q8 fraction. The GPU and CPU evaluate the same two triangles emitted by the
terrain mesh:

```text
first triangle, u + v <= 256:
q16 = h00 * (256 - u - v) + h10 * u + h01 * v

second triangle, u + v >= 256:
q16 = h10 * (256 - v) + h01 * (256 - u) + h11 * (u + v - 256)

ground_meters = q16 / 65536
```

The Q16 name denotes a signed fixed-point meter value with denominator 65,536. Terrain
samples remain the signed integer values already stored at 1/256 meter resolution.

The experiment does not accept terrain LOD composition, sampling outside an object's
owning region, slope frames, normal reconstruction, feet/IK, physics, collision,
navigation, or a general scene-query service.

## Workload

1. Preserve the Experiment 0015 cell-center fixture and exact ground hash as a
   compatibility case.
2. Generate 1,024 arbitrary-position objects per region over the same 25-region active
   pair. Interior cell fractions are deterministic Q8 values spanning both terrain
   triangles and the shared diagonal.
3. Place every first/last cell row and column exactly on its owning region boundary.
   Neighboring regions therefore contain 1,280 same-world-position edge pairs across
   40 logical neighbor edges.
4. Keep terrain and instance physical cache mappings deliberately different in all 25
   active slots.
5. Run terrain-first and object-first direct composition into one reverse-Z depth and
   one `R32_UINT` semantic attachment.
6. Sweep default, boundary, corner, high, and grazing cameras; then movement, revisit,
   teleport, and Sidecar restart.
7. Measure requested-only release probes for cell-center and arbitrary fixtures after
   warm-up. Report sampling/cull, skeletal, terrain, combined GPU, and publication
   distributions separately.

## Controlled Variables

- The 128 by 128 logical world, 5 by 5 active region window, terrain pack, camera
  collection, animation settings, catalog hashes, cache capacities, and pass orders are
  fixed.
- Terrain LOD, surface resolve, and occlusion remain disabled.
- The arbitrary fixture changes only object XZ positions and grounding encoding.
- Cell-center composition remains the default and retains its accepted formula and
  hash.

## Metrics

- Fixture kind, position lattice denominator, ground denominator, and exact position
  hash.
- GPU/CPU ground hashes over 25,600 signed values, minimum/maximum, mismatch count, and
  first mismatch.
- First-triangle, second-triangle, and diagonal sample counts.
- Logical neighbor edges, boundary pair comparisons, position mismatch count, ground
  mismatch count, and first mismatch.
- Ground-buffer allocation/readback bytes and cull-write/mesh-read counts.
- Pair token, transaction IDs, logical mappings, physical slots, and slot divergence.
- Fixed terrain and skeletal dispatch counts, shared clear count, color/object-ID
  hashes, known semantic ranges, and unknown IDs.
- P50/P95/P99 GPU stage, combined, and publication distributions.

## Pass Criteria

- The accepted Experiment 0015 cell-center GPU/CPU hash remains
  `7e6779f8a69768b2c883aa339865c823d00dcaed63e3d6fa588e823a1e0e162c`.
- Arbitrary mode writes exactly 25,600 finite signed Q16 ground values; GPU and CPU
  hashes are identical and mismatch count is zero.
- First-triangle, second-triangle, and diagonal counts are all non-zero.
- All 40 logical neighbor edges and 1,280 same-position boundary pairs are observed;
  position and ground mismatch counts are zero.
- Terrain and instance mappings differ physically for every active logical region.
- Culling and mesh emission consume the same validated ground values without an extra
  sampling-only dispatch.
- Terrain-first and object-first color and object-ID attachments are byte-identical,
  both semantic ranges are visible, and unknown-ID count is zero.
- CPU command recording remains fixed at the accepted three terrain and five skeletal
  dispatch/indirect operations, independent of sample values and visible output.
- Movement, revisit, teleport, and restart reproduce exact logical ground evidence.
- Experiment 0015 and affected standalone regressions pass with no validation error,
  device loss, unbounded allocation, or hidden fallback.

## Evidence

The canonical workflow is:

```powershell
runseal :terrain-sampling
```

Generated evidence remains ignored under
`out/captures/0016-gpu-arbitrary-terrain-sampling/`.

## Results

The canonical fixture writes exactly 25,600 signed Q16 ground numerators in the range
`[-123552, 164608]`. GPU and CPU hashes are both
`c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db`,
and the deterministic absolute-position hash is
`509b4ffb49cdbdd29b40d9be2baf3b8c8030508060fcadc43932eb497eb03658`.
Mismatch count is zero.

The fixture covers 9,025 first-triangle, 7,550 diagonal, and 9,025 second-triangle
samples. All 40 logical neighbor edges and 1,280 paired boundary positions are present.
Position-bit and ground-numerator mismatch counts are both zero. Terrain and instance
physical slots differ for all 25 active logical regions.

The canonical camera produces 10,497 visible animated objects, 512 shared poses,
45,789 meshlets, 1,789,218 emitted vertices, 2,460,672 emitted triangles, and 7,156,872
skin influences. Every GPU aggregate equals the CPU oracle. Terrain remains the
accepted 400-patch, 32,400-vertex, 51,200-triangle full-resolution path.

Terrain-first and object-first attachments are byte-identical. The color hash is
`b345988cb1fcda9e8c6e09a50106a6a3efdf47391bb181ff2473d218890d7b72` and the
object-ID hash is
`b1f4a196de5f4d801d58395d3767e0b6edf8cbdfed75b0d5814eef558972f292`.
Both semantic ranges are visible and unknown-ID count is zero. Default, boundary,
corner, high, and grazing cameras, movement, revisit, teleport, and Sidecar restart all
pass the exact oracle.

Release measurements contain 32 requested probes per pass order. Arbitrary
terrain-first combined GPU P50/P95/P99 is 4.972/8.513/10.395 ms and object-first is
6.057/10.587/10.880 ms. Their fused sampling-and-cull P95 values are 2.593 and 0.195
ms. Cached pair publication P50/P95/P99 is 25.402/33.892/36.609 ms and
25.614/33.729/33.905 ms respectively. The same run reproduces the accepted
cell-center hash
`7e6779f8a69768b2c883aa339865c823d00dcaed63e3d6fa588e823a1e0e162c`.
The observed long tails make these characterization data, not evidence for a preferred
fixture or pass order.

The same revision passes the canonical Experiment 0015 compatibility workflow and the
affected Experiments 0007-0014 workflows. All debug and benchmark Sidecar processes are
stopped after evidence collection.

## Conclusion

The hypothesis passes. Arbitrary 1/512 meter positions can be sampled against the
actual terrain triangles inside the existing fused GPU cull, consumed by mesh emission,
and validated exactly without changing either runtime format or CPU submission shape.
The same logical boundary positions remain bit-identical and height-identical across
independently resident region payloads.

The accepted boundary remains composition-specific and workbench-owned. Terrain LOD
composition, out-of-owner sampling, slope frames, normal reconstruction, feet or IK,
physics, collision, navigation, authored placement, and a general scene-query service
remain unaccepted.

## Promotion

Promote the Q8 position convention, exact two-triangle integer oracle, fixture-frozen
pair publication, and boundary-pair evidence as the baseline for the next composition
experiment. Do not promote a reusable terrain query API until a workload requires CPU
or non-render consumers.
