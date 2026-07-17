# ADR 0127: Native Run Modifier Re-press Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0124 Native Run Modifier Re-press Readmission

## Context

Prototype unit evidence proves gait derives only from current held Shift plus nonzero locomotion,
and live acceptance now proves native Shift release transitions held-W Run to Walk. No real-process
session proves pressing Shift again while W remains held transitions Walk back to Run.

## Decision

- Maintain one real-process session with one visible-window native sequence that posts W-down,
  delayed Shift-down, and delayed Escape in exact order while the acceptance owner reads the
  existing readiness value before awaiting sequence completion.
- Use exact presentation as the transition oracle: readiness is Walk clip 1/yaw 49,152; completion
  retains negative-Z motion but is Run clip 2/yaw 49,152 with a later epoch.
- Keep normalized held state in the sole `HostInput` and gait/presentation selection in the
  existing pure Prototype policies.
- Add no gait state, acceleration, velocity, input history, event stream, product telemetry,
  Runtime route, or renderer/GPU/resource behavior.

## Consequences

- Native Run gains complementary modifier re-press readmission proof after the accepted release
  transition.
- The bounded native ordering plus Walk readiness and Run completion prove the transition without
  deriving a Walk/Run step split from host timing.
- This decision does not authorize configurable gait, sprint stamina, blending, root motion,
  another controller, product behavior, traversal changes, or Runtime/GPU/resource changes.

## Evidence

Experiment 0124 accepted through `canonical-prototype-v40` on its first run in 147.077 seconds.
PID 11268 received W-down/Shift-down/Escape in one exact visible-window sequence with intervals
508.8601 and 207.5121 ms. Readiness was Walk clip 1/yaw 49,152 at local Z `-64`; completion was
Run clip 2/yaw 49,152 at local Z `-1792`, epoch `3 -> 19`, with zero render blocks, unchanged
reset/suspend/resume/stall counts, idle object state, and the exact two-value clean exit.
