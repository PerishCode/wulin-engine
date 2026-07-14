# ADR 0043: Runtime Frame Transaction

- Status: Accepted
- Date: 2026-07-14
- Supersedes: ADR 0040
- Superseded by: None

## Context

ADR 0040 established deterministic integer source-duration presentation time, but its mutable
tick, running state, counters, controls, and automatic advancement lived inside the skeletal
renderer. Experiment 0039 subsequently promoted one engine `Runtime` owner. Leaving time mutation
inside rendering would force later update, replay, and simulation work to coordinate with a
renderer-internal decision or create another clock.

Experiment 0040 tests the narrower prerequisite before choosing any elapsed-time or simulation
policy: make the runtime own the accepted timeline and define when a successful frame commits it.

## Decision

- `Runtime` owns one `PresentationTimeline`: active tick, running state, automatic advance count,
  manual step count, and wrap count. Rendering modules contain no mutable timeline or
  pause/resume/set/step authority.
- A runtime frame samples the current tick and, for probe frames, its complete status snapshot.
  One immutable `RenderFrame` carries that evidence through renderer, skeletal constants, surface,
  and CPU/GPU probes.
- Capture and probe observe the sampled pre-commit tick. After the renderer returns success,
  `Runtime` commits one automatic advance only when canonical composition is enabled. Idle-shell
  and failed frames do not advance.
- Canonical presentation time retains 4,800 integer units per nominal second, an 80-unit frame
  quantum, and the exact 31,002,560-frame common period. Source duration still selects phase as
  `floor(((frame * 80) mod duration) * 64 / duration)` plus authored phase offset.
- `canonical.time.status`, `pause`, `resume`, `set`, and `step` retain their fields, bounds,
  paused-only mutation rules, invalid-request rollback, and process-local persistence across
  source changes, composition, traversal, prefetch, and rollover.
- Workbench host pause remains an operator scheduling state: it may skip ordinary runtime frames
  while allowing requested capture/probe frames. It is not the canonical presentation pause.
- Repository validation rejects renderer-owned timeline state or controls and requires one
  `PresentationTimeline` owner under `crates/engine-runtime`.

## Consequences

- The renderer is a temporal consumer rather than the authority deciding whether a frame advances.
  A later experiment can define engine update or replay scheduling at the runtime boundary without
  reaching into rendering internals.
- Exact source-duration animation, controls, hashes, submissions, resources, and lifecycle
  behavior remain unchanged.
- The timeline is still frame-driven. This decision does not define elapsed wall time, display
  pacing, interpolation, fixed or variable simulation steps, gameplay/network clocks, input
  sampling, root motion, or state transitions.

## Evidence

Experiment 0040 passed the 598.8-second direct workflow with exact pre-migration attachment and
shadow hashes, source-duration phases 0/63/0/0 at ticks 0/42/43/85, all source/failure/hold gates,
32 reactive and 32 prepared crossings, zero transient handle growth, and 16 complete lifecycle
cycles. All 21 engine-runtime tests and the repository guard passed.

Generated evidence is ignored under
`out/captures/0040-runtime-frame-transaction/`.
