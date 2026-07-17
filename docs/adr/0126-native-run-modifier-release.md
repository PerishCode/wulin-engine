# ADR 0126: Native Run Modifier Release

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0123 Native Run Modifier Release

## Context

Prototype acceptance proves native held Shift+W selects exact forward Run at readiness. Unit
evidence proves gait derives only from current held Shift plus nonzero locomotion, but no live
session proves that releasing Shift clears only the modifier while held W continues as Walk.

## Decision

- Maintain one real-process session with one visible-window native sequence that posts Shift-down,
  W-down, delayed Shift-up, and delayed Escape in exact order while the acceptance owner reads the
  existing readiness value before awaiting the sequence completion.
- Use exact presentation as the transition oracle: readiness is Run clip 2/yaw 49,152; completion
  retains negative-Z motion but is Walk clip 1/yaw 49,152 with a later epoch.
- Queue the existing focus-discontinuity W-down/focus-loss batch while its exact window thread is
  suspended so the maintained unchanged-actor gate cannot sample between those messages.
- Keep normalized held state in the sole `HostInput` and gait/presentation selection in the
  existing pure Prototype policies.
- Add no gait state, acceleration, velocity, input history, event stream, product telemetry,
  Runtime route, or renderer/GPU/resource behavior.

## Consequences

- Native Run gains its complementary modifier-release-to-Walk session proof.
- The bounded native ordering plus Run readiness and Walk completion prove the release transition
  without deriving a Run/Walk step split from host timing.
- Acceptance helper launch time can no longer drive this session into the finite boundary before
  modifier release, and the focus-discontinuity gate retains its exact actor-state assertion.
- This decision does not authorize configurable gait, sprint stamina, blending, root motion,
  another controller, product behavior, traversal changes, or Runtime/GPU/resource changes.

## Evidence

Experiment 0123 accepted through `canonical-prototype-v39` in 142.711 seconds. PID 22072 received
Shift-down/W-down/Shift-up/Escape in one exact visible-window sequence with intervals 4.6121,
505.9149, and 208.2274 ms. Readiness was Run clip 2/yaw 49,152 at local Z `-192`; completion was
Walk clip 1/yaw 49,152 at local Z `-2304`, epoch `3 -> 19`, with zero render blocks, unchanged
reset/suspend/resume/stall counts, idle object state, and the exact two-value clean exit.
