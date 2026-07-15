# Experiment 0048: Fixed Terrain Body Motion

Status: Accepted

## Hypothesis

One caller-owned vertical terrain-body state can consume the accepted 60 Hz simulation quantum
through an exact single-step transaction, producing deterministic falling, landing, grounded hold,
and upward departure without runtime body storage, live wall time, render-frame dependence, float
integration, downward snap, or hidden movement policy.

## Scope

This experiment adds a caller-owned motion value containing the accepted `TerrainBody` plus signed
vertical velocity in Q16 height numerators per simulation step. One step uses semi-implicit Euler:
checked signed acceleration first updates velocity, then the new velocity updates center height.
The runtime queries the committed terrain snapshot at the unchanged horizontal position exactly
once and applies the accepted minimum-upward contact resolver.

A non-separated predicted contact with non-positive velocity is grounded and returns zero vertical
velocity. Positive velocity is never cancelled merely because the predicted foot is touching or
required correction. A separated body is never snapped downward. The returned motion remains
caller-owned and can be supplied to the next explicit step.

Horizontal velocity, slope/normal/material response, footprint, step height, skin width, tolerance,
jump impulse policy, gravity constant, input mapping, live clock sampling, stall splitting,
interpolation, actor identity/storage, object collision, networking, and rendering are out of
scope. Acceleration is caller supplied; this experiment does not encode Wulin tuning in the engine.

## Workload

1. Define typed motion and step-result values plus one pure checked step over an exact terrain
   height. Cover separated fall, exact landing, penetration correction, grounded hold, positive
   departure, velocity/position overflow, and unrepresentable resolution.
2. Consume the step counts returned by eight 125 ms schedule advances and by the accepted 60-part
   nominal partition. Apply identical acceleration for every due step and require byte-exact final
   motion/contact hashes after one second regardless of batch partition.
3. Run a controlled upward departure followed by apex, descent, landing, and 32-step grounded hold.
   Record takeoff/apex/landing ticks, correction count, state SHA-256, and exact final body/velocity.
4. Exercise the real workbench/runtime boundary against a committed terrain snapshot. Reject
   unavailable terrain, invalid body shape, malformed payload, velocity overflow, position
   overflow, and unrepresentable contact without retaining partial state.
5. Prove every step reports zero allocation, source read, GPU copy/readback, fence wait, and
   synchronization work. The runtime must not mutate its schedule or presentation state.
6. Use focused Rust tests, one short process gate, and `runseal :guard`. Do not repeat the long GPU
   workflow: this change adds a caller-invoked CPU transaction and diagnostic route only, without
   modifying frame, renderer, GPU resource, or lifecycle execution. Add the gate to the live
   canonical wrapper so the next legitimately required full run includes it.

## Controlled Variables

- One call advances exactly one `SIMULATION_STEPS_PER_SECOND = 60` tick. It accepts no elapsed time
  and cannot iterate a `SimulationAdvance` batch internally.
- Body center, half-height, velocity, and acceleration use signed Q16 height numerators. Velocity is
  measured per fixed step and acceleration per fixed step squared; the common height denominator
  remains 65,536.
- The integrator computes `next_velocity = velocity + acceleration` and
  `predicted_center = center + next_velocity` with checked `i64` intermediates and representable
  signed `i32` outputs before contact resolution.
- Grounded is exact: predicted touching or penetrating plus non-positive next velocity. A grounded
  output has zero vertical velocity and its resolved foot equals terrain exactly.
- The committed terrain query and minimum-upward correction remain the sole contact authority.
  No render LOD, camera state, source fallback, tolerance, or previous terrain value participates.
- State is passed and returned by value. Failure cannot mutate the caller, runtime schedule,
  presentation timeline, terrain snapshot, or renderer.

## Metrics

- Focused test count, exact per-step state sequences, partition hashes, rollback cases, and failure
  messages.
- Due-step count, takeoff/apex/landing ticks, grounded-hold count, correction count, final body and
  velocity, result/replay SHA-256, and elapsed CPU time.
- Allocation, source-read, GPU-copy/readback, fence-wait, synchronization, schedule-mutation, and
  presentation-mutation counts.
- Existing `runseal :guard` ownership, dependency, compilation, test, protocol, and Flavor gates.

## Acceptance Criteria

- The typed transaction implements the stated semi-implicit formula for exactly one accepted tick,
  with checked arithmetic and no float, rounded duration, hidden iteration, or partial mutation.
- Equal total due-step counts produce identical motion/contact results independent of valid schedule
  batching. Exact replay reproduces all steps and hashes.
- A separated falling body remains unsnapped until it reaches/crosses terrain; exact or penetrating
  landing resolves to the minimum valid center, zero velocity, and grounded. Grounded hold remains
  exact; positive departure is not cancelled.
- Invalid or unrepresentable input fails explicitly. The committed terrain snapshot is sampled once
  per successful step, and unavailable terrain has no fallback.
- Each step reports zero allocation, I/O, GPU, fence, and synchronization work and leaves schedule
  and presentation states byte-exact.
- Focused tests, the short process gate, and `runseal :guard` pass. The unchanged frame/GPU/lifecycle
  path retains the accepted Experiment 0047 full-workflow evidence rather than paying another
  approximately eleven-minute run with no new coverage.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows workbench. Motion
stepping itself is CPU-only and performs no operating-system clock or GPU call.

## Evidence

The focused `engine-runtime`/`workbench` suite passed all 33 tests. Pure tests cover exact falling,
touching landing, penetration correction, grounded hold, positive departure, two valid schedule
partitions, deterministic replay, velocity overflow, position overflow, and unrepresentable
contact without partial caller state.

The short real-process gate passed in 31 seconds. Eight 125 ms advances and the accepted 60-part
nominal partition each produced the same 60-step sequence across a process restart:

- step SHA-256: `ac02bf0bc3ac0229d5663b566d7d4b0524640318cbd7158b8294614aa8dc6856`;
- final-state SHA-256: `fd2d79e9ce175e8702a0a91079a4213cbb009edcd8d714815687b9f6dd371fb5`;
- first grounded tick 19, grounded count 42, correction count 41, maximum center numerator
  203,684, and apex ticks 9 and 10;
- coarse stepping 2,503.5 ms and nominal stepping 3,815.3 ms in the reference process gate;
- zero per-step allocation, source read, GPU copy/readback, fence wait, synchronization, schedule
  mutation, and presentation mutation counts.

Unavailable terrain, malformed payload, invalid body shape, velocity overflow, position overflow,
and unrepresentable contact all rejected explicitly. A clean third process retained neither motion
nor schedule state. `runseal :guard` passed with zero Flavor denies and all repository suites green.

No full canonical workflow was repeated. The implementation changes only a caller-invoked CPU
transaction and diagnostic route, so Experiment 0047 remains the current frame/GPU/lifecycle
evidence. The live `runseal :canonical-runtime` wrapper now includes this gate and will exercise it
during the next merge candidate that legitimately requires the full workflow.
