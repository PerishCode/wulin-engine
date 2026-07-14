# ADR 0036: Deterministic Temporal Presentation

- Status: Superseded
- Date: 2026-07-14
- Supersedes: None
- Superseded by: ADR 0040

## Context

Experiment 0032 made animation selection explicit schema-3 presentation authority, but
the live runtime kept its animation time tick fixed at zero. Rendered objects therefore
held one valid authored pose indefinitely even though the GPU skeletal and surface paths
already supported phase evaluation.

Experiment 0033 tests the narrow temporal prerequisite before external asset work: make
presentation visibly advance while preserving deterministic evidence and keeping time
independent from content I/O, residency, and atomic publication.

## Decision

- The skeletal renderer owns the sole canonical presentation clock. Its tick is modulo
  the fixed 64-sample animation catalog and the process starts at tick 0 in running state.
- A running clock advances by one tick after each submitted canonical frame. Probe and
  capture evidence for a frame observe the same pre-advance tick. Idle-shell frames do
  not advance canonical presentation time.
- `canonical.time.status`, `pause`, `resume`, `set`, and `step` are the complete operator
  surface. Set accepts `0..63`; step accepts `1..=4096`; both require a paused clock and
  reject invalid requests without mutation.
- The live animation phase is the authored phase offset plus the runtime tick modulo 64.
  The clock cannot choose or derive animation enablement, clip, phase offset, variation,
  archetype, material, or yaw.
- Clock state persists across source switches, pair publications, traversal, prefetch,
  and rollover within one process. Content scheduling, I/O, copy, failure, and
  publication neither wait on nor reset it.
- Deterministic acceptance and capture workflows explicitly pause and set the tick.
  Frame-rate-independent interpolation, wall-clock ownership, gameplay/network clocks,
  root motion, and asset import are not part of this decision.

## Consequences

- The synthetic canonical scene now changes naturally across rendered frames while
  retaining exact pause/set/step reproduction and 64-tick wraparound.
- An incomplete terrain/object transaction cannot freeze presentation or expose a half;
  the old complete pair continues animating until the new complete pair publishes.
- Presentation speed is currently frame-count based. A later timing experiment must
  supersede this boundary before introducing wall-clock pacing or network synchronization.
- Stable keys and content namespaces remain identity/cache inputs only; temporal state
  adds no GPU page, source variant, compatibility path, or publication transaction.

## Evidence

Experiment 0033 passed the direct 449.1-second workflow with exact tick 0/1/64 control,
11 automatic advances, invalid-request rollback, animated held-pair continuity, unchanged
content reports during time-only changes, all prior source/failure/traversal gates, a
64-publication resource plateau, and 16 complete lifecycle cycles. The ignored report is
generated at
`out/captures/0033-deterministic-temporal-presentation/acceptance.json`.
