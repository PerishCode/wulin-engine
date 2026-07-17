# ADR 0124: Native Counter-Clockwise Camera Wrap

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0121 Native Counter-Clockwise Camera Wrap

## Context

Prototype camera-policy tests prove Q counter-clockwise wrap from orbit 0 to orbit 3 and locomotion
tests prove orbit-3 W maps to positive world X. Real-process acceptance already proves clockwise E,
held-E repeat suppression, full-value invalid-key rejection, and simultaneous Q/E cancellation,
but it does not prove the remaining Q-only wrap branch through exact Win32 transport.

## Decision

- Maintain one real-process session that atomically queues Q-down and W-down after orbit-zero
  readiness, then exits after one bounded delay.
- Use exact camera-relative locomotion as the wrap oracle: accepted orbit 3 is positive X at zero Z
  with Walk yaw 0.
- Keep native normalization in the sole `HostInput`, wrap arithmetic in the existing pure Prototype
  camera candidate, and rotation in the existing locomotion policy.
- Add no product input value, history, event stream, controller state, completion camera telemetry,
  Runtime route, or renderer/GPU/resource behavior.

## Consequences

- The complementary native counter-clockwise branch gains exact adapter-to-product evidence through
  one measured window-thread batch and one directional Walk oracle.
- The bounded invariant owns only acceptance evidence and does not expand the already saturated
  general camera evidence file.
- This decision does not authorize key remapping, arbitrary camera angles, pointer/gamepad input,
  raw-input handling, replay, another controller, product behavior, traversal changes, or
  Runtime/GPU/resource changes.

## Evidence

Experiment 0121 passed `canonical-prototype-v37` in 139.716 seconds. PID 6788 queued Q/W on exact
window thread 18860 with a 0.0011 ms interval and total atomic span; Escape followed after
232.1975 ms.

Completion produced exactly 14 positive-X Walk steps: delta `(448, 0)` Q9, clip 1, yaw 0, and
animation epoch `1 -> 35`. Clock reset/suspend/resume/stall stayed `1/0/0/0`, Ready/sample advanced
from `2/3` to `40/41`, object policy remained idle, render blocks remained zero, and the two-value
session exited with code zero and empty stderr. All previous Prototype gates, Deno checks, init,
and the repository guard passed without product or engine changes.
