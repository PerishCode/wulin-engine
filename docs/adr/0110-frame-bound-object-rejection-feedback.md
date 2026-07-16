# ADR 0110: Frame-Bound Object Rejection Feedback

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0107 Rejected Object Action Feedback

## Context

ADR 0109 rejects an in-radius side or rear object action as typed `OutsideFacing`. Its frame remains
amber Selected, exactly like no action, so the accepted rejection has no distinct product-visible
result. The existing frame feedback transaction and capacity-one 12-frame acknowledgement already
provide the necessary identity, projection, rollback, and bounded lifetime.

## Decision

- `ObjectTargetFeedbackKind` has exactly three values: amber `Selected`, green `Activated`, and red
  `Rejected`.
- Only a resolved, in-radius `OutsideFacing` Prototype attempt produces Rejected feedback. Missing,
  unavailable, malformed, and out-of-radius outcomes remain feedback-free.
- The rejected qualified identity is submitted through the existing immutable frame candidate.
  Exact returned projection starts the existing acknowledgement with 11 frames remaining; later
  frames decrement only when the same identity and kind are projected.
- Rejected completion is always `applied=false`. It increments the existing ineligible count and
  never commits consumption, exclusion, suppression, canonical mutation, or another action result.
- The existing surface resolve maps the third fixed kind to the exact CPU-oracled red mix
  `color * 0.30 + (1.0, 0.12, 0.08) * 0.70`.

## Consequences

- Passive selection, successful activation, and exact facing rejection are visually distinct while
  sharing one qualified identity and one frame transaction.
- There is no second timer, queue, registry, product action state, renderer lifetime, pass,
  resource, descriptor, copy, readback, or synchronization path.
- This decision does not define feedback for general failures or authorize mutation, inventory,
  rewards, dispatch, respawn, persistence, networking, or Wulin semantics.

## Evidence

Ten focused policy tests cover rejected construction, exact projection, 12-frame countdown,
unprojected completion, rollback, and unchanged consumption. `canonical-frame-v11` passes in
45.636 seconds: all three kinds cover the same 3,472 pixels of ID 987, produce distinct stable
color hashes, preserve the exact object-ID attachment, replay immediately, and clear exactly.

`canonical-prototype-v24` passes in 77.910 seconds. Its native side-facing ID 496 attempt reports
delta `(160,0)` Q9, distance 25,600 Q18, yaw/direction/dot `49,152 / (0,-1) / 0`, exact submitted
and projected Rejected feedback, `applied=false`, 11 remaining frames, zero consumption/exclusion,
and one rejected frame.

`runseal :guard` passes with zero Flavor denies after the Workbench three-kind static contract and
interaction report boundary were made explicit. The final-worktree `canonical-runtime-v15` passes
in 257.299 seconds with all source/window, rollback, restart, 32+32 traversal, resource, and
lifecycle gates unchanged. Five warm/eight measured publications retain 492 handles and 21
threads; private bytes change by -49,152. The report retains 24 files / 25,346,275 bytes and records
980 Sidecar invocations.
