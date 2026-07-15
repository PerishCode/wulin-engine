# ADR 0063: Retired Independent Simulation Controls

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

Runtime's accepted schedule/body transaction prepares both states and commits them together. Older
diagnostics still mutate the schedule alone or the retained body alone, allowing live states that
the dual authority is designed to prevent. Their recurring support also repeats the same pure
partition, single-tick, and batch proofs through eight process-start sites on every canonical run.

The focused schedule and spatial tests remain the correct authorities for their pure contracts.
The process-level schedule probe duplicates the accepted one-hour private test.

## Decision

- Delete `simulation.advance`, `simulation.probe`,
  `canonical.terrain.body.retained.advance`, and
  `canonical.terrain.body.retained.batch` without aliases or fallback routes.
- Delete their protocol payloads/variants, workbench dispatch, four public `Runtime` forwarders,
  `RetainedTerrainBodyAdvance`, and the live `simulation_probe` implementation.
- Delete `simulation-schedule.ts`, `retained-advance.ts`, and `retained-batch.ts` plus their
  canonical-wrapper imports, calls, and report fields.
- Retain read-only `simulation.status`, retained spawn/read/despawn, the sole
  `simulation.terrain.body.advance` mutation route, and every focused schedule/single/batch/dual
  test and private implementation.
- Check all retired verbs inside the existing dual prepublication process and add a stable
  forbidden-file/symbol guard.

## Consequences

- Live callers cannot advance simulation time without the retained body or mutate the body without
  its schedule.
- Historical experiments retain their evidence, but their process controls are not compatibility
  surfaces.
- The direct canonical wrapper loses 848 physical lines of recurring support and eight process-start
  sites while preserving the one combined gate that covers successful and failed dual commit.
- Future clock/application composition has one explicit mutation authority and no old bypass.

## Evidence

Experiment 0060 removed four verbs, four protocol/dispatch chains, four public `Runtime` forwarders,
one obsolete result type, the live one-hour probe, three support files, and the corresponding
wrapper report fields. The complete working change is a net deletion, while all 62 engine-runtime
tests—including the private one-hour schedule proof and retained single/batch rollback tests—pass.

The 53.3-second fresh setup plus focused dual gate used the same three measured scenario processes;
its existing prepublication process rejected all four retired verbs as `unknown_event` without an
additional start. Fractional, coarse/nominal, query-failure, and arithmetic-failure evidence
remained exact, with unchanged SHA-256
`d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`.
`runseal :init` and `runseal :guard` passed with zero Flavor denies. The long canonical workflow was
not run because frame, renderer, GPU resources, synchronization, and lifecycle owners are unchanged.
