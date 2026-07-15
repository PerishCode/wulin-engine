# ADR 0069: Bounded Actor Render Projection

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The runtime now owns one exact actor, while the canonical renderer consumes only immutable
source-resident region pages. Direct GPU binding would otherwise have to invent coordinate,
window-admission, vertical-position, and rollover policy at the same time as resources and shaders.

## Decision

- Establish one immutable actor render projection before allocating GPU actor resources.
- Keep signed global regions and Q9/Q16 values exact; emit no float global position.
- Let the enabled published composition select the active window and let the existing canonical
  projection select semantic region and window-relative position.
- Preserve the complete actor and published global config in the transaction evidence.
- Reject missing/stale actor, unavailable composition, and outside-window placement explicitly.
- Add no GPU resource, frame input, alternate renderer, fallback projection, camera behavior, or
  multi-actor abstraction in this stage.

## Consequences

- A later GPU experiment receives a measured exact spatial contract instead of choosing transform
  policy inside shader/resource work.
- Actors outside the current active window remain live CPU state but have no render projection.
- Actor semantic identity and visible rendering remain deliberately unresolved.

## Evidence

Experiment 0066 passed 69 focused runtime tests, affected-package compilation/clippy, strict
TypeScript checking, and a 6.61-second real-process gate over freshly cooked signed sources. The
gate proved far signed coordinates, exact center/edge Q9, canonical/origin-alias invariance,
outside/stale rollback, unchanged projection-local authorities, and identical immediate replay
hashes. The projection performed no GPU/frame/resource/synchronization work; the existing pair
publication was setup only, so the long canonical workflow was not repeated. Init passed in 0.25
seconds and the final repository guard passed in 3.72 seconds with zero Flavor denies.
