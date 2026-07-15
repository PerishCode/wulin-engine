# ADR 0073: Retired Standalone Actor Projection Surface

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0070 Mandatory Actor Projection Cleanup

## Context

Experiment 0066 introduced a public `Runtime::project_actor` forwarding method, crate-root
projection type, `actor.project` diagnostic, and recurring process gate to prove the actor/render
coordinate seam before any GPU consumer existed. Experiment 0068 then made that projection and its
failure preflight part of the sole renderer frame transaction.

No application or engine consumer uses the standalone forwarding path. Keeping it would preserve a
second read-only execution surface and recurring acceptance workload for an intermediate experiment
rather than current runtime behavior.

## Decision

- Remove `Runtime::project_actor` and the crate-root `ActorRenderProjection` export.
- Remove `actor.project`, its protocol/parser/dispatch implementation, and the standalone
  `.runseal/support/actor/projection.ts` gate.
- Remove the projection-only field from canonical runtime acceptance and the deleted support file
  from stable indexes.
- Retain `Renderer::project_actor`, frame preflight, the private projection type, exact signed/Q9
  arithmetic tests, actor upload, and `runseal :canonical-actor` as the sole live consumer and
  focused acceptance path.
- Extend the recurring removal guard so the deleted public method, type export, inspect verb,
  protocol symbols, or support path cannot return as compatibility surfaces.

## Consequences

- Actor projection remains an implementation detail of successful frame admission rather than an
  independently callable runtime transaction.
- Operators can inspect actor lifecycle/simulation state and GPU frame evidence, but cannot invoke
  an obsolete projection-only control.
- Experiment 0066 and ADR 0069 remain decision history; their former API is not maintained as an
  alias or fallback.
- No projection formula, frame order, shader, GPU resource, synchronization, actor behavior,
  prototype policy, or content format changes.

## Evidence

Experiment 0070 records the static inventory, private tests, dependency-free live old-verb
rejection, exact canonical actor replay/capture, and repository guard. The cleanup deletes 293 live
lines before documentation while adding only 11 guard/visibility lines.
