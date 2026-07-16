# Experiment 0103: Committed Prototype Object Action

Status: Accepted

## Hypothesis

One normalized Enter press can produce an exact player-visible action over the prototype's retained
object target if eligibility is recomputed from the committed actor position and the action commits
only when the same source-qualified identity is projected into the successful frame, without
introducing mutable canonical objects or engine-owned product interaction state.

## Scope

- Promote one pure `CanonicalObject::proximity_from` calculation returning exact terrain position,
  signed Q9 deltas, and Q18 squared distance inside an inclusive radius. Make nearest-object search
  consume the same calculation.
- Replace the identity-only frame target directly with immutable `ObjectTargetFeedback { identity,
  kind }`, where kind is `Selected` or `Activated`; retain no compatibility field.
- Return submitted feedback from `RenderOutcome` only when post-publication projection found that
  exact identity in the frame snapshot.
- Reuse the existing root word and sole surface resolve: mode 1 is the accepted amber selection,
  mode 2 is a fixed green activation acknowledgement. Add no GPU pass, resource, descriptor,
  allocation, copy, readback, or synchronization.
- Add one prototype-owned capacity-one Enter intent. Resolve the retained identity and apply the
  fixed inclusive 512-Q9 gate only after a nonzero actor commit. Commit a 12-successful-frame
  acknowledgement only after exact activated projection.

Persistent object mutation, inventories, interaction dispatch, a gameplay/network identity,
visibility or line-of-sight policy, new assets/formats, networking, and Wulin semantics are out of
scope.

## Workload

1. Unit-test exact inclusive proximity, one-Q9 exclusion, signed extremes, and equality with the
   result used by nearest search.
2. Exercise pending intent across fractional/stalled/render-blocked work, reset/suspend
   cancellation, committed ineligible consumption, malformed resolution rollback, projection miss,
   target change, and exactly 12 successful projected acknowledgement frames.
3. Capture selected and activated frames for one visible object, require identical exact pixel sets,
   distinct deterministic colors, unchanged semantic attachment, replay, clear, and invalid
   ID/feedback rejection.
4. Post native F+Enter+W before readiness. Require observation and action to use the same committed
   actor output, exact retained identity/proximity, one projected activated frame, committed count
   one, remaining acknowledgement count 11, and no copied canonical object state.
5. Preserve source replacement/window departure/revisit, rollback, restart, 32+32 traversal,
   resource convergence, artifact bound, and lifecycle acceptance.

## Controlled Variables

- Canonical source/page/publication ownership, target snapshot lifetime, actor transaction, camera,
  cull, visible-record layout, object-ID attachment, surface dispatch, and frame synchronization are
  unchanged.
- Eligibility reads only the already-committed actor output and one on-demand typed resolution of
  the retained identity; it does not rerun nearest selection.
- The visible record remains 56 bytes, root constants remain 60 DWORDs, surface statistics remain
  32 bytes, and descriptor count remains 98.
- Selected uses `base * 0.45 + (1.0, 0.62, 0.08) * 0.55`; Activated uses
  `base * 0.30 + (0.12, 1.0, 0.32) * 0.70`.
- The acknowledgement duration counts only successful frames that project the same activated
  identity. Target change or unavailable lifetime clears it.

## Metrics

- Exact target identity, terrain position, signed Q9 deltas, Q18 squared distance, attempt outcome,
  projected feedback, commit count, remaining acknowledgement frames, and copied-state absence.
- Selected/activated target pixels, color hashes, semantic attachment hashes, replay, and clear.
- Existing dispatch/resource shape, handles, threads, private bytes, artifact count/bytes, and full
  workflow duration.

## Acceptance Criteria

- Proximity is pure, exact, inclusive, signed-safe, and the sole distance authority consumed by
  nearest search and the prototype action.
- No nonzero actor commit means no resolution, proximity work, or action consumption. Reset and
  suspension cancel pending input; malformed resolution returns no policy mutation.
- An eligible attempt commits only when the successful frame returns the exact submitted activated
  feedback. Projection miss commits no acknowledgement.
- The first projected frame leaves 11 of 12 acknowledgement frames. Only successful matching frames
  decrement the remainder; changed/unavailable targets clear it.
- Native F+Enter+W acquires and acts on the exact target in one committed product step without
  copied object content or engine-owned product state.
- All focused/full GPU, rollback, traversal, resource, restart, and lifecycle contracts pass.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 sources centered at
`(2^40, -2^40)`, and the maintained focused/full acceptance workflows.

## Evidence

All 94 engine-runtime tests, six public canonical-object position/proximity tests, five interaction
policy tests, two asynchronous observation-order support tests, and the remaining
prototype/reference-host tests pass. The support gate accepts both valid orderings—observation then
traversal revalidation, or traversal then direct observation of the published token—and rejects
stale, uncounted, regressing, or excessive publication states.

`canonical-frame-v9` passes in 32.302 seconds. Selected and Activated both affect exactly 3,472
pixels of visible ID 987. Their color hashes are `6d483d96…6292` and `315a59de…a683`; the object-ID
attachment is unchanged, both replay exactly, clear restores baseline, and unknown `pulsing`
feedback plus ID 1024 are rejected.

`canonical-prototype-v21` passes in 72.589 seconds. Native F+Enter+W acquires ID 496 at exact
committed delta `(160, 0)` Q9 and squared distance 25,600, returns the same activated identity from
the frame, commits once, retains 11 acknowledgement frames, and copies no object state.

The final-worktree `canonical-runtime-v11` passes in 245.223 seconds. Source replacement and
same-source departure suppress feedback; source revisit and return restore exactly 3,472 pixels.
Existing rollback, restart, reactive/prepared 32+32 traversal, and two lifecycle cycles pass.

Four warm and eight measured resource publications retain 492 handles and 21 threads; private bytes
move from 425,627,648 to 426,115,072 (+487,424), within the accepted plateau. The report contains 24
files / 25,346,259 bytes. The state-driven full workflow records 943 Sidecar invocations and stage
times of 7.408 seconds setup, 24.324 bootstrap, 16.869 prototype, 12.426 actor lifecycle, 28.628
simulation actor, 86.095 canonical correctness, 13.626 reactive traversal, 13.811 prepared
traversal, 24.631 resources, and 15.443 lifecycle.

## Conclusion

Accepted. The prototype now owns one exact committed object action and bounded visible
acknowledgement. Runtime remains an immutable frame-transaction consumer and canonical objects
remain read-only.

## Promotion

Promoted pure exact object proximity, selected/activated immutable frame feedback, exact projected
feedback outcome, capacity-one Enter intent, committed 512-Q9 eligibility, 12-successful-frame
acknowledgement, and native product evidence. Promoted no object mutation, gameplay dispatcher,
persistent identity, new GPU work/resource, networking, or Wulin behavior.

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
