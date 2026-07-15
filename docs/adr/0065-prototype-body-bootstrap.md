# ADR 0065: Prototype Body Bootstrap

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The prototype has strict canonical content readiness but no simulation entity. Host time cannot
become a useful live dependency until one composition root owns a retained body handle. Extending
the shared bootstrap schema or using workbench inspect controls would either add configuration
without demonstrated need or make diagnostics authoritative for prototype behavior.

## Decision

- Let `apps/prototype` create exactly one retained terrain body after canonical publication and
  before readiness output.
- Derive its horizontal position from the existing configured `globalCenter` with exact local Q9
  zero; do not add a bootstrap schema field.
- Query the committed terrain once, use a prototype-owned fixed 65,536-Q16 half-height, place the
  foot exactly touching, and set step velocity to zero.
- Treat query, arithmetic, or spawn failure as terminal before readiness.
- Publish the exact terrain sample and retained body in the one prototype readiness line.
- Keep workbench behavior, HostClock, schedule advance, input/locomotion, actors, frames, rendering,
  and presentation unchanged.

## Consequences

- Prototype owns a real process-local simulation handle without yet defining movement behavior.
- Identical content/config restarts can prove exact initial state and generation reset.
- Live clock driving and movement policy remain separate experiments over a concrete body consumer.

## Evidence

Experiment 0062 added two focused prototype derivation/failure tests and exact retained-body
assertions to the existing prototype lifecycle gate. A 50.4-second fresh-cook targeted run proved
byte-identical generation-one body evidence across two distinct process IDs at a signed far center,
terminal no-readiness behavior for invalid/missing/corrupt inputs, and empty final Sidecar PID state.

The measured terrain/center/half-height numerators were `76288`, `141824`, and `65536`, with local Q9
zero and zero step velocity. `runseal :init` and `runseal :guard` passed with zero Flavor denies. The
full canonical workflow was not run because the targeted gate covers the changed prototype startup
while workbench frame/GPU/resource/synchronization behavior remains unchanged.
