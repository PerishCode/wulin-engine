# Experiment 0052: Planar-First Terrain-Body Advance

Status: Accepted

## Hypothesis

One caller-owned terrain-body motion can execute exactly one deterministic spatial tick by applying
the accepted bounded planar translation before the accepted fixed vertical step. Accepted movement
can reuse its destination terrain sample, blocked movement can still apply vertical motion at the
unchanged origin, and downhill separation can begin falling in the same tick, without horizontal
velocity, input mapping, live schedule driving, body storage, swept collision, or rendering.

## Scope

This experiment composes the two accepted caller-owned transactions without replacing either
authority. Inputs are copied motion, signed planar Q9 displacement, a nonnegative Q16 step-up limit,
and signed Q16 vertical acceleration for one fixed step.

The planar transaction runs first against start-of-tick motion. Its accepted or exact unchanged
blocked output becomes the input to one vertical step. If planar output occupies the already sampled
candidate position, vertical integration reuses that terrain height; a blocked nonzero displacement
queries the retained origin once before vertical contact. The combined result exposes both complete
subtransactions, final motion, grounded state, query count, fixed rate, and denominators.

The order deliberately makes step-up acceptance independent of same-tick vertical velocity and
makes newly separated downhill motion fall immediately. Jump clearance, swept terrain collision,
footprints, slopes/materials, sliding, horizontal velocity/acceleration, speed, input actions,
simulation batch iteration, live time, runtime body identity/storage, object collision, actors,
camera behavior, and gameplay tuning remain out of scope.

## Workload

1. Add one pure planar-first composition owner over the accepted translation and vertical
   integrator. Preserve complete intermediate evidence and return no partial advance on failure.
2. Prove accepted flat/uphill grounding, same-tick downhill falling, blocked-horizontal plus
   vertical progress, positive departure, zero displacement, signed seams, and exact final output.
3. Count query calls and positions. Require one destination query for accepted/zero moves and two
   ordered queries—blocked destination then retained origin—only when positions differ.
4. Prove invalid limit and coordinate overflow stop before vertical work; prove destination/origin
   unavailability, velocity/center overflow, and unrepresentable contact fail transactionally.
5. Exercise one strict workbench diagnostic over a current committed snapshot. Reproduce controlled
   accepted, blocked, downhill, seam, and equal-step-partition sequences and require exact hashes,
   schedule/presentation independence, and zero non-CPU work.
6. Run focused Rust/workbench tests and `runseal :guard`. Retain the new gate in the live canonical
   wrapper without executing the long GPU/lifecycle workflow for a CPU-only caller transaction.

## Controlled Variables

- Planar translation inputs and semantics remain exactly Experiment 0051: signed Q9 displacement,
  explicit nonnegative Q16 step-up limit, atomic blocked output, and no downward snap.
- Vertical integration remains exactly Experiment 0048: one 60 Hz step, checked semi-implicit
  velocity/center update, exact contact, and nonpositive grounded velocity reset.
- Planar translation always completes before vertical integration begins. A blocked result is a
  successful planar outcome, not a failed combined tick.
- Terrain samples are immutable committed-snapshot values for the duration of the call. A sample is
  reused only when the vertical input position equals the planar candidate position.
- The combined call does not advance or inspect the simulation schedule. Callers invoke it exactly
  once for each due step and supply displacement, limit, and acceleration explicitly.
- Failure returns no combined value and mutates no caller/runtime state.

## Metrics

- Focused test count and exact accepted/blocked/downhill/departure/seam/error branch assertions.
- Terrain query count/order/position for one-query reuse and two-query blocked paths.
- Controlled trajectory ticks, grounded/blocked counts, final state, result/replay SHA-256, and
  short process-gate elapsed time.
- Allocation, source-I/O, GPU, fence, synchronization, frame, renderer, schedule, and presentation
  mutation counters plus `runseal :guard` results.

## Acceptance Criteria

- Exactly one combined transaction orders bounded planar translation before one vertical step and
  returns the vertical output as final motion. No parallel coordinate/contact implementation,
  fallback order, hidden mode, or retained body state exists.
- Same-tick vertical velocity cannot alter the planar step-up decision. A blocked nonzero move keeps
  the origin but still executes vertical motion; an accepted downhill move begins falling in that
  tick rather than remaining suspended with zero velocity.
- Accepted/zero translation uses exactly one terrain query through destination reuse. A blocked
  move with a distinct candidate uses exactly two ordered queries and applies vertical contact only
  to the retained origin.
- Invalid and unrepresentable operations fail explicitly without partial output. Signed seams and
  fixed-point denominators remain exact.
- Equal due-step partitions and immediate replay produce identical intermediate/final evidence and
  hashes while schedule and presentation states remain unchanged.
- Focused tests, the short process gate, and `runseal :guard` pass with zero non-CPU work. Experiment
  0047 remains the current full frame/GPU/lifecycle evidence.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows workbench. It reads
only immutable committed CPU terrain data and has no renderer or GPU dependency.

## Evidence

The implementation adds one focused `TerrainBodyAdvance` owner that calls the accepted translation
before the accepted vertical integrator. It does not duplicate normalization, contact, correction,
or grounded logic. Accepted/zero moves reuse `translation.contact.terrain`; a blocked distinct
candidate performs one additional query at the retained output position. The runtime and strict
`canonical.terrain.body.advance` diagnostic consume that same copied-value transaction.

All 49 focused `engine-runtime` tests passed, including six new advance tests. They prove accepted
uphill height reuse and grounding, immediate downhill falling, blocked planar identity plus vertical
progress at origin, zero movement, signed seams, exact final output, one/two-query order, validation
before query, destination and retained-origin failure, signed-region overflow, velocity overflow,
and contact overflow without partial results. The extracted strict terrain protocol submodule and
workbench target compile cleanly.

The final fresh-process gate passed in 20,516.5 ms over the current cooked snapshot. At signed
region `(2^40, -2^40)`, local `(-3904, -3968)` / `(-3776, -3968)` again exposed exact heights
130,048 / 130,176 and a 128-Q16 rise. The accepted uphill used one query and grounded at the upper
sample. The below-limit candidate was blocked, retained exact planar input, used the ordered
destination/origin pair of queries, and still ran vertical grounding at the origin. The downhill
case used one query and ended the same tick separated with velocity -10 rather than suspended at
zero. Zero displacement and a positive signed-region seam also used one query.

Two 60-advance sequences grouped as `8/8/8/8/7/7/7/7` and sixty individual calls produced
byte-identical step evidence and final motion. The direct cases plus sequence produced result and
replay SHA-256
`7463970a8748a5aa02567c2ea94b64d2b8e527968360d30b34cef2568db02142`.
Negative limit, malformed input, pre-publication unavailability, outside destination, blocked
outside retained origin, velocity overflow, and unrepresentable contact all failed explicitly.
Simulation and presentation states were unchanged; every allocation, source-I/O, GPU, fence,
synchronization, frame, renderer, and timeline mutation count was zero.

`runseal :guard` passed in 5.3 seconds with zero Flavor denies and all repository suites green. The
live canonical wrapper now identifies Experiment 0052 and retains the advance gate. The long
canonical workflow was not executed because no frame, renderer, GPU resource, synchronization, or
lifecycle path changed; Experiment 0047 remains the current full evidence.
