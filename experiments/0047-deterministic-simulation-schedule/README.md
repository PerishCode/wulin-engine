# Experiment 0047: Deterministic Simulation Schedule

Status: Accepted

## Hypothesis

The engine runtime can own one simulation schedule that converts explicit bounded elapsed
nanoseconds into exact fixed 60 Hz step batches with partition-invariant tick/remainder state,
bounded per-call work, transactional failure, and no dependence on presentation time, render-frame
success, an operating-system clock, input, body state, actors, source I/O, or GPU work.

## Scope

This experiment adds one runtime-owned rational accumulator. A caller supplies an elapsed interval
in nanoseconds; the runtime returns the start tick, zero through eight due fixed steps, end tick,
and sub-step remainder. Accepted elapsed intervals are at most 125,000,000 ns. The schedule uses
an independent 60-steps-per-second numerator over a 1,000,000,000-nanosecond denominator, so no
rounded 16,666,667 ns quantum accumulates drift.

The workbench exposes diagnostic status, advance, and isolated long-duration probe commands for
controlled evidence. The probe times a private replay copy; its clock is not an input to the live
schedule. Runtime frames, canonical publication, presentation controls, and the prototype loop do
not automatically advance simulation in this experiment. A later host experiment must choose how
to sample monotonic wall time and split or reject stalls before it can drive this schedule.

Gravity, velocity, terrain-body stepping, grounded policy, locomotion, jump, actor identity and
storage, input sampling/mapping, interpolation, render pacing, network time, rollback, persistence,
and gameplay are out of scope. Presentation time remains its existing frame-transaction authority
and is not reused as simulation time.

## Workload

1. Define the fixed schedule constants, exact accumulator state, typed advance result, checked
   arithmetic, and runtime ownership. Cover zero/sub-step intervals, exact boundaries, seven/eight
   step batches, invalid elapsed bounds, tick/counter overflow, and failure rollback.
2. Compare multiple partitions and orders totaling the same elapsed duration, including 60
   nominal-frame intervals and eight 125 ms intervals for one exact second. Require the same final
   60 ticks and zero remainder; exact replay of one sequence must reproduce every batch and hash.
3. Advance a long controlled duration through bounded calls, recording step total, tick/remainder,
   batch histogram, result SHA-256, elapsed CPU time, and zero allocation/I/O/GPU/synchronization
   counters.
4. Expose `simulation.status` and `simulation.advance` diagnostics. Reject malformed, oversized,
   and overflowing requests without state mutation. Process restart must reset the schedule.
5. Render idle and canonical frames, mutate/pause/step presentation time, reorder sources, hold all
   four I/O/copy gates, fail both pair halves, traverse 32 reactive plus 32 prepared crossings, and
   roll over without an explicit simulation advance. Simulation status must remain byte-exact.
6. Use focused tests and a short runtime timing gate during implementation, run `runseal :guard` at
   the stable checkpoint, then run the complete canonical workflow once for the merge candidate.

## Controlled Variables

- `SIMULATION_STEPS_PER_SECOND` is 60 and `SIMULATION_TIME_DENOMINATOR` is 1,000,000,000 ns.
  Successful advance computes `total = remainder + elapsed_ns * 60`, emits
  `floor(total / 1,000,000,000)` steps, and retains the exact modulus.
- Accepted elapsed input is `0..=125,000,000` ns. With a prior remainder below the denominator,
  one call emits at most eight steps. Larger intervals fail; this experiment does not silently
  clamp, drop, or retain a catch-up backlog.
- Tick, successful-advance count, emitted-step count, and remainder use checked integer mutation.
  The state commits atomically only after every next value is representable.
- The live schedule is process-local runtime state. It has no wall-clock source, thread, sleep,
  lock, worker, callback, per-step closure, body/actor collection, or renderer execution
  dependency. Probe-only status snapshots may accompany diagnostic frame evidence.
- A successful or failed `Runtime::frame` does not advance simulation. Presentation pause/set/step
  does not mutate it, and simulation advance does not mutate presentation time.
- Existing sources, identity, residency, composition, traversal, rendering, captures, hashes,
  resources, and lifecycle controls remain unchanged.

## Metrics

- Focused test count; exact step/remainder sequences; rollback results for invalid and overflow
  cases; partition/replay hashes.
- Controlled call count, elapsed input sum, emitted step count, zero-through-eight batch histogram,
  final tick/remainder, result SHA-256, and elapsed CPU time.
- Per-advance allocation bytes, source reads, GPU copies/readbacks, fence waits, synchronization,
  and frame/presentation mutation counts.
- Simulation status across idle/canonical frames, presentation mutation, source/failure/hold,
  traversal, rollover, restart, resource plateau, and lifecycle operations.
- Existing controlled GPU/query/contact hashes and resource/lifecycle evidence.

## Acceptance Criteria

- One runtime-owned schedule implements the exact rational formula with a remainder strictly below
  1,000,000,000. Every accepted call emits zero through eight steps and returns matching typed
  start/end evidence without float conversion, rounded quantum, clamp, drop, or backlog.
- Equal elapsed totals produce equal final tick/remainder regardless of valid partition/order.
  Exact replay reproduces every batch and hash. One exact second produces 60 steps and remainder
  zero; the long-duration workload has no drift or overflow.
- Oversized elapsed input and any unrepresentable next counter/tick fail without partial mutation.
  Restart begins at tick/remainder zero. No hidden wall clock or render frame can mutate the state.
- Each successful advance reports zero allocation, source read, GPU copy/readback, fence wait, and
  synchronization work. Work is bounded by one fixed arithmetic transaction rather than due-step
  iteration.
- Presentation and simulation states remain independent through controls, failures, holds,
  traversal, rollover, resource plateau, and 16 lifecycle cycles. Existing controlled hashes,
  submissions, and resources remain exact.
- Focused tests, the short runtime gate, `runseal :guard`, and the one final canonical merge
  workflow pass without validation error or device removal.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows host. The complete
merge-candidate workflow retains the accepted D3D12 Agility SDK, DXC, and reference adapter selected
by `runseal :init`; simulation scheduling itself performs no GPU work.

## Evidence

The direct merge-candidate workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0047-deterministic-simulation-schedule/`.

## Results

- Four schedule-focused tests within the 29-test `engine-runtime` suite proved exact one-second
  partition/reorder/replay behavior, zero/sub-step/eight-step bounds, transactional invalid and
  overflow failure, and 216,000 drift-free ticks across one hour. Focused crate checks and
  `runseal :guard` passed.
- The short process gate completed in 25.1 seconds. Eight 125 ms calls emitted
  `[7,8,7,8,7,8,7,8]`; the 60-part nominal sequence also ended at tick 60 with zero remainder.
  Their batch hashes were respectively
  `a59026aed1a7b8c6a23608bb39e41bec41ec02449352fe20db44a25b4c5260db` and
  `1195443996da299f24598021fa91294fe2e6d11fdb7343bf64e005b54a77a33b`.
- The merge-candidate one-hour probe made 28,800 bounded advances in 33,069,100 ns, emitted
  216,000 steps, retained zero remainder, and recorded 14,400 seven-step plus 14,400 eight-step
  batches. Exact replay reproduced
  `1ee26e9eba0160996a3cb554c17f7641ce766728f57f97bc2a1167350ca2a374` with zero reported
  allocation, source read, GPU copy/readback, fence wait, or synchronization work.
- Oversized and malformed inputs were rejected without mutation. Two process replacements reset
  all schedule state, simulation advance left presentation status exact, idle frames did not
  advance simulation, and the probe did not mutate the live schedule.
- The one final `runseal :canonical-runtime` workflow passed in 692.8 seconds. Every canonical,
  failure, hold, presentation, traversal, rollover, resource, and lifecycle probe retained the
  zero schedule invariant; reactive/prepared traversal each completed 32 samples, the 64-publication
  plateau held 531 handles with zero transient growth, and all 16 lifecycle cycles stopped cleanly.
  Accepted color, PNG, object-ID, diagnostic, terrain-query, and terrain-contact hashes were
  unchanged.

## Conclusion

The hypothesis is accepted. `Runtime` now owns one exact, bounded, explicit 60 Hz simulation
schedule whose state is independent from presentation and rendering. It establishes the quantum
needed by later spatial simulation without prematurely choosing a live host clock, stall policy,
body store, gravity, locomotion, actor update, or interpolation policy.
