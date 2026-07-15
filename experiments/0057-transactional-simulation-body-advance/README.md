# Experiment 0057: Transactional Simulation-Body Advance

Status: Accepted

## Hypothesis

`Runtime` can atomically compose one caller-supplied elapsed interval with the retained terrain-body
batch by preparing both schedule and body outputs off-state and committing them only after every
check/query succeeds. This can prove exact schedule/body tick agreement and rollback without adding
a host clock, frame driver, focus/stall policy, input sampling, or presentation binding.

## Scope

Add one handle-addressed operation that accepts bounded elapsed nanoseconds and one controlled
per-tick spatial command. Copy the current schedule and retained body, advance the schedule copy,
execute exactly its emitted 0..=8 steps in local body state, then commit the body and prepared
schedule. Return both accepted advances as one transaction result.

Keep the existing explicit schedule-only and retained batch diagnostics as independent authorities.
Do not sample wall time, invoke from a frame, add pause/backlog policy, derive commands from input,
widen retained capacity, or bind actors/presentation.

## Workload

1. Submit one nanosecond from clean state. Require zero emitted/body steps, schedule remainder 60,
   zero terrain queries, and exact body identity.
2. From identical independent processes, advance one second as eight 125 ms calls and as the exact
   20x16,666,666 ns + 40x16,666,667 ns nominal partition. Require tick 60, zero remainder, and exact
   final retained motion/hash equality.
3. Cause a body query failure after at least two local steps from a clean schedule. Require exact
   schedule status and retained body rollback despite a valid prepared schedule batch.
4. Reject stale handle and elapsed above 125 ms before terrain query; cover arithmetic/contact
   failure without partial schedule/body commit.
5. Require unchanged presentation time and zero source, GPU, frame, renderer, wait, or
   synchronization work.
6. Run focused tests, a short real-process gate, `runseal :init`, and `runseal :guard`; do not run the
   long canonical workflow because no production frame/GPU/resource/lifecycle path changes.

## Controlled Variables

- Schedule constants, rational accumulator formulas, counters, and maximum batch remain unchanged.
- Retained generation, capacity, batch formulas, query ordering, and commit-on-success remain
  unchanged.
- One spatial command repeats within each elapsed-derived batch; input sampling remains deferred.
- Elapsed remains explicit caller data rather than wall-clock observation.

## Metrics

- Elapsed/start/end tick, emitted/body step count, remainder, schedule counters, generation, exact
  body input/output, and terrain-query count.
- Coarse/nominal one-second final-state and evidence hashes.
- Exact schedule/body status before and after each controlled failure.
- Non-CPU work counters, focused test count, short-gate elapsed time, and guard result.

## Acceptance Criteria

- Every success has equal emitted schedule and executed body step counts; zero-step elapsed commits
  only rational schedule state.
- Coarse and nominal one-second partitions end at tick 60/remainder 0 with byte-identical retained
  motion and generation.
- Every handle, elapsed, query, contact, or arithmetic failure commits neither schedule nor body.
- Presentation and all source/GPU/frame/renderer/synchronization evidence remain unchanged/zero.
- Focused tests, short process evidence, `runseal :init`, and `runseal :guard` pass without the long
  canonical workflow.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, deterministic signed terrain fixture,
and Windows reference workbench. Generated evidence remains ignored under `out/`.

## Evidence

Three private tests cover fractional zero-step preparation, coarse/nominal partition equality, and
schedule-copy preservation on body failure. All 62 engine-runtime tests and the workbench check pass;
Flavor reports zero deny issues.

The 32.59-second three-process gate proved that one nanosecond commits schedule remainder 60 with
zero body steps/queries. Eight 125 ms calls and the exact 20x16,666,666 ns + 40x16,666,667 ns nominal
partition both ended at tick 60/remainder 0 with byte-identical generation/motion and 60 terrain
queries; their successful schedule submission counts correctly remained 8 and 60. Evidence SHA-256
is `d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`.

A valid prepared seven-step schedule batch failed on body step 3 outside the active snapshot, and a
separate batch failed on step 1 from vertical-velocity overflow. Both left complete schedule status
and retained body byte-identical. Presentation and all source/GPU/frame/renderer/synchronization
counters remained unchanged or zero. `runseal :init` and `runseal :guard` passed.

The long canonical workflow was intentionally not run because this explicit CPU/control transaction
does not change production frame, GPU-resource, synchronization, or lifecycle behavior.
