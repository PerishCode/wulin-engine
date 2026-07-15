# Experiment 0056: Transactional Retained-Body Batch

Status: Accepted

## Hypothesis

One retained terrain body can execute an explicit 0..=8 fixed-step batch as a single transaction:
validate and copy once, advance only local motion for every requested tick, and replace the live
slot once only after complete success. This can prove body-side batch rollback and partition
equivalence before elapsed time, the simulation schedule, host clocks, input, or presentation drive
the body.

## Scope

Add one handle-addressed retained batch operation over the accepted planar-first spatial tick. The
caller supplies a bounded step count and one repeated per-tick delta, step-up limit, and acceleration.
The operation reports exact input/output, step and terrain-query counts, and unchanged generation.

Do not consume or mutate `SimulationSchedule`; sample time or input; define focus/stall/backlog
policy; add horizontal velocity/controller state; widen retained capacity; bind actors or
presentation; or add a frame-loop driver.

## Workload

1. Exercise 0, 1, and 8-step batches with exact handle validation and a maximum tied to the accepted
   simulation batch bound.
2. Compare one 8-step batch with eight one-step retained advances from identical independent process
   state. Require byte-identical final retained motion and equal summed terrain-query count.
3. Start near the committed five-by-five snapshot edge so at least two local ticks succeed and a
   later tick queries outside the snapshot. Require the whole stored body to remain exact input.
4. Reject empty/stale handles, step count 9, malformed payloads, invalid limits, query failure, and
   arithmetic failure without partial commit.
5. Require unchanged simulation/presentation status and zero source, GPU, frame, renderer, or
   synchronization work.
6. Run focused tests, a short real-process gate, `runseal :init`, and `runseal :guard`. Do not run
   the long canonical workflow because production frame/GPU/resource/lifecycle paths remain
   unchanged.

## Controlled Variables

- The retained slot remains capacity one with the accepted nonzero generation lifetime.
- Each tick reuses the accepted planar-first query order, destination reuse, contact, and fixed
  vertical integration without modifying their formulas or result types.
- One command repeats across the explicit batch; input sampling and command changes between ticks
  remain deferred.
- Maximum batch size remains exactly `SIMULATION_MAX_STEPS_PER_ADVANCE` (8).
- Zero steps performs no terrain query and preserves exact retained motion.

## Metrics

- Requested/executed step count, total terrain-query count, generation, and exact input/output.
- One-batch versus partitioned final-state and evidence hashes.
- Number of successful local steps before the controlled query failure and exact stored rollback.
- Per-operation allocations, source reads, GPU copies/readbacks, waits, synchronization, schedule,
  presentation, frame, and renderer work.
- Focused test count, short-gate elapsed time, and repository guard result.

## Acceptance Criteria

- 0..=8 steps are bounded and deterministic; 9 fails before query or retained mutation.
- One 8-step batch and eight one-step operations converge to byte-identical retained motion with the
  same generation and total query count.
- Any validation, query, contact, or arithmetic failure at any tick commits no retained output.
- Simulation schedule and presentation state remain byte-identical and all non-CPU work counters
  remain zero.
- Focused tests, short process evidence, `runseal :init`, and `runseal :guard` pass without invoking
  the long canonical workflow.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, the deterministic signed terrain
fixture, and the Windows reference workbench. Generated evidence remains ignored under `out/`.

## Evidence

Three focused private tests cover zero/maximum bounds, one 8-step batch versus eight one-step
partitions, pre-query rejection of step count 9, and a controlled third-query failure. All 59
engine-runtime tests and the workbench check pass; Flavor reports zero deny issues.

The 33.79-second real-process gate used three clean processes. Before publication, empty/stale and
malformed handles, step count 9, and a negative limit all failed without changing the stored body;
zero steps returned identical input/output with zero queries. After publication, one 8-step batch
and eight retained single-step operations produced byte-identical generation and final motion with
eight total terrain queries. Their evidence SHA-256 is
`110128827404dbe0dabc06fb31ccb9c5e66b5294d3728ff32a07fb986757b9f0`.

The controlled rollback body advanced locally across two valid snapshot regions, then failed at
step 3 of 8 with `terrain query region is outside the published active window`; readback remained
the exact batch input. Simulation and presentation status were byte-identical, and every source,
GPU, frame, renderer, wait, and synchronization counter remained zero. `runseal :init` and
`runseal :guard` passed.

The long canonical workflow was intentionally not run. Production frame, renderer, GPU-resource,
synchronization, and lifecycle paths are unchanged, and the new CPU/control boundary has focused
and short-process evidence wired into the live wrapper.
