# ADR 0085: Plain Prototype v0 Stage Boundary

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0082 Plain Prototype v0 Stage Seal

## Context

The project has advanced through narrowly accepted runtime, host, simulation, actor, camera,
presentation, traversal, and operator boundaries. The resulting application is already a coherent
finite single-actor loop, but the repository describes it only as an accumulation of experiments.
Continuing to add conspicuous features before naming this boundary would blur the difference
between a proven engine skeleton and the next gameplay/source stage.

The application deliberately has no inspect endpoint. Adding telemetry solely to claim sustained
prototype traversal would weaken that product boundary. Expanding the deterministic offline
sandbox would likewise conceal, rather than solve, the absence of a source service or an explicit
finite-world edge policy.

## Decision

- Seal the current application/runtime composition as plain Prototype v0 after the product
  lifecycle, focused prototype gate, long canonical-runtime checkpoint, and merge guard pass.
- Define Prototype v0 narrowly: strict canonical cold start; native input and bounded host time;
  one grounded capacity-one actor; fixed gravity and W/A/S/D displacement; transactional
  Survey/Walk, facing, and actor-local phase; one actor-relative camera; and engine-owned traversal
  activated once with prefetch disabled.
- Keep `runseal :prototype` as the sole manual product operator and `runseal :canonical-prototype`
  as its focused acceptance owner. Add no aggregate stage wrapper or application diagnostic route.
- Treat the zero-origin `[-8,8]²` sandbox as an explicit finite source horizon. Prototype v0 does
  not claim sustained product traversal, boundary behavior, a streaming service, gameplay
  interaction, multiple actors, networking, or Wulin content.
- Begin the next stage only from a separately audited finite-world or gameplay dependency; the v0
  seal does not choose that policy.

## Consequences

- The repository gains an honest milestone for the plain engine skeleton without changing the
  product or inflating the accepted capability surface.
- One long canonical rerun is justified at the stage boundary, while ordinary experiments retain
  focused iteration and one merge-checkpoint guard.
- Future work can change stage rather than silently extending Prototype v0; its first experiment
  must state which excluded dependency it makes real.

## Evidence

Experiment 0082 passed a source-free product lifecycle in 13.3 seconds with exact deterministic
source/config hashes, PID-replacing restart, and zero-process stop. `canonical-prototype-v8` passed
in 36.748 seconds. The stage-only `canonical-runtime-v1` checkpoint passed in 744.8 seconds with
32 reactive plus 32 prepared crossings, a 64-publication bounded resource sweep, and 16 complete
lifecycle cycles. The merge guard passed with zero Flavor denies.
