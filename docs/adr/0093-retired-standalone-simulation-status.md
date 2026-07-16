# ADR 0093: Retired Standalone Simulation Status

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0090 Mandatory Simulation Status Cleanup

## Context

ADR 0063 retained `simulation.status` when independent simulation mutation and probe controls were
removed. It was then useful as the only direct black-box view of the runtime-owned schedule.

The current runtime has two stronger authorities: `canonical.status` publishes the same exact
schedule object as part of the sole aggregate runtime state, and every successful actor transaction
publishes its exact `SimulationAdvance`. No product consumes the standalone method or inspect verb;
only maintained acceptance support still calls it.

The actor gate also repeats eight already-retired verbs on every run even though stable removal
guards and settled experiment history own their absence. Both surfaces now preserve stage history
inside live execution.

## Decision

- Delete `Runtime::simulation_status` and the complete `simulation.status` protocol/dispatch chain.
- Read maintained black-box schedule assertions from
  `canonical.status.simulationSchedule`; continue using `SimulationAdvance` for transaction-local
  tick, step, and remainder evidence.
- Retain private schedule status encoding because canonical aggregate/frame/probe evidence and
  focused tests consume it. This decision removes only the duplicate standalone forwarding path.
- Delete the recurring eight-verb retired-control request/report list. Retain stable removal guards
  and historical records; keep one current process rejection for the newly retired status verb.
- Add no alias, redirect, new verb, response field, cache, or product telemetry.

## Consequences

- Simulation schedule state has one inspect authority instead of standalone and canonical aliases.
- Actor acceptance retains exact transaction, rollback, retained-frame, lifecycle, and restart
  evidence without a compatibility status endpoint.
- Earlier retired control names stop appearing in recurring process work and reports; their absence
  remains enforced statically.
- Product Jump/time behavior, actor/runtime transaction semantics, renderer/GPU work, sources,
  formats, assets, and Wulin behavior are unchanged.

## Evidence

Experiment 0090 removes the public Runtime method, protocol variant/parser, and workbench dispatch.
All 16 maintained reads consume the pre-existing canonical aggregate. Eight earlier retired process
requests and their report field are deleted; the newly retired verb returns generic
`unknown_event`, and a deliberate method restoration is rejected by the stable guard before build
work.

Focused checks pass 83 runtime tests and workbench/Deno checks. Expanded `canonical-actor-v7`
passes in 79.793 seconds: lifecycle/restart replay hashes match, fractional schedule is `0/60`,
coarse and nominal one-second partitions both reach `60/0` with equal actors and 60 queries,
query/arithmetic failures roll back, and pending render backpressure preserves canonical/probe
schedule `1/20` with zero commits. Actor/GPU/presentation/grounded/lifecycle evidence remains
exact. Init and guard pass with zero Flavor denies. No product, renderer/GPU/resource,
synchronization, source, format, asset, or Wulin implementation changed.
