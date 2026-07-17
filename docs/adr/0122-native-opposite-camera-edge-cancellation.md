# ADR 0122: Native Opposite Camera Edge Cancellation

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0119 Native Opposite Camera Edge Cancellation

## Context

`HostInput` unit tests retain both changing edges from one ingest, and the Prototype camera policy
unit tests cancel simultaneous Q/E press edges. Real-process acceptance already proved a single E
edge, duplicate held-E suppression, and checked rejection of an invalid E alias, but it had not
proved the opposite-edge branch through exact Win32 transport. Experiment 0118 established a
bounded atomic window-thread batch that removes message/frame splitting as a confounder.

## Decision

- Maintain one real-process session that atomically queues Q-down, E-down, and W-down after
  orbit-zero readiness, then exits after one bounded delay.
- Use exact camera-relative locomotion as the cancellation oracle: accepted cancellation is
  negative Z at orbit 0; either unpaired edge would select a nonzero orbit and rotate movement.
- Keep edge normalization in the sole `HostInput` and camera cancellation in the existing pure
  Prototype candidate. Add no product input value, history, event stream, controller state, or
  Runtime route.
- Separate shared session process framing from the bounded session-gate matrix now that their
  combined source has reached its enforced size boundary.

## Consequences

- Same-ingest opposite camera edges gain exact adapter-to-product evidence using one measured
  window-thread batch and one directional Walk oracle.
- The acceptance native transport permits delayed Escape after restoring an otherwise atomic key
  batch; the delayed exit is not part of that batch.
- Session framing and gate composition have explicit separate owners without changing the
  product's two-value readiness/completion schema.
- This decision does not authorize key remapping, input telemetry, raw-input handling, replay,
  another controller, product behavior, or Runtime/GPU/resource changes.

## Evidence

Experiment 0119 passed `canonical-prototype-v35` on its first run in 128.648 seconds. PID 7632
queued Q/E/W on exact window thread 1728 with intervals of 0.0012/0.0010 ms and a total span of
0.0022 ms; Escape followed after 238.676 ms.

Completion retained orbit 0 and produced exactly 14 negative-Z Walk steps: delta `(0, -448)` Q9,
clip 1, and yaw 49,152. Clock reset/suspend/resume/stall stayed `1/0/0/0`, Ready/sample advanced
from `2/3` to `39/40`, object policy remained idle, render blocks remained zero, and the two-value
session exited with code zero and empty stderr. `runseal :guard` and every prior Prototype gate
passed without product or engine changes.
