# ADR 0128: Retired Actor Velocity Compatibility Probes

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0125 Retired Actor Velocity Compatibility Probes

## Context

Canonical actor admission still sent two malformed requests introduced when the required
transactional velocity input replaced its predecessor schema. One deleted
`initial_step_velocity_delta_q16`; the other added the never-supported
`initial_velocity_delta_q16` alias. Both copied generic invalid-payload results into every actor
report even though all maintained callers use the current field and current transaction gates
prove nonzero ordering and rollback.

## Decision

- Delete both recurring malformed actor requests, their assertions, and their report fields.
- Keep current process evidence for valid required-field commands, admitted nonzero velocity
  ordering, invalid-presentation rollback, and pending-window rollback.
- Extend the existing simulation-control removal guard as the sole static authority preventing the
  historical probes from returning and preserving the current evidence owner.
- Advance the canonical actor report directly to v10. Do not change wrappers that do not consume
  this gate.

## Consequences

- Canonical actor acceptance performs two fewer historical Workbench requests and serializes two
  fewer report fields.
- Strict payload decoding remains product behavior, but settled predecessor-shape rejection is no
  longer recurring acceptance evidence.
- No alias, fallback, compatibility decoder, replacement rejection registry, product behavior, or
  engine/GPU/resource path is added.

## Evidence

The change deletes two recurring malformed requests, two assertion blocks, and two report fields.
`runseal :guard` passes with zero Flavor denies, and `runseal :init` confirms the pinned toolchain
and hook surface.

`canonical-actor-v10` passes in 85.111 seconds. Its prepublication report keys are exactly
`before`, `invalidPresentation`, `response`, and `after`, with no retired or alias result.
The current admitted transaction retains delta 16,384, velocity `0 -> 16,384`, and center height
numerator `141,824 -> 158,208`. Pending admission retains one prepared step/query with zero
schedule/actor commits and no actor advance. Lifecycle, simulation rollback, GPU, and animation
gates pass; five generated files total 5,355,420 bytes.
