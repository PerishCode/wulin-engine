# ADR 0086: Explicit Playable Region Boundary

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0083 Explicit Playable Region Boundary

## Context

Prototype v0 deliberately uses a finite, self-contained source horizon. Its bootstrap selects
sources and an initial composition target but declares no playable extent. Normal held locomotion
can therefore outlive available traversal data: traversal retains the last valid publication after
a missing request, and the actor eventually reaches terrain outside that window through the
runtime's correctly strict fatal query path.

The source index cannot safely define product space because a signed pack may be sparse. Expanding
the operator's cooked horizon only delays the same failure, while converting runtime source/query
failure into collision would hide missing data and weaken accepted transaction semantics. The
finite-world edge is therefore application policy whose authored authority must arrive through the
mandatory bootstrap document.

## Decision

- Replace bootstrap schema 1 directly with strict schema 2. Add one inclusive signed global-region
  `playableRegionBounds` rectangle with `minimum` and `maximum` coordinates; require ordered axes
  and containment of `globalCenter`. Retain no schema-1 decoder or optional fallback.
- `reference-host::bootstrap::Plan` owns validation, exposes the rectangle to its application, and
  includes it in pending/ready startup evidence. It does not validate the rectangle against pack
  indexes or pass it into the engine runtime.
- Before the existing simulation transaction, the prototype reads its current retained actor and
  evaluates the requested X and Z axes independently over
  `SIMULATION_MAX_STEPS_PER_ADVANCE`. Set an axis to zero exactly when that maximum candidate would
  leave the rectangle. Build presentation from the admitted command and continue gravity,
  schedule, camera, rendering, and transaction execution normally.
- The maintained product operator owns conservative inclusive `[-6,6]²` bounds inside its cooked
  `[-8,8]²` centers. Focused canonical documents declare only the regions they exercise.
- Preserve strict runtime missing-source, traversal, terrain-query, published-window, and rollback
  semantics. Add no engine boundary/collision mode, source-index inference, compatibility surface,
  product inspect command, or periodic telemetry.

## Consequences

- Sustained prototype locomotion gains an explicit finite edge without presenting a sparse source
  pack as an authored world or weakening the engine's failure contracts.
- Boundary response is deliberately conservative by at most one maximum fixed-step batch. Each
  axis remains independent, so safe sliding, gravity, presentation, and simulation time continue.
- Bootstrap schema 2 is a breaking repository-controlled replacement. All maintained document
  writers change together and old generated bootstrap files must be regenerated.
- This decision does not provide an infinite source service, arbitrary gameplay volumes, collision
  geometry, world wrapping, multiple actors, Wulin space, or a general engine boundary system.

## Evidence

Experiment 0083 reproduced the pre-change strict failure in 11.282 seconds. It then passed 23
reference-host, 16 prototype, and 77 engine-runtime tests plus `canonical-prototype-v9` in 71.271
seconds. The focused real process received explicit activation and held W, remained live for
15,002.745 ms with empty stderr, and was terminated only by evidence cleanup. The maintained
operator emitted schema 2 with `[-6,6]²` bounds and completed a live start/zero-process stop. The
merge-checkpoint guard passed with zero Flavor denies.
