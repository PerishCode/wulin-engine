# Experiment 0094: Exact Canonical Object Position

Status: Accepted

## Hypothesis

One exact authored object already returned from the committed canonical snapshot can convert its
planar position into the sole `TerrainPosition` domain with checked integer work only, while object
identity remains scoped to its owner region and authored local ID.

## Scope

- Add `CanonicalObject::terrain_position` as a pure checked conversion of authored X/Z.
- Multiply by the existing 512 Q9 denominator and require an exact integer in the authored closed
  range `[-4096, 4096]` on each axis.
- Preserve `-4096..4095` in the owner region and normalize an authored `+4096` independently into
  local `-4096` of the checked adjacent region.
- Reject non-finite, non-lattice, out-of-range, and signed-region-overflow inputs without rounding
  or clamping.
- Return the derived position beside the unchanged raw object in the strict workbench query and
  independent source-byte oracle.

Enumeration, proximity, facing, tie-breaking, visibility, selection radius, interaction input or
action policy, persistent identity, collision/navigation, ECS ownership, networking, and Wulin
semantics are out of scope.

## Workload

1. Prove same-region lattice positions at signed far coordinates and all four authored closed-square
   corners.
2. Prove independent X, Z, and diagonal `+8m` normalization and checked `i64::MAX` overflow.
3. Reject NaN, infinity, half-Q9 values, and coordinates immediately outside the closed authored
   range.
4. Decode schema-3 bytes independently and compare raw object plus derived position for authored
   IDs 0, 31, 511, 992, and 1023.
5. Preserve those results through physical order A/B, source revisit, adjacent-window replacement,
   corrupt object/terrain rollback, and process restart.
6. Preserve the exact canonical GPU frame and immediate replay.
7. Run the complete optimized full acceptance across 32 reactive plus 32 prepared crossings, the
   eight-publication resource checkpoint, and two lifecycle checkpoint cycles.

## Controlled Variables

- Schema-3 bytes, object cache residency, committed-snapshot lookup, pair publication, GPU resources,
  renderer submission, terrain query, source namespace, and authored identity remain unchanged.
- Identity continues to use `(owner region, authored local ID)` even if the derived spatial region
  differs at a positive edge.
- Conversion consumes the already copied `CanonicalObject`; it performs no snapshot lookup, source
  I/O, allocation, GPU work, fence wait, synchronization, mutation, grounding, or height conversion.
- The implementation reuses `TerrainPosition::translated_q9` for the sole half-open normalization
  and signed-region overflow contract rather than introducing a second coordinate type.
- The old workbench response revision is replaced, not retained as a compatibility surface.

## Metrics

- Exact owner region, raw X/Z, spatial region, and local X/Z Q9 for every sample.
- Strict invalid-input and overflow outcomes.
- Query-side allocation/source/GPU/fence/synchronization counters.
- Physical-order/revisit/rollback/restart equality, frame hashes, traversal counts, active resource
  samples, lifecycle cleanup, and workflow duration.

## Acceptance Criteria

- Every accepted finite coordinate converts exactly with no rounding; every invalid coordinate fails
  without a partial position.
- IDs 0/31/992/1023 prove same/X/Z/diagonal region ownership independently from physical record
  order; ID 511 proves a positive X seam with a non-edge Z coordinate.
- A positive edge at `i64::MAX` fails through the existing checked region offset.
- A/B order, source revisit, adjacent publication, both failed pair types, and restart never change
  the raw or derived result for a retained object.
- All successful queries report zero additional work and the exact GPU frame/replay remains stable.
- No selection, interaction, identity, gameplay, networking, compatibility, format, asset, or Wulin
  boundary is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3
sources centered at `(2^40, -2^40)`, physical orders A/B, and the maintained canonical frame/runtime
workflows.

## Evidence

Four public-API integration tests pass. They cover far signed interior lattice points, all four
closed-square corners, independent seam normalization, NaN/infinity/non-lattice/out-of-range
rejection, and both positive-axis `i64::MAX` overflow cases. The complete repository test run passes
90 engine-runtime tests including these four.

`canonical-frame-v3` passed in 17,289.044 ms. The independent source-byte oracle proved:

- ID 0: owner/spatial `(2^40,-2^40)`, local `(-4096,-4096)`;
- ID 31: spatial X `+1`, local `(-4096,-4096)`;
- ID 511: spatial X `+1`, local `(-4096,-255)`;
- ID 992: spatial Z `+1`, local `(-4096,-4096)`;
- ID 1023: spatial X/Z `+1`, local `(-4096,-4096)`.

Every query reported zero allocation, source read, GPU copy/readback, fence wait, and synchronization.
The first and replay color hash remained
`8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135`; object-ID hash remained
`01951615d1b4645bdfba68991c75b8ea333482d312f31f39ed3b907ca479da5b`.

`canonical-runtime-v3` passed in 273.472 seconds. Twenty-five accepted real-process queries proved
A/B, revisit, adjacent, both failed-pair retentions, and restart. The newly admitted edge object's
owner `(baseX+3,baseZ)` and diagonal-edge spatial position `(baseX+4,baseZ+1)` remained deliberately
distinct. Both traversal sweeps passed 32 samples. The eight-publication checkpoint held 503 handles
at baseline/peak/final and private bytes finished 1,032,192 above baseline, within the 16 MiB bound.
Both lifecycle cycles cleaned up. The report retained 113 probes, 32 observations, six captures,
24 files, and 25,346,271 bytes.

Repository guard passes with zero Flavor deny issues. The deep resource/lifecycle workflow was not
repeated because this experiment adds a pure conversion of an already returned value and changes no
resource owner or lifetime; the full active/lifecycle checkpoints are the proportional integration
gate defined by the accepted ownership policy.

## Conclusion

Accepted. A committed `CanonicalObject` can now enter the existing exact terrain-position domain
without inventing selection or interaction policy. Authored identity ownership and normalized
spatial ownership remain explicit and separate.

## Promotion

Promoted `CanonicalObject::terrain_position`, the exact workbench response revision, public-API
conversion tests, and maintained source-byte seam evidence. Promoted no enumeration, selection,
interaction, persistent identity, second scene, format/asset change, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test -p engine-runtime --test object_position
runseal :canonical-frame
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-frame/` and
`out/captures/canonical-runtime/`.
