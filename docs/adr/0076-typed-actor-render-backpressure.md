# ADR 0076: Typed Actor Render Backpressure

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0073 Typed Actor Render Backpressure

## Context

Canonical actor simulation now prepares copied schedule/motion state and preflights the complete
candidate before dual commit. A candidate can pass the published window yet miss a non-prefetch
pending window while preserving authoritative state, but the renderer exposed that expected
pressure through the same untyped error channel used for published-window, identity, terrain,
arithmetic, configuration, and renderer failures. Prototype propagated every such error and
terminated before it could render the retained actor.

Matching error strings in the application would make renderer wording an implicit control API and
could accidentally swallow genuine failures. The recoverable condition must be classified where
active-window membership is known.

## Decision

- Renderer preflight returns a private admitted/active-blocked/pending-blocked result. Missing
  composition, config divergence, projection arithmetic, and all other structural failures remain
  errors.
- Frame callers require admission and convert active/pending blocks to their established exact
  errors. Frame safety and the sole projection path are unchanged.
- `Runtime::advance_simulation_actor` returns `ActorSimulationOutcome`: either the committed
  `Advanced` value or `RenderBlocked` with prepared step/query counts. Only a pending block after
  published admission becomes the typed outcome; a published active block remains an error. A block
  occurs after complete copied preparation and before actor or schedule mutation.
- Workbench schema 2 reports the same two outcomes with explicit prepared-work and commit counts.
  Schema 1 is removed rather than retained as an alias.
- Prototype consumes a blocked Ready sample once, increments a checked counter, performs no retry
  or elapsed accumulation, and continues its existing camera/frame order with the retained actor.
  Its normal all-zero horizontal policy cannot currently trigger a block.

## Consequences

- Expected pending-publication pressure is a typed nonfatal application outcome without weakening
  error handling for published admission, stale identity, elapsed bounds, terrain, motion
  arithmetic, projection structure, or frame execution.
- Actor and simulation time pause for a blocked sample while HostClock keeps its admitted baseline;
  no catch-up burst or hidden command substitution is introduced.
- Horizontal input and camera-driven traversal can now be evaluated over an explicit backpressure
  boundary, but neither is accepted by this decision.
- No shader, pass, GPU resource, upload, copy, barrier, fence, wait, source format, or alternate
  runtime path changes.

## Evidence

Experiment 0073 records private classification/error-identity tests, schema-2 advanced and blocked
workbench evidence, exact actor/schedule/pending rollback, a retained successful frame, prototype
block-consumption tests, direct restart equality, and lifecycle cleanup. `runseal :canonical-actor`
passed in 36.740 seconds, `runseal :canonical-prototype` passed in 33.457 seconds, and the repository
guard passed with zero Flavor denies.
