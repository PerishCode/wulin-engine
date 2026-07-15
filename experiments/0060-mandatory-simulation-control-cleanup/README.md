# Experiment 0060: Mandatory Simulation-Control Cleanup

Status: Accepted

## Hypothesis

After transactional schedule/body dual commit is accepted, independent schedule mutation,
retained single/batch mutation, and the process-level schedule probe are history surfaces. Their
complete live chains and recurring process gates can be deleted while private schedule/spatial
contracts remain the sole implementation authorities behind the dual transaction.

## Scope

Delete `simulation.advance`, `simulation.probe`,
`canonical.terrain.body.retained.advance`, and
`canonical.terrain.body.retained.batch`; their protocol variants/payloads, workbench dispatch,
public `Runtime` forwarders, obsolete retained result type, live schedule-probe implementation,
three Runseal support files, and canonical-wrapper calls/evidence.

Retain `simulation.status`, retained spawn/read/despawn, `simulation.terrain.body.advance`,
`SimulationSchedule::advance`, private terrain single/batch composition, and all focused tests.
Add a stable removal guard and check the four retired verbs inside the existing dual prepublication
process. Add no alias, fallback, deprecated route, or new wrapper mode.

## Workload

1. Delete all four inspect/parser/dispatch/Runtime chains and the three support files.
2. Delete the live one-hour schedule probe while preserving the focused one-hour schedule test.
3. Remove superseded gate imports/calls/report fields from the direct canonical wrapper.
4. Require all four retired verbs to return `unknown_event` inside the current dual gate, then prove
   fractional, coarse/nominal, query-failure, and arithmetic-failure dual behavior unchanged.
5. Add a forbidden-file/symbol guard that excludes historical experiment/ADR text.
6. Measure deleted support lines and canonical-wrapper gate/startup removal.
7. Run focused runtime/workbench checks, the dual process gate, `runseal :init`, and
   `runseal :guard`. Do not run the long canonical workflow because frame/GPU/resource/lifecycle
   code is unchanged and the modified control/support boundary has a direct gate.

## Controlled Variables

- Schedule formulas, bounds, status, and all private tests remain unchanged.
- Retained slot lifecycle, private batch execution, dual prepare/commit, and failure rollback remain
  unchanged.
- Presentation, input, host clock/activation, frames, renderer, streaming, and GPU work remain
  unchanged.
- The canonical wrapper stays direct and non-recursive; later legitimate full runs consume the
  reduced gate set.

## Metrics

- Deleted files, physical lines, verbs, variants, payloads, dispatch branches, result type, Runtime
  forwarders, and wrapper evidence fields.
- Retired verb `unknown_event` results and unchanged dual evidence SHA-256.
- Focused test counts, dual-gate elapsed time/process count, Flavor denies, and guard result.

## Acceptance Criteria

- No retired file, live verb, variant, forwarding method, result type, probe function, error prefix,
  or gate name remains outside historical docs; a stable guard prevents restoration.
- Private schedule/single/batch authorities and all focused tests pass.
- The existing dual gate rejects all retired verbs and preserves its accepted result hash and exact
  rollback behavior without an extra process.
- The direct wrapper no longer starts or reports the three superseded gates.
- Focused checks, dual process evidence, `runseal :init`, and `runseal :guard` pass without the long
  canonical workflow.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains and Windows reference workbench. Generated
process evidence remains ignored under `out/`.

## Evidence

The complete independent control chain was removed without aliases: four verbs, their protocol
variants/payloads, four workbench branches, four public `Runtime` forwarders, one obsolete retained
result type, and the live process-level schedule probe. Three recurring support files totaling 848
physical lines were deleted; the direct canonical wrapper fell from 411 to 402 physical lines and
lost eight process-start sites plus their report fields.

The private schedule, retained single-tick/batch, and dual-composition implementations/tests remain.
All 62 engine-runtime tests pass, including exact one-second partitioning, one-hour no-drift,
single/batch rollback, and schedule/body dual rollback.

A 53.3-second fresh setup plus focused dual gate rejected `simulation.advance`,
`simulation.probe`, `canonical.terrain.body.retained.advance`, and
`canonical.terrain.body.retained.batch` as `unknown_event` inside its existing prepublication
process. No extra process was added. Fractional zero-step, coarse/nominal one-second convergence,
step-3 query failure, and step-1 arithmetic failure remained exact; result SHA-256 is unchanged at
`d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`.

The stable guard rejects removed files, live symbols, verbs, forwarders, result/probe names, error
prefixes, and gate functions. `runseal :init` and `runseal :guard` passed with zero Flavor denies.
The long canonical workflow was not run because production frame, renderer, GPU resource,
synchronization, and lifecycle ownership is unchanged; the modified control/support boundary has
the direct focused gate above.
