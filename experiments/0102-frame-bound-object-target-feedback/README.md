# Experiment 0102: Frame-Bound Object Target Feedback

Status: Accepted

## Hypothesis

The retained source-qualified prototype target can produce exact visible feedback through the sole
existing object surface path if identity is supplied as immutable frame input and projected only
after the frame's composition publication, without adding a marker, object list, pass, allocation,
copy, readback, synchronization point, or renderer-owned product target.

## Scope

- Add one optional `CanonicalObjectIdentity` to `FrameRequest`. Runtime and renderer retain no
  product target between frames.
- Validate authored local ID before render work, then project source and owner region against the
  post-publication snapshot. Source replacement, no composition, and window departure produce an
  ordinary no-feedback frame.
- Put the streamed authored local ID in the previously-zero high identity word of the existing
  56-byte `VisibleObject`; preserve the actor's two-word generation identity.
- Reuse `surface_animation.yzw` for enabled/semantic-region/authored-ID constants and apply a static
  amber emphasis inside the existing surface resolve.
- Reuse word 5 of the existing 32-byte surface statistics buffer for emphasized pixels. On probes,
  independently derive the exact expected pixel count from visibility candidate addresses and the
  committed authored-ID permutation, and run the existing independent CPU shade oracle.
- Let the prototype submit its identity-only target on every frame while its accepted target policy
  retains intent. Keep validation cadence and target ownership from Experiment 0101 unchanged.
- Add strict workbench target set/clear controls only as diagnostic frame-input ownership for GPU
  acceptance; they do not mutate Runtime product state.

Interaction effects, outlines/markers, pulsing, visibility or line-of-sight policy, selection
eligibility, gameplay-persistent identity, a second scene/index, automatic scans, new assets or
formats, networking, and Wulin semantics are out of scope.

## Workload

1. Unit-test invalid local IDs, source mismatch, outside-window identity, exact current projection,
   and same-source semantic remapping.
2. Prove authored-ID matching under a non-identity physical permutation and exclude neighboring
   physical candidates.
3. Capture baseline/replay, select one actually visible streamed sample, reject ID 1024, set its
   exact target, capture emphasized/replay frames, clear, and require exact baseline replay.
4. Keep one target across source A to source B replacement, return to A, same-source far departure,
   and return; require immediate disable/restore against each frame's post-publication snapshot.
5. Launch native F+W and require the acquired identity to be submitted in the same product frame
   without copied canonical object content.
6. Preserve full rollback, restart, 32+32 traversal, resource plateau, artifact bound, and two-cycle
   lifecycle acceptance.

## Controlled Variables

- Object sources, schema-3 page validation, cache and publication ownership, skeletal cull,
  visibility, shadow, occlusion, materials, frame transaction, and presentation time are unchanged.
- Page admission already proves every authored ID `0..1023` occurs exactly once, so projection does
  not scan a CPU page and the shader does not read source pages.
- Target constants occupy existing root-constant words; the root remains 60 DWORDs and the surface
  descriptor count remains 98.
- The visible record remains 56 bytes, surface statistics remain 32 bytes, and probe-only readback
  uses the already-existing statistics copy.
- The amber transform is static: `base * 0.45 + (1.0, 0.62, 0.08) * 0.55`. It does not depend on
  time, availability state, or host input.

## Metrics

- Projected active index, semantic region, authored local ID, emphasized pixel count, CPU/GPU sample
  delta, color hash, and semantic attachment hash.
- Source-replaced, departed, revisited, and returned projected-target presence.
- Visible-record bytes, root constants, descriptors, dispatches, resources, handles, threads,
  private bytes, artifact count/bytes, and full workflow duration.
- Prototype submitted identity/count and copied-state absence.

## Acceptance Criteria

- Invalid authored ID fails before render work. Source mismatch and outside-window identities do not
  fail or alias another object; they produce zero target pixels.
- A current exact identity changes only its object's resolved color, reports a nonzero pixel count
  equal to the independent visibility/address oracle, and leaves the semantic attachment unchanged.
- Target replay is exact; clear exactly restores baseline. Same-source revisit restores feedback,
  while source replacement and departure disable it in their first rendered frame.
- Product target state remains prototype-owned and identity-only. Runtime/renderer keep no retained
  target, second object list, copied object content, new pass/resource, or synchronization path.
- Existing GPU, rollback, traversal, resource, restart, and lifecycle contracts pass.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 A/B sources centered at
`(2^40, -2^40)`, and the maintained focused/full acceptance workflows.

## Evidence

All 94 engine-runtime tests, all prototype tests, and reference-host tests pass. Projection tests
cover invalid/source/window/remap cases; the surface test proves authored-ID matching after physical
permutation and excludes its neighbor.

`canonical-frame-v8` passes in 24.280 seconds. It selects visible center-region ID 987 from the
independently checked probe sample, rejects ID 1024, and reports exactly 3,472 emphasized pixels in
both target frames. Color changes from `8b13d214…4135` to `6d483d96…6292`; the object-ID attachment
stays `01951615…da5b`; clear returns exactly to the baseline color/stable evidence.

`canonical-prototype-v20` passes in 73.149 seconds. Native F+W again acquires ID 496 from committed
local Z `-32` Q9, submits that complete identity in one target-feedback frame, copies no object state,
and revalidates from publication token 1 to traversal token 2.

The final-worktree `canonical-runtime-v10` passes in 282.045 seconds. Source replacement and
same-source departure expose no projected target; source
revisit and return restore the identical 3,472 pixels. Existing A/B, rollback, restart, 32+32
traversal, and two lifecycle cycles pass.

Resource convergence uses five warm publications and eight measured publications. Handles remain
527 and threads 24; private bytes move from 412,069,888 to 412,659,712 (+589,824), within the
accepted plateau. The report remains 24 files totaling 25,346,225 bytes.

## Conclusion

Accepted. A source-qualified identity is now sufficient for exact frame-bound visible feedback
through the sole surface path. Snapshot projection makes source/window lifetime visible in the same
frame transaction without adding persistent renderer state or structurally new GPU work.

## Promotion

Promoted optional frame target input, post-publication target projection, authored local ID in the
existing visible record, static exact-match surface emphasis, existing-stats pixel evidence,
prototype identity forwarding, workbench diagnostic target controls, and source/window lifecycle
gates. Promoted no interaction effect, marker/outline, time pulse, persistent gameplay identity,
engine-owned target, second scene/index, resource/pass/synchronization, asset/format, networking, or
Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype -p reference-host
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :canonical-frame
runseal :canonical-prototype
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/`.
