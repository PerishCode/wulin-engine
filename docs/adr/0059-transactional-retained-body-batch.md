# ADR 0059: Transactional Retained-Body Batch

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0050 establishes an explicit simulation schedule that emits at most eight fixed steps per
elapsed submission. ADR 0057 establishes one retained-body tick with commit-after-success. Calling
the two operations sequentially cannot form a live driver: whichever state commits first would
remain advanced if the other operation later failed. The body side therefore needs a bounded batch
transaction before schedule/body dual commit can be designed safely.

Adding elapsed sampling, focus/stall policy, input mapping, and schedule composition at the same time
would hide body rollback failures behind unrelated time policy. A schedule reservation/token API
would add generic surface before a concrete consumer proves its required commit shape.

## Decision

- Add one handle-addressed retained-body batch operation bounded to
  `SIMULATION_MAX_STEPS_PER_ADVANCE` (8).
- Validate and copy the live retained generation once, execute every requested tick against local
  motion, accumulate exact terrain-query count, and replace the slot once only after complete
  success. Preserve the generation.
- Accept zero steps as an exact no-query identity transaction. Reject more than eight before terrain
  lookup. Any validation, query, contact, or arithmetic failure returns no batch output and commits
  no retained motion.
- Repeat one explicit delta, step-up limit, and acceleration for the batch. This is controlled batch
  input, not an accepted host-input sampling policy.
- Report the failing one-based step and complete underlying cause without publishing partial motion.
- Do not consume or mutate the simulation schedule, sample elapsed time or input, define stall/focus
  policy, widen body capacity, or bind actor presentation.

## Consequences

- The body side can now match the schedule's maximum emitted batch without partial retained commit.
- A later experiment may compose elapsed-derived schedule preview, this body batch, and a final dual
  commit. It must still prove zero-step behavior, schedule rollback, elapsed ownership, and input
  independence rather than sequencing the existing committed operations.
- Repeating one command across a batch remains provisional controlled input; command derivation and
  changes between simulation ticks are not part of this decision.

## Evidence

Experiment 0056 added three focused private tests for zero/maximum bounds, 8-versus-8x1 partition
equality, pre-query bound rejection, and a controlled third-query failure. All 59 engine-runtime
tests and the workbench check passed; Flavor reported zero deny issues.

The 33.79-second real-process gate used three clean processes. Prepublication empty/stale/malformed,
step count 9, and invalid-limit failures preserved exact state; zero steps performed no query. With
published terrain, one 8-step batch and eight retained single-step operations ended with byte-
identical generation/motion and eight total queries, producing SHA-256
`110128827404dbe0dabc06fb31ccb9c5e66b5294d3728ff32a07fb986757b9f0`. A second body crossed two
valid snapshot regions locally, failed at step 3 of 8 outside the active window, and remained exact
input. Simulation/presentation state and all source, GPU, frame, renderer, wait, and synchronization
counters remained unchanged. `runseal :init` and `runseal :guard` passed. The long canonical workflow
was not run because no production frame, renderer/GPU resource, synchronization, or lifecycle path
changed.
