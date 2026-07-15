# ADR 0080: Transactional Actor Presentation Command

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0077 Transactional Locomotion Presentation

## Context

The retained runtime actor already owned schema-3 presentation, but its sole simulation transaction
accepted only scalar motion arguments and replaced only motion. The prototype consequently spawned
the imported Fox permanently in Walk, including while stationary. Adding an independent
presentation setter would allow animation state to commit before motion was rejected by a pending
render window, splitting one locomotion decision across two authorities.

Presentation also cannot change on an arbitrary fractional host sample. The rational simulation
schedule must decide when a gameplay command becomes an emitted fixed step so elapsed partitioning
does not alter actor state.

## Decision

- `Runtime::advance_simulation_actor` accepts one `ActorSimulationCommand` containing horizontal
  displacement, step-up limit, vertical acceleration, and a complete `ActorPresentation`. The prior
  scalar signature is removed.
- Command presentation validates before schedule or terrain-query preparation. A zero-step advance
  preserves the complete actor even when the desired presentation differs.
- A nonzero-step advance builds one complete actor candidate with prepared motion and desired
  presentation. Existing published/pending render admission runs before the retained actor and
  schedule commit together. Fatal failure or typed pending block commits neither.
- The capacity-one slot replaces the complete validated actor while preserving its exact handle.
  The motion-only `ActorMotionBatch` name is replaced by `ActorStateTransition`.
- The existing `simulation.actor.advance` diagnostic payload is replaced in place with required
  presentation fields and its response becomes schema 3. No optional old payload or parser fallback
  remains.
- Prototype policy owns fixed clip choice: initial and zero-displacement commands use imported
  Survey clip 0; nonzero W/A/S/D commands use Walk clip 1. Archetype 7, material 63, yaw 0, phase 0,
  and variation 0 remain fixed.

## Consequences

- Locomotion motion and animation selection now commit or roll back as one actor state transition.
- Fractional sampling remains partition-safe and presentation validation cannot consume schedule or
  terrain-query work.
- The runtime accepts caller-authored presentation but does not infer gameplay state from movement.
- Directional yaw, Run selection, blending, state-machine transitions, root motion, traversal
  prefetch, sustained crossing, multi-actor storage, and Wulin policy remain later experiments.
- Renderer, shaders, actor GPU record/resources, synchronization, source formats, and presentation
  clock ownership are unchanged.

## Evidence

Experiment 0077 records `canonical-actor-v3` passing in 30.932 seconds with invalid and fractional
rollback, exact Survey-to-Walk admitted commit, exact blocked Walk retention, pending/frame
stability, and existing GPU actor assertions. `canonical-prototype-v6` passed in 38.080 seconds:
two stationary processes retained clip 0 and the process-qualified W process committed clip 1 with
exact movement, camera, traversal, zero-block, restart, failure, and cleanup evidence.
