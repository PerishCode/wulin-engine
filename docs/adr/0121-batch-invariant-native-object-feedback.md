# ADR 0121: Batch-Invariant Native Object Feedback

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0118 Batch-Invariant Native Object Feedback

## Context

The maintained Prototype workflow intermittently expected Rejected but received a valid Activated
frame. Atomic native posting reproduced the mismatch, proving that F/Enter/direction message
splitting was not the cause. The first committed simulation transaction may contain `1..=8` fixed
steps; moving across the fixed 256-Q9 object grid can change the nearest target and therefore the
exact facing result. A focus-reset attempt also could not guarantee one emitted step.

## Decision

- Keep product time and the complete `1..=8` simulation batch domain unchanged.
- Maintain one acceptance-only native input transaction that briefly suspends the exact visible
  window thread, queues focus activation plus the complete F/Enter key group, and restores the
  thread in `finally`.
- Prove Activated and Rejected with stationary yaw-zero actors in two deterministic signed source
  windows: base has exact nearest delta `(160, -32)` Q9; `base + 4` has exact nearest delta
  `(-224, -32)` Q9.
- Cook the second fixture and its required traversal center through the existing canonical setup.
- Keep capacity-one evidence at the base fixture. Move only after the first consumption, then
  release D before the exclusion-aware second F/Enter action so its query origin equals final
  position.
- Split acceptance input ownership into native transport, named actions, and composed sequences.

## Consequences

- Object-feedback acceptance is invariant across every allowed first batch without a time reset,
  retry, relaxed threshold, or dynamically accepted feedback kind.
- The native helper gains bounded window-thread suspend/resume evidence, but the product owns no
  thread control, event history, input journal, replay path, or new report.
- The canonical Prototype fixture gains two cooked centers and the workflow gains one stationary
  baseline. Generated sources remain disposable and ignored.
- Product input, object observation/action policy, Runtime, renderer/GPU resources, frame
  transaction, and synchronization remain unchanged.

## Evidence

Experiment 0118 passed `canonical-prototype-v34` in 120.784 seconds. Activated PID 10244 committed
three stationary steps at the base center, selected authored ID 496 at `(160, -32)` Q9, and
submitted/projected Activated. Rejected PID 13120 committed one stationary step at `base + 4`,
selected authored ID 495 at `(-224, -32)` Q9, and submitted/projected Rejected. Both F/Enter
batches spanned 0.0012 ms on their exact window threads.

The sustained session retained one consumed identity and one exclusion-aware capacity rejection,
with 12 Rejected and 87 suppression frames after a 1040.276 ms D motion that ended before the
second action. `runseal :guard`, all prior Prototype sessions, source/traversal oracles, restart,
and Sidecar cleanup passed without product or engine changes.
