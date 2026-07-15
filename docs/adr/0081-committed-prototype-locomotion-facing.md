# ADR 0081: Committed Prototype Locomotion Facing

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0078 Committed Locomotion Facing

## Context

Experiment 0077 made locomotion clip and motion one actor transaction, but prototype presentation
still fixes yaw at zero. Directional yaw is already a validated schema-3/GPU field, and the
normalized command has exact cardinal or diagonal signs. The missing boundary is application
policy state: when input becomes stationary, facing should retain the last direction that actually
committed, not reset to an arbitrary axis or move ahead after fractional/blocked work.

Clip start phase is a separate animation-transition/time question. Sustained traversal already
progresses inside the renderer and has no missing application callback. Neither is bundled here.

## Decision

- The prototype owns one committed-facing policy initialized to yaw 0. It is command-authoring
  state, not another actor snapshot or runtime readback.
- Nonzero normalized displacement maps from signs to eight exact Q16 headings using imported Fox
  local +X as forward. Stationary displacement reuses the committed heading.
- The policy observes only advanced actor output with at least one emitted fixed step. Zero-step
  advance does not update it; a render-blocked outcome has no advanced output to observe.
- The existing complete motion/presentation command carries the selected yaw through the accepted
  actor/schedule admission and commit. No second presentation mutation is added.
- Survey/Walk, material/archetype, phase/variation, input magnitude, camera, traversal, runtime,
  renderer, GPU, synchronization, and format behavior remain unchanged.

## Consequences

- The prototype actor faces its committed motion direction and remains facing that direction when
  locomotion stops.
- Fractional and pending-window outcomes cannot desynchronize application-facing state from the
  retained runtime actor.
- The policy creates no actor readback or duplicate complete actor authority.
- Animation transition timing, blending, root motion, Run, analog steering, camera rotation, jump,
  and Wulin gameplay remain later decisions.

## Evidence

Experiment 0078 records all eight exact policy headings and zero/nonzero observation tests.
`canonical-prototype-v7` passed in 38.726 seconds: stationary processes retained yaw/clip `0/0`,
while the native-W process atomically committed yaw/clip `49152/1` and Z `-32` in one step/query
with zero blocks and unchanged camera, traversal, restart, failure, and cleanup evidence.
