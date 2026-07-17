# ADR 0118: Retired Final Unknown-Event Witnesses

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0115 Retired Final Unknown-Event Witnesses

## Context

Three settled compatibility checks still contacted the live Workbench once per owning acceptance
run: `simulation.status`, `canonical.time.status`, and `canonical.objects.query`. All three reached
the generic unknown-event fallback and copied that fact into reports, despite existing static owner
guards already forbidding their routes and implementations.

## Decision

- Delete the three recurring retired-verb requests and their two helper functions.
- Delete `retiredStatus` and `retiredVerb` report fields without an alias, optional fallback, or
  version branch.
- Extend the existing presentation, simulation, and object guards to forbid the removed helper and
  report names beside their route/API checks.
- Keep process-level rejection tests only for malformed input to current live verbs and for current
  transaction rollback.

## Consequences

- Maintained acceptance no longer serializes or executes historical unknown-event evidence.
- Generic unknown-event behavior remains available for genuinely unknown input but is not treated
  as recurring product evidence.
- Actor, frame, and runtime report revisions advance directly to v9, v12, and v17.
- Runtime/product behavior and engine/GPU/resource ownership do not change.

## Evidence

Experiment 0115 removed three IPC requests, two helpers, and three report fields. `runseal :guard`,
`canonical-actor-v9` (75.926 seconds), `canonical-frame-v12` (43.215 seconds), and
`canonical-runtime-v17` (253.391 seconds) passed. The full workflow retained 979 Sidecar
invocations, 492 handles, 21 threads, +344,064 private bytes, two lifecycle cycles, and 24
representative artifacts totaling 25,346,327 bytes. No final report contains a removed verb,
helper/report name, or `unknown_event`.
