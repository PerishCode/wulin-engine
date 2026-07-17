# ADR 0125: Native Camera Re-Press Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0122 Native Camera Re-Press Readmission

## Context

Real-process acceptance proves initial native E admission and suppresses duplicate E-down while the
key remains held. Unit evidence proves release/press edge normalization and the four-state camera
cycle, but no live process proves that releasing held E and pressing it again creates a second
camera action or reaches orbit 2.

## Decision

- Maintain one real-process session that begins with the accepted held-E/orbit-1 readiness, then
  atomically queues E-up, E-down, and W-down before a bounded delayed exit.
- Use exact camera-relative locomotion as the re-admission oracle: accepted orbit 2 is positive Z
  at zero X with Walk yaw 16,384. If release or re-press is lost, orbit 1 instead produces negative
  X.
- Keep edge normalization in the sole `HostInput`, camera state in the existing pure Prototype
  candidate/commit policy, and rotation in the existing locomotion policy.
- Add no product input value, history, event stream, controller state, completion camera telemetry,
  Runtime route, or renderer/GPU/resource behavior.

## Consequences

- Held-key duplicate suppression gains its complementary live release/re-press readmission proof.
- Orbit 2 gains a directional real-process locomotion witness without a product report change.
- This decision does not authorize key remapping, arbitrary camera angles, pointer/gamepad input,
  raw-input handling, replay, another controller, product behavior, traversal changes, or
  Runtime/GPU/resource changes.

## Evidence

Experiment 0122 passed `canonical-prototype-v38` in 138.736 seconds. PID 1752 published orbit-1
readiness after startup E-down, then queued E-up/E-down/W on exact window thread 18524 with
intervals of 0.0016/0.0010 ms and a total atomic span of 0.0026 ms; Escape followed after
211.8739 ms.

Completion produced exactly 13 positive-Z Walk steps: delta `(0, 416)` Q9, clip 1, yaw 16,384, and
animation epoch `1 -> 35`. Clock reset/suspend/resume/stall stayed `1/0/0/0`, Ready/sample advanced
from `2/3` to `40/41`, object policy remained idle, render blocks remained zero, and the two-value
session exited with code zero and empty stderr. All previous Prototype gates, Deno checks, init,
and the repository guard passed without product or engine changes.
