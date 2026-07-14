# ADR 0040: Source-Duration Presentation Time

- Status: Superseded
- Date: 2026-07-14
- Supersedes: ADR 0036
- Superseded by: ADR 0043

## Context

ADR 0036 introduced one deterministic renderer-owned clock, but reduced time modulo the fixed 64
sample slots. Experiment 0036 subsequently cooked the pinned Fox source durations while leaving
them unused, so Survey, Walk, and Run all looped once every 64 submitted frames regardless of their
authored duration.

Experiment 0037 tests whether source duration can affect sampled phase without adding wall-clock
ownership, float shader time, content mutation, runtime asset lookup, or a second animation path.

## Decision

- Canonical presentation time uses 4,800 integer units per nominal presentation second and
  advances 80 units after each submitted canonical frame. It remains frame-driven; elapsed wall
  time cannot advance it.
- Pinned Survey, Walk, and Run durations are verified as 16,400, 3,400, and 5,560 units. Fixture
  clips retain the previous 5,120-unit/64-frame duration.
- The renderer clock is a frame in `0..31,002,560`. This is the exact common period of the fixture
  and imported durations at an 80-unit frame quantum, and its 2,480,204,800-unit product fits in
  `u32`.
- Live sampled phase is the authored phase offset plus
  `floor(((frame * 80) mod duration) * 64 / duration)`, modulo 64. Rig and authored clip select the
  duration but remain content authority; time cannot choose either.
- The CPU oracle and GPU cull/pose work use the same integer formula. Three imported source
  durations occupy an aligned four-DWORD root-constant block; fixed clip slots select their source
  duration by the verified `clip % 3` alias. The skeletal root constants total 60 DWORDs, keeping
  the complete root signature at 61 DWORDs with its descriptor table.
- `canonical.time.status`, `pause`, `resume`, `set`, and `step` remain the complete deterministic
  operator surface. Set accepts frames below the common period; step accepts `1..=4096` while
  paused. Invalid requests do not mutate state.
- Probe and capture observe the pre-advance frame. Clock state persists across source changes,
  composition, traversal, prefetch, and rollover; those operations neither wait on nor reset it.

## Consequences

- Authored Walk now alternates 43- and 42-frame sampled loops, averaging its exact 42.5-frame
  source duration at the fixed quantum. Survey and Run use the same bounded mechanism.
- Fixture phase behavior remains exact, but global clock frame 64 no longer wraps. Exact complete
  state repetition occurs at the declared common period.
- No GPU resource, descriptor, palette allocation, content copy, publication transaction, or
  indirect dispatch is added.
- This decision still does not define display-rate pacing, wall-clock interpolation,
  gameplay/network synchronization, clip blending, state transitions, root motion, compression,
  or dynamic animation assets.

## Evidence

Experiment 0037 passed the direct 539.4-second workflow. Controlled Walk frames 0/42/43/85 selected
GPU phases 0/63/0/0; frames 43 and 85 reproduced frame 0 color, PNG, object-ID, surface, and palette
evidence exactly. The maximum CPU/GPU palette delta was `2.3283064e-10`.

The exact common-period wrap, automatic time, held-pair continuity, all source/failure/traversal
regressions, a 64-publication resource plateau, and 16 lifecycle cycles passed. Generated evidence
is ignored under `out/captures/0037-source-duration-playback/`.
