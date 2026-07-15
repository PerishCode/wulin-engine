# ADR 0051: Caller-Owned Fixed Terrain Motion

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0049 accepted exact caller-owned vertical terrain contact but intentionally excluded velocity,
gravity, time, and grounded policy. ADR 0050 then accepted one explicit rational 60 Hz schedule
without a consumer or live clock. Selecting a host clock next would drive only diagnostic counters
and prematurely choose stall/focus policy; selecting actor storage would combine motion with later
identity and lifetime work.

Experiment 0048 therefore tests the first spatial consumer as a caller-owned single-tick value
transaction. It can prove the integration/contact policy before either a host or actor collection
depends on it.

## Decision

- Define caller-owned vertical motion as an accepted `TerrainBody` plus signed Q16 height velocity
  per fixed simulation step. Acceleration is a caller-supplied signed Q16 delta per step squared.
- One transaction represents exactly one accepted 60 Hz step and uses checked semi-implicit Euler:
  update velocity, update center with that velocity, query committed terrain once, then apply the
  existing exact contact resolver.
- Predicted touching or penetration with non-positive velocity is grounded and returns zero
  vertical velocity. Positive velocity is preserved; separated bodies are never snapped downward.
- The runtime returns the complete input, predicted body, contact, output motion, grounded flag,
  fixed rate, and denominator evidence but retains no body or motion state.
- The step does not advance the schedule, sample time/input, iterate a batch, render, touch the GPU,
  or choose a gravity/jump constant. Callers explicitly apply it once per due schedule step.
- Live clock policy, horizontal motion, slopes, actors, and gameplay tuning remain later decisions.

## Consequences

- Fixed-step gravity and jump experiments can use one deterministic vertical transaction without
  coupling to frame rate or actor storage.
- Velocity units are deliberately tied to the accepted tick. Any later change in simulation rate
  requires a new decision rather than an implicit rescale.
- Exact grounded behavior has no tolerance or support hysteresis. Those policies require direct
  evidence if later gameplay needs them.
- Because the implementation is caller-invoked CPU-only code outside frame/GPU/lifecycle paths,
  focused tests, a short process gate, and repository guard are proportionate acceptance evidence.

## Evidence

Experiment 0048 passed 33 focused tests and a 31-second real-process gate. Two equal one-second
schedule partitions produced the same 60 motion/contact steps with SHA-256
`ac02bf0bc3ac0229d5663b566d7d4b0524640318cbd7158b8294614aa8dc6856` and final-state SHA-256
`fd2d79e9ce175e8702a0a91079a4213cbb009edcd8d714815687b9f6dd371fb5` across a process restart.
The controlled body first grounded on tick 19, remained grounded for 42 results, and ended at the
exact terrain-supported center with zero velocity. Failure cases were explicit; every successful
step reported zero allocation, I/O, GPU, fence, synchronization, schedule mutation, and
presentation mutation. `runseal :guard` passed.

The canonical frame, renderer, GPU resources, and lifecycle paths did not change, so the long
workflow was not repeated solely to restate Experiment 0047 evidence. The accepted wrapper now
contains the terrain-motion gate for the next change that requires a full canonical run.
