# ADR 0100: Committed Prototype Object Observation

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0097 Committed Prototype Object Observation

## Context

The runtime now provides one exact bounded nearest-object result over the committed snapshot, but
only diagnostic workflows consume it. The first product consumer must establish when an observation
is eligible and which actor position it observes without turning a 25,600-candidate linear scan into
automatic frame work.

A retained target would also require source-replacement lifetime, disappearance, selection
replacement, rendering feedback, and persistent identity decisions. An interaction action would
add eligibility and authoritative effects. Neither boundary follows from spatial proximity alone.

## Decision

- The prototype owns one capacity-one F observation intent and a fixed inclusive 512-Q9 radius.
- Duplicate presses coalesce. The policy owns only a pending bit and retains no object, count, or
  selection.
- Apply the current host elapsed sample before admitting the input: Reset and Suspended cancel the
  intent; Ready and Stalled preserve it.
- Preserve pending intent when no fixed step is emitted and across typed pending-window render
  backpressure.
- After a successful nonzero actor advance, invoke the existing nearest query from the exact
  committed actor output position. Clear the intent only after query success.
- Treat a successful no-candidate response as completion. Treat query failure as fatal without
  pre-consuming policy state.
- Keep the completed origin/result only in same-completion readiness evidence. Add no recurring
  query, retained target, interaction action, engine/host input policy, or alternate object owner.

## Consequences

- The first product-side object discovery is explicitly ordered after actor commit and cannot
  observe a speculative or render-blocked position.
- Work remains demand-driven: no F press means no nearest scan, and one pending intent causes at
  most one successful scan.
- Product code still has no target lifetime, selection, highlight, interaction, or persistent
  identity. Those require separate experiments.
- F currently exposes read-only observation behavior; it does not claim player-facing feedback or
  gameplay effect.

## Evidence

Experiment 0097 passes the focused engine-runtime, prototype, and reference-host tests, including
two pure observation-policy tests. The final `canonical-prototype-v17` run passes in 71.900
seconds. A visible native F+W process commits the actor to local Z `-32` Q9 and then emits that exact
position as the query origin. An independent `.wlr` oracle scans all 25,600 candidates and exactly
matches authored ID 496, delta `(160,0)`, and squared distance 25,600. Ordinary and restarted
processes retain no intent/result; all focused failure, boundary, locomotion, presentation, camera,
jump, traversal, Escape, lifecycle, and cleanup gates remain exact with no residual process.
Repository guard passes with zero Flavor deny issues.
