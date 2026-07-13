# Experiment 0017: GPU LOD Terrain Composition

Status: Accepted

- Related ADRs: ADR 0020

## Hypothesis

The accepted GPU terrain LOD path can render the atomic terrain/object composition
while all 25,600 arbitrary-position objects retain their exact full-resolution ground
values, the selected visible terrain approximation stays within a registered 0.125
meter contact-error bound, and CPU submission remains fixed and independent of selected
LOD or visible work.

## Scope

This experiment composes the accepted Experiment 0014 terrain patch LOD with the
accepted Experiment 0016 arbitrary-position grounding path. Full-resolution terrain
heights remain the physical and grounding source of truth. Camera-selected LOD changes
only visible terrain geometry.

The canonical contact oracle evaluates the actual selected terrain triangles at every
fixture XZ position. It uses signed Q18 meter numerators with denominator 262,144 so
full-resolution, LOD1, LOD2, and transition-edge interpolation remain exact integers.
The 0.125 meter bound is a fixture-specific visual approximation gate, not a general
terrain error policy.

The experiment does not accept camera-dependent object ground, geomorphing, authored
terrain, general screen-space-error policy, slope frames, feet/IK, physics, collision,
navigation, sampling outside the owning region, or a reusable scene query.

## Workload

1. Reproduce Experiment 0016 arbitrary-Q8 composition with terrain LOD disabled and
   preserve its exact ground, position, and attachment hashes.
2. Enable automatic terrain LOD with near/middle patch radii 2/6. Require the accepted
   canonical patch distribution `[25,144,231]`, exact transition-edge oracle, and
   reduced emitted geometry.
3. For every object, evaluate the selected visible terrain triangle, including a
   coarser neighbor's transition edge where applicable. Compare it to the unchanged
   full-resolution Q18 ground and report signed/absolute residual distributions.
4. Force terrain LOD0, LOD1, and LOD2. LOD0 must be byte-identical to the disabled
   baseline. LOD1, LOD2, and automatic mode must remain inside the contact bound.
5. Run terrain-first and object-first composition for disabled, automatic, and forced
   controls. Require byte-identical attachments within each controlled workload.
6. Sweep interior, patch-edge, region-edge, corner, high, and grazing cameras. Camera
   movement may change LOD selection and contact residuals but must not change exact
   ground values or pair publication.
7. Move, revisit, teleport, and restart the atomic pair while LOD is enabled. Require
   exact logical grounding, boundary continuity, physical-slot independence, and
   deterministic LOD/contact evidence for repeated state.
8. Collect 32 requested release probes for disabled, automatic, forced LOD1, and forced
   LOD2 composition. Report work, terrain, fused ground/cull, skeletal, combined GPU,
   and publication distributions separately.

## Controlled Variables

- Terrain and region format V1, canonical pack, arbitrary-Q8 fixture, 5 by 5 active
  region set, 25,600 objects, cache capacities, animation settings, semantic ranges,
  and 1280x720 attachments remain unchanged.
- Exact grounding always samples full-resolution terrain triangles and writes Q16
  values. Mesh emission consumes the same ground buffer regardless of terrain LOD.
- Terrain LOD uses the accepted three levels, patch topology, camera-patch convention,
  transition projection, and 2/6 automatic bands.
- LOD disabled records the accepted three terrain operations. LOD enabled adds exactly
  one fixed 400 by 2 by 1 transition-validation dispatch. The five skeletal operations
  remain unchanged.
- Correctness uses the debug Sidecar namespace. Timings use the release namespace with
  validation disabled. Contact analysis is requested-only CPU evidence over already
  published terrain and instance snapshots; it does not alter normal frame submission.

## Metrics

- Exact ground and position hashes, denominator, range, mismatch count, boundary pairs,
  triangle coverage, ground allocation/readback bytes, and mesh consumption count.
- Terrain LOD settings, camera patch, LOD hash/counts, edge/transition oracle, emitted
  patches/vertices/triangles, reduction, fixed dispatches, and resources.
- Contact Q18 denominator; selected-surface and residual hashes; negative, zero, and
  positive counts; minimum/maximum signed residual; maximum absolute residual; P50,
  P95, and P99 absolute meters; threshold and exceedance count.
- Pair token, transaction IDs, mappings, physical-slot divergence, publication count,
  and publication duration.
- Color/object-ID hashes, semantic ranges, unknown IDs, process identities, validation
  state, and device-removal state.
- P50/P95/P99 terrain, fused ground/cull, skeletal stage, combined GPU, and publication
  distributions by controlled LOD and pass order.

## Pass Criteria

- The disabled arbitrary-Q8 ground hash remains
  `c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db`,
  position hash remains
  `509b4ffb49cdbdd29b40d9be2baf3b8c8030508060fcadc43932eb497eb03658`,
  and disabled color/object-ID attachments retain their accepted hashes.
- Ground hash, range, triangle coverage, boundary evidence, and skeletal use of the
  ground buffer are byte-identical across disabled, automatic, forced LOD0/1/2, every
  camera, movement, revisit, teleport, and restart.
- Disabled and forced LOD0 contact residuals are exactly zero for all 25,600 samples.
  Automatic, forced LOD1, and forced LOD2 have no residual whose absolute value exceeds
  0.125 meter. Every workload records all 25,600 samples and no non-finite value.
- Automatic LOD reproduces `[25,144,231]`, transition mismatch count zero, maximum
  adjacent delta at most one, and positive vertex/triangle reduction. Forced levels
  reproduce their accepted exact geometry aggregates.
- LOD-disabled terrain-first/object-first attachments remain byte-identical, and both
  orders are also byte-identical within each LOD-enabled control. Both semantic ranges
  are visible and unknown-ID count is zero.
- LOD disabled retains three fixed terrain and five fixed skeletal operations. LOD
  enabled records exactly four fixed terrain and five fixed skeletal operations,
  independent of selected distribution, residuals, visible objects, and emitted work.
- Pair publication, movement, revisit, teleport, and restart preserve atomic snapshots,
  25/25 physical mapping divergence, exact arbitrary grounding, and bounded contact
  evidence.
- Experiments 0014-0016 and affected standalone regressions pass with no validation
  error, device loss, hidden fallback, unbounded growth, or residual Sidecar process.

## Evidence

The canonical workflow is:

```powershell
runseal :lod-composition
```

Generated evidence remains ignored under
`out/captures/0017-gpu-lod-terrain-composition/`.

## Results

LOD-disabled composition reproduces the accepted arbitrary-Q8 ground, position, color,
and object-ID hashes. Forced LOD0 writes zero contact residual for all 25,600 samples
and remains byte-identical to the disabled attachments despite recording the additional
fixed transition-validation dispatch.

Canonical automatic LOD reproduces patch counts `[25,144,231]`, 59 transition edges,
maximum adjacent delta one, and zero transition mismatch. Emitted geometry falls from
32,400 vertices and 51,200 triangles to 7,704 and 9,656, reductions of 76.22 and 81.14
percent. The exact ground hash remains
`c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db`.

The automatic Q18 contact residual hash is
`9475aa38c5906ad6216b9ffa03d0fc8f57bd538db540fc70f083538ded8ed703`.
There are 2,640 exact-zero samples. Absolute P50/P95/P99 is
0.003662/0.028564/0.045410 meter and the maximum is 0.089600 meter. Signed residuals
range from -23,488 to 22,400 Q18 units. No sample exceeds the 0.125 meter gate.

Forced LOD1 emits 10,000 vertices and 12,800 triangles; its maximum contact residual is
0.020508 meter. Forced LOD2 emits 3,600 vertices and 3,200 triangles; its maximum is
0.089600 meter. Forced LOD0, LOD1, LOD2, and automatic attachments are each
byte-identical between terrain-first and object-first execution. Automatic color and
object-ID hashes are
`579f3800a4f9603e5298919a7da34fe66a04f54d6bbcd464666dfd67449c158a` and
`20d8017adf2bcde946eff8a8f834d563d9f703b0a4a3bf142fd06478c27fcc75`.

Interior, patch-edge, region-edge, corner, high, and grazing cameras preserve the exact
ground hash while changing only registered LOD/contact evidence. Adjacent movement,
diagonal movement, teleport, logical revisit, and process restart all pass. Teleport's
maximum contact residual remains below 0.09 meter, and every movement state has zero
threshold exceedance.

Release measurements contain 32 pair publications and requested probes for each LOD
and pass-order combination. Automatic combined GPU P50/P95/P99 is
5.609/5.764/5.771 ms terrain-first and 5.564/6.176/6.790 ms object-first. Automatic
terrain P50/P95/P99 is 0.248/0.373/0.396 ms and 0.141/0.142/0.146 ms. Cached
publication P50/P95/P99 is 25.674/32.858/33.903 ms and 25.648/34.049/34.860 ms.
Disabled and forced controls show similar order-sensitive and tail variability, so the
experiment accepts exact work reduction and bounded visual error, not a frame-time
speedup or preferred pass order.

## Conclusion

The hypothesis passes. Terrain render LOD can participate in atomic terrain/object
composition while exact full-resolution grounding remains camera-independent and
byte-stable. The selected visible surface approximation is measurable, deterministic,
and bounded for the canonical terrain rather than silently conflated with physical
ground.

The accepted boundary remains workbench-owned. It does not establish a general
screen-space-error policy, authored-terrain tolerance, geomorphing, local contact
refinement, camera-dependent ground, slope frames, feet/IK, collision, navigation, or a
reusable scene query.

## Promotion

Promote exact-ground/render-approximation separation, Q18 selected-surface contact
evidence, and LOD-aware fixed composition submission as the baseline for the next
terrain/object visual experiment. Keep the 0.125 meter threshold fixture-local and
require future terrain sources to register their own error gate.
