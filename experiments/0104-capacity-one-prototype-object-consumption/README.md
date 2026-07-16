# Experiment 0104: Capacity-One Prototype Object Consumption

Status: Accepted

## Hypothesis

The prototype can consume exactly one acted-on canonical object for its process lifetime by
retaining only the source-qualified identity, excluding it from later nearest selection, and
eliminating its exact streamed candidate in the existing skeletal cull, without mutating canonical
source data or adding GPU work/resource ownership.

## Scope

- Commit one consumed identity with the existing successful Activated frame.
- Add one optional exact nearest-query exclusion while preserving complete candidate validation,
  scan count, tie order, and zero-allocation behavior.
- Defer visual suppression until the existing 12-frame acknowledgement ends.
- Project one immutable suppression through source/window lifetime and reject the exact candidate
  in the sole skeletal cull before downstream rendering work.
- Retain consumption across same-source window departure; clear it on source replacement or
  process restart; reject replacement when the capacity-one slot is occupied.

Mutable canonical packs/snapshots, a general object registry, inventories, drops, respawn,
persistence, networking, and Wulin semantics are out of scope.

## Workload

1. Unit-test nearest exclusion, unchanged 25,600 candidate scan count, next-object selection,
   invalid source/ID rejection, and exact suppression projection/packing.
2. Exercise action commit, immediate exclusion, deferred suppression, exactly 12 acknowledgement
   frames, capacity exhaustion without resolution, same-source retention, source replacement clear,
   and restart-empty state.
3. Capture baseline/suppressed/replay/clear frames. Require exactly one fewer visible, shadow, and
   occlusion-source candidate; one more skeletal rejection; CPU/GPU equality; changed color;
   unchanged semantic object-ID attachment; and exact restoration.
4. Run native F+Enter+W and require the committed identity to become the exclusion in the same
   successful frame while suppression remains deferred behind the acknowledgement.
5. In full acceptance, require source replacement and window departure to unproject suppression,
   source revisit and window return to restore it, then preserve traversal, rollback, resource, and
   lifecycle gates.

## Controlled Variables

- Canonical source/page/publication ownership, object identity, actor transaction, camera, cull
  dispatch count, visible-record layout, shadow/surface passes, and frame synchronization remain
  unchanged.
- Candidate exclusion affects winner selection only after validation and candidate counting.
- Suppression uses one formerly unused skeletal root constant word: enable bit 31, active index in
  five bits, and authored local ID in ten bits.
- The visible record remains 56 bytes, skeletal root constants remain 60 DWORDs, and no resource,
  descriptor, pass, allocation, copy, readback, or synchronization is added.

## Metrics

- Consumed/excluded/submitted/projected identities, acknowledgement frames, capacity outcome, and
  nearest candidate count.
- Skeletal visible/rejected counts and CPU oracle, shadow casters, occlusion source-visible count,
  color/object-ID hashes, replay, and clear restoration.
- Existing handles, threads, private bytes, artifacts, workflow duration, and stage timings.

## Acceptance Criteria

- Consumption commits only with the exact successful Activated projection and never replaces a
  live capacity-one identity.
- The consumed identity cannot win nearest selection, but all candidates remain validated and
  counted and the deterministic next candidate wins.
- Acknowledgement behavior is unchanged; suppression starts only after it ends.
- Exact source/window projection controls suppression. Same-source return restores it; source
  replacement and restart clear prototype consumption.
- Exactly one streamed object is removed before every downstream rendering path, with CPU/GPU
  equality and no structural GPU/resource growth.
- Focused, native-process, final full-runtime, guard, and workspace checks pass.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 sources centered at
`(2^40, -2^40)`, and the maintained focused/full acceptance workflows.

## Evidence

All 96 engine-runtime tests, six canonical-object position/proximity tests, six interaction-policy
tests, seven observation-policy tests, and the remaining prototype/reference-host tests pass.

`canonical-frame-v10` passes in 40.920 seconds. Suppressing visible ID 987 changes visible
10,538→10,537, rejected 15,062→15,063, shadow casters 10,538→10,537, and occlusion source-visible
10,538→10,537. CPU/GPU results match; replay is exact; color changes; the semantic object-ID
attachment is unchanged; and clear plus one history-warm frame restores the baseline exactly.

`canonical-prototype-v22` passes in 75.750 seconds. Native F+Enter+W commits ID 496 at exact delta
`(160, 0)` Q9 / squared distance 25,600, stores the same identity as the immediate nearest
exclusion, leaves 11 acknowledgement frames, and correctly defers suppression. Idle and restarted
processes contain no consumed identity.

The final-worktree `canonical-runtime-v12` passes in 265.079 seconds. The exact ID 987 suppression
projects in source A, is absent after source-B replacement, returns after source-A revisit, is
absent outside the same-source active window, returns with the window, and clears back to the exact
baseline. Existing rollback, restart, reactive/prepared 32+32 traversal, and two lifecycle cycles
pass.

Five warm and eight measured resource publications retain 492 handles and 21 threads; private bytes
settle from 427,048,960 to 426,463,232 (-585,728). The report contains 24 files / 25,346,262 bytes
and records 988 Sidecar invocations. Stage times are 9.244 seconds setup, 24.446 bootstrap, 16.984
prototype, 12.592 actor lifecycle, 28.478 simulation actor, 97.391 canonical correctness, 18.759
reactive traversal, 13.630 prepared traversal, 26.285 resources, and 15.268 lifecycle.

The first full run exposed that an early suppression return left the grounding diagnostic slot
stale across A→B→A rebinding. Moving exact rejection after the existing grounding write but before
frustum/visibility removed the defect; the focused and final full reruns both pass.

## Conclusion

Accepted. The prototype owns one runtime-session consumption slot; nearest selection and the sole
render cull consume the same qualified identity without mutating canonical content.

## Promotion

Promoted capacity-one consumed identity, exact nearest exclusion, post-ack immutable frame
suppression, source/window lifetime, exact early skeletal rejection, and native/full lifecycle
evidence. Promoted no registry, inventory, dispatcher, source rewrite, respawn, persistence,
networking, or Wulin behavior.

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
