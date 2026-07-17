# ADR 0144: Native Focus Locomotion Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0141 Native Focus Locomotion Readmission

## Context

The maintained focus-discontinuity session proved that pre-loss held W and same-batch Jump/object
edges do not survive into resumed simulation, but its exact unchanged-actor oracle did not prove
that later locomotion remains usable. A discontinuity boundary must reject stale input without
poisoning future direction-key admission.

## Decision

- Retain the atomic Space/F/Enter/W/focus-loss batch and exact suspend/resume/reset contract.
- After recovery, atomically post a fresh A-down on the same PID/window, hold it for at least
  250 ms, post A-up, and wait at least 250 ms before the same helper posts Escape.
- Require exact negative-X displacement in 32-Q9 Walk units and exact zero Z displacement.
- Require final Survey clip 0 to retain the committed left yaw 32,768.
- Keep object policies idle and preserve all earlier same-batch action-suppression evidence.
- Add no product input state, action/controller, event queue, report field, compatibility path,
  process, Runtime behavior, or renderer/GPU/source/resource/synchronization ownership.

## Consequences

- The real process now proves both stale forward-input suppression and fresh orthogonal-locomotion
  readmission across one focus discontinuity.
- Direction release is proven by the final Survey transition rather than inferred from timing.
- Product behavior and the exact two-value session contract remain unchanged; only the maintained
  acceptance workload and oracle become stronger.

## Evidence

`canonical-prototype-v56` passed on its first run in 180.909 seconds with a 442,700-byte report.
PID 9148/thread 24044 received the exact pre-loss batch and later A-down/A-up/Escape sequence.
A was held for 272.3119 ms and followed by 261.9928 ms of stationary work. The actor committed 16
exact Walk steps from `(0,0)` to `(-512,0)` Q9, retained its generation/region, finished with zero
vertical velocity as Survey/yaw 32,768, and advanced animation epoch `1 -> 234`.

The clock recorded exactly one suspend, one resume, and one post-resume reset; suspended samples
advanced `0 -> 76`, Ready/sample counts advanced `2/3 -> 171/249`, and stall/render-block counts
remained zero. Object policies stayed idle and the process emitted exactly two values. All 103
engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor remained at zero denies
and five existing warnings.
