# ADR 0112: Frame-Bound Capacity-Exhaustion Feedback

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0109 Capacity-Exhausted Object-Action Feedback

## Context

The Prototype retains exactly one process-scoped consumed identity. Exact nearest exclusion lets a
later observation select a different object, but Enter currently reports capacity exhaustion only
as an invisible ineligible counter. `OutsideFacing` already proved one red immutable frame
rejection, and the bounded successful-session contract can now accept a post-readiness observation
and action without recurring telemetry.

A capacity rejection differs from facing rejection: capacity is known before canonical object
resolution, so adding proximity/facing work would be false authority. It also occurs while the
first identity has already become immutable frame suppression; acknowledging the second identity
must not make the first one reappear.

## Decision

- When capacity is exhausted and a different currently resolved target exists, the Prototype
  derives one red `Rejected` feedback candidate from that target's qualified identity only.
- Capacity rejection performs no canonical object resolution, proximity calculation, or facing
  calculation. Its report contains no such evidence.
- Missing or unavailable targets keep feedback-free `CapacityExhausted`.
- A projected capacity rejection returns `applied=false` and uses the existing 12-frame
  acknowledgement owner.
- The sole consumed identity, committed count, nearest exclusion, and source/session lifetime do
  not change.
- Suppression of the consumed identity remains active when the acknowledgement belongs to a
  different rejected target. Only the original Activated acknowledgement may defer suppression of
  that same consumed identity.

## Consequences

- Capacity exhaustion becomes visible without inventing a second product effect or canonical
  object authority.
- One frame may carry red target feedback for the second identity and suppression for the first;
  both already use the sole frame transaction.
- This decision does not authorize repeated consumption, another timer, an action-result queue,
  canonical mutation, registry, inventory, reward, dispatch, respawn, persistence, networking, or
  Wulin semantics.

## Evidence

Eleven exact interaction-policy tests prove typed projected/unprojected capacity rejection,
distinct/resolved-target validation, no proximity/facing evidence, immutable count/identity state,
and continuous suppression while the second identity owns the existing acknowledgement.

`canonical-prototype-v26` passes in 80.596 seconds. The sustained native process moves from
readiness live frame 5 to completion live frame 792, retains consumed/excluded qualified ID 496,
selects independently verified qualified ID 501 after exact `D up`, `F up/down`, and `Enter
up/down`, projects exactly 12 red rejection frames while retaining 776 suppression frames, and
finishes with committed/ineligible counts 1/1, no acknowledgement, no copied object state, and no
event history. `runseal :guard` passes with zero Flavor denies. No engine, renderer, shader, ABI,
resource, descriptor, copy, readback, or synchronization structure changed.
