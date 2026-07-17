# Experiment 0109: Capacity-Exhausted Object-Action Feedback

Status: Accepted

## Hypothesis

After the Prototype's immutable capacity-one consumption is committed, an Enter attempt against a
different currently resolved target can project the existing red `Rejected` frame feedback without
resolving canonical object data, changing consumption, or interrupting suppression of the consumed
identity.

## Scope

- Keep the exact consumed identity, committed count, nearest exclusion, and source/session lifetime
  unchanged.
- If capacity is exhausted and a resolved retained target exists, derive one immutable red
  `Rejected` candidate from that target identity only.
- Reuse the existing frame transaction and 12-successful-projected-frame acknowledgement owner.
- Keep the already consumed identity suppressed while the different rejected target is
  acknowledged.
- Preserve feedback-free `CapacityExhausted` for missing or unavailable targets.
- In one sustained native process, move after first consumption, release locomotion, re-press
  `F+Enter`, and require an independently computed exclusion-aware second target, red projection,
  unchanged first consumption, and exact final completion state.

Canonical object resolution, proximity/facing work, a second consumption slot, a second
acknowledgement/timer, a rejection queue/history, canonical mutation, registry, inventory, reward,
dispatch, respawn, persistence, networking, or Wulin semantics is out of scope.

## Workload

1. Extend the pure interaction policy with a typed capacity rejection carrying only target identity
   and existing `Rejected` feedback.
2. Prove projected/unprojected capacity rejection, missing/unavailable fallback, exact counter
   updates, immutable consumption, and simultaneous first-identity suppression.
3. Keep outside-facing rejection's exact proximity/facing evidence and behavior unchanged.
4. Extend the bounded sustained session with exact native key-up/key-down transitions for
   locomotion stop and `F+Enter` re-press.
5. Compute the expected second target from the committed final actor position, source pack, window,
   radius, and exact consumed-identity exclusion.

## Controlled Variables

- Runtime, renderer, object feedback ABI, surface colors, canonical source/snapshot, actor
  transaction, nearest query, object resolver, and GPU resource shape remain unchanged.
- Capacity rejection validates no canonical object payload and carries no proximity or facing
  evidence.
- The existing acknowledgement advances only on exact returned projected frames.
- The consumed identity remains the sole nearest exclusion and sole frame suppression candidate.

## Metrics

- Exact first/second qualified identities, feedback kind, acknowledgement kind/count, committed and
  ineligible counts, target state, suppression frame count, native transitions, process identity,
  completion sequence, actor position, stdout count, exit code, stderr, and focused duration.
- Full-runtime resource/lifecycle acceptance is required only if the frame/suppression transaction
  changes structurally; otherwise the maintained focused Prototype workflow is the acceptance owner.

## Acceptance Criteria

- A resolved second target produces projected `Rejected` with reason `capacity-exhausted`,
  `applied=false`, and no canonical object resolution/proximity/facing evidence.
- The first consumed identity, committed count one, nearest exclusion, and suppression remain exact
  throughout the second target's acknowledgement.
- Missing/unavailable capacity attempts remain feedback-free and every malformed input rolls back.
- The sustained real process matches an independent exclusion-aware source oracle and exits with
  exactly one readiness and one completion.
- Focused tests, `runseal :guard`, and `runseal :canonical-prototype` pass without engine/GPU/resource
  ownership changes.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained bounded Prototype session workflow.

## Evidence

All 45 Prototype tests pass, including 11 exact interaction-policy tests. The added cases prove
projected and unprojected capacity rejection, distinct/resolved-target validation, feedback-free
unavailable fallback, no proximity/facing report fields, stable committed/consumed state, and
continuous first-identity suppression throughout the second identity's acknowledgement. Workspace
Clippy, Deno checks, Flavor, and `runseal :guard` pass with no new deny issue.

`canonical-prototype-v26` passes in 80.596 seconds. One real process consumes qualified ID 496 at
readiness live frame 5, continues moving, then receives exact `D up`, `F up/down`, and `Enter
up/down` transitions. The exclusion-aware source oracle selects qualified ID 501 from the final
stationary actor position. Completion at live frame 792 retains ID 496 as the sole consumed
identity and nearest exclusion, retains ID 501 as a resolved target, reports committed/ineligible
counts 1/1, exactly 12 Rejected frames, 776 projected suppression frames, no acknowledgement, and
no copied object state or event history.

The engine, renderer, shaders, frame ABI, resources, descriptors, copies, readback, and
synchronization are structurally unchanged, so the maintained focused Prototype workflow owns this
product-policy risk; the full-runtime workflow is not repeated.

## Conclusion

Accepted. Capacity exhaustion can be visible as exact red frame feedback for a different resolved
target while the immutable first consumption remains suppressed and no canonical object resolution
or second effect occurs.

## Promotion

Promoted typed identity-only capacity rejection, distinct/resolved-target validation, concurrent
consumed suppression plus rejected-target acknowledgement, exact native post-readiness
re-observation/action input, and exclusion-aware source-oracle evidence. Promoted no second
consumption, proximity/facing authority, object resolution, timer, result history, canonical
mutation, registry, inventory, reward, dispatch, respawn, persistence, networking, or Wulin
semantics.

## Reproduction

```powershell
cargo test --locked -p prototype
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :guard
runseal :canonical-prototype
```

Generated reports remain ignored under `out/captures/`.
