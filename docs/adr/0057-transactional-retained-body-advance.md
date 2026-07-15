# ADR 0057: Transactional Retained-Body Advance

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0055 defines a complete copied-value planar-first terrain tick. ADR 0056 gives `Runtime` one
generation-addressed retained motion, but it deliberately exposes no mutation beyond lifecycle.
A future schedule driver needs one safe state transition before it can iterate due steps. A generic
setter would bypass terrain/contact invariants, while adding time or input now would combine
several unproven policies with the first stored update.

## Decision

- Add one handle-addressed runtime operation that validates and copies the exact live retained body
  before terrain lookup.
- Run the accepted copied-value planar-first advance unchanged against the committed CPU snapshot.
- Commit only `advance.output`, only after the entire spatial transaction succeeds, and preserve the
  live generation. Return retained input, complete copied-value evidence, and retained output.
- Keep slot replacement private and checked. Do not expose a generic setter or unchecked commit.
- Empty or stale handles fail before query. Validation, query, arithmetic, and contact failures
  return no retained advance and leave the previously readable retained value unchanged.
- Do not consume the simulation schedule, sample elapsed time or input, widen body capacity, or bind
  presentation in this decision.

## Consequences

- Runtime-owned state can now perform one exact simulation tick without transferring authority to
  the host or duplicating the accepted spatial rules.
- A later schedule driver may iterate this operation, but stall splitting, focus policy, input
  sampling, displacement derivation, and batch failure semantics still require their own evidence.
- The generation identifies lifetime, not mutation version; successful motion updates deliberately
  preserve it.
- Experiment 0055 remains the mandatory cleanup point before adding live driving.

## Evidence

Experiment 0054 added private checked replacement and the runtime read-compute-commit operation.
All 56 focused runtime tests passed, including exact replacement, unchanged generation, empty/wrong
handle rollback, and the previously accepted planar-first/query-order suites.

The final 23.36-second real-process gate ran twice across independent processes. Before terrain
publication, empty/stale handles and a negative limit rejected with exact retained rollback. After
publication, accepted uphill and downhill used one query, blocked uphill used the ordered two-query
path, and snapshot/velocity failures preserved exact state. Both runs produced SHA-256
`54dacac84b69c1ef1e98d127de23e646b0d18e6c9934e50d3e832abefa56f529` with unchanged
simulation/presentation status and zero source, GPU, frame, renderer, or synchronization work per
operation. `runseal :guard` passed. The long canonical workflow was not run because frames and GPU
resources still do not consume retained state.
