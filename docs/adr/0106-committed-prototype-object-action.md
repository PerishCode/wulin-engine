# ADR 0106: Committed Prototype Object Action

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0103 Committed Prototype Object Action

## Context

ADR 0104 retains one source-qualified object target in the prototype. ADR 0105 makes that identity
visible as immutable amber frame feedback. Neither establishes whether an input action is eligible
at the actor's current committed position or whether a visible effect actually entered the
successful frame.

Retained `Resolved` availability is insufficient because an actor can move far within the same
published window. Repeating nearest selection is also incorrect: another closer object can win even
while the retained target remains eligible. A canonical object overlay would prematurely introduce
mutable lifetime authority across lookup, nearest, GPU cull, and publication.

## Decision

- Define pure `CanonicalObject::proximity_from(origin, max_distance_q9)`. It converts the object's
  exact authored position into `TerrainPosition`, computes signed region-aware Q9 deltas and Q18
  squared distance, and returns `None` outside the inclusive circle. Existing nearest search reuses
  it.
- Replace the identity-only `FrameRequest` field with optional immutable
  `ObjectTargetFeedback { identity, kind }`. Kinds are `Selected` and `Activated`; no old field or
  alias remains.
- Project feedback only after pending composition publication. `RenderOutcome` returns the exact
  submitted feedback only when that identity projects into the successful frame.
- Encode none/selected/activated as modes 0/1/2 in the existing surface constant. Selected keeps
  its amber transform; Activated uses one fixed green transform in the same resolve. GPU structure
  and resource ownership do not change.
- The prototype owns one capacity-one Enter intent. Reset/Suspended cancel it; fractional, stalled,
  and typed render-blocked work retain it. The next nonzero actor commit resolves the exact retained
  target and applies the fixed inclusive 512-Q9 proximity gate.
- An eligible attempt submits Activated feedback. Only matching projected feedback commits the
  acknowledgement. The successful candidate is frame one of 12; the remaining 11 decrement only on
  successful matching projected frames. Target change or unavailable lifetime clears it.
- F+Enter+W may observe and act on the target in the same committed step.

## Consequences

- Action eligibility derives from the current committed actor output and exact target identity,
  without another spatial selection or copied canonical object state.
- An ineligible committed attempt is consumed without an effect. Malformed resolution/proximity
  failure preserves policy state. Projection loss after eligibility produces no acknowledgement.
- Runtime and renderer retain no product target/action state. Canonical objects remain immutable and
  snapshot-scoped; their identity is still not a persistent gameplay/network identifier.
- Visible records remain 56 bytes, surface constants remain 60 DWORDs, statistics remain 32 bytes,
  descriptors remain 98, and no pass/resource/allocation/copy/readback/synchronization is added.
- This decision does not authorize object mutation, interaction dispatch, inventories, gameplay
  persistence, networking, or Wulin semantics.

## Evidence

Experiment 0103 passes all focused Rust tests. `canonical-frame-v9` passes in 32.302 seconds with
3,472 exact pixels for both Selected and Activated, distinct deterministic color hashes, unchanged
object-ID attachment, replay/clear, and strict invalid-kind/ID rejection.

`canonical-prototype-v21` passes in 72.589 seconds. Native F+Enter+W acquires ID 496 at exact delta
`(160, 0)` Q9 / squared distance 25,600, receives the same activated identity from the successful
frame, commits once with 11 frames remaining, and copies no canonical object content.

The final-worktree `canonical-runtime-v11` passes in 245.223 seconds. Source replacement/departure
suppress feedback; source revisit/return restore the same 3,472 pixels. Rollback, restart, 32+32
traversal, and two lifecycle cycles pass. Four warm and eight measured publications retain 492
handles and 21 threads; private bytes increase 487,424 within the accepted plateau. The report
contains 24 files / 25,346,259 bytes.
