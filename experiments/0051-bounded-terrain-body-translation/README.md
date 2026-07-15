# Experiment 0051: Bounded Terrain-Body Translation

Status: Accepted

## Hypothesis

One caller-owned terrain-body motion value can translate by an exact signed planar Q9 displacement
against the committed terrain snapshot while accepting no more than a caller-supplied nonnegative
upward Q16 correction. An accepted translation preserves vertical velocity, a blocked translation
returns the input motion unchanged, and downhill separation remains unsnapped, without horizontal
velocity, collision sweeping, slope policy, actor storage, input mapping, live time, or rendering.

## Scope

This experiment composes the accepted canonical `TerrainPosition::translated_q9` operation with the
accepted exact `TerrainBodyContact` resolver. The caller supplies one copied `TerrainBodyMotion`,
signed X/Z displacement, and a nonnegative step-up limit. The runtime validates and normalizes the
candidate before reading the committed terrain height at exactly that candidate position.

If the candidate requires upward correction no greater than the limit, the output adopts the exact
resolved body and preserves the input vertical velocity. If the correction exceeds the limit, the
translation is blocked and output is exactly equal to input. Touching and separated candidates
require no correction and are accepted; separated downhill movement retains the prior vertical
coordinate rather than snapping to terrain.

This is a caller-owned transaction, not locomotion. Horizontal velocity/acceleration, continuous
collision, footprint or capsule sampling, slope/normal/material policy, sliding, gravity ordering,
input actions, simulation-schedule consumption, runtime body identity/storage, camera behavior,
object collision, and gameplay tuning remain out of scope.

## Workload

1. Add one focused pure transaction owner with a prepare phase that rejects a negative step-up
   limit and unrepresentable canonical translation before terrain lookup, followed by a finish
   phase that resolves one exact candidate contact and applies the bounded acceptance rule.
2. Expose the transaction through `Runtime` and one strict diagnostic workbench verb. Preserve the
   copied input, candidate, contact, output, blocked decision, displacement, limits, and fixed-point
   denominators in the response with zero mutation or non-CPU work.
3. Prove zero and signed displacement, positive/negative/diagonal region seams, accepted
   corrections below and equal to the limit, blocked correction above the limit, zero-limit
   behavior, downhill separation without snap, vertical-velocity preservation, and exact blocked
   output identity.
4. Prove negative limit, signed-region overflow, outside-snapshot lookup, and unrepresentable
   contact fail explicitly without partial output. Prove accepted translation partitions converge
   to the same final motion when each controlled segment remains within the same limit.
5. Run focused Rust/workbench tests, one short fresh-process gate over a current cooked snapshot,
   and `runseal :guard`. Wire the gate into the live canonical wrapper, but do not execute the long
   GPU/lifecycle workflow for this CPU-only transaction.

## Controlled Variables

- Planar coordinates use the accepted denominator 512 and unique half-open signed-region/local-Q9
  normalization. Per-call X/Z displacement remains signed `i32` Q9.
- Body heights, vertical velocity, terrain height, correction, and step-up limit use denominator
  65,536. The limit is a nonnegative caller value, not hidden engine tuning.
- Candidate body center, half-height, and vertical velocity begin exactly equal to input. Only the
  position is translated before contact resolution.
- The committed terrain snapshot is queried once at the normalized candidate position. Invalid
  limit or coordinate overflow fails before that lookup.
- Acceptance is exactly `correction_numerator <= step_up_limit_q16`. Blocked output equals the full
  input motion; accepted output uses the contact's resolved body and unchanged vertical velocity.
- No downward correction is introduced. A lower terrain sample yields a separated accepted body at
  its unchanged center height.

## Metrics

- Focused test count and exact branch coverage for accepted, equal-limit, blocked, zero-limit,
  downhill, seam, partition, and error cases.
- Fresh-process accepted/blocked correction values, serialized blocked-identity result, unchanged
  vertical velocity, deterministic result/replay SHA-256, runtime-work counters, and elapsed time.
- `runseal :guard` compilation, ownership, protocol, forbidden-surface, test, and Flavor results.

## Acceptance Criteria

- Exactly one bounded planar translation transaction composes canonical position and contact
  authorities. No alternate coordinate/contact path, fallback, compatibility verb, body store, or
  locomotion controller is introduced.
- Valid candidates query the committed normalized destination once. Negative limits and region
  overflow fail before lookup; outside-window and unrepresentable-contact errors return no output.
- Corrections below or equal to the limit are accepted exactly; corrections above it are blocked.
  Blocked output serializes exactly as input, including body and vertical velocity.
- Accepted output preserves vertical velocity. Downhill separated translation preserves center
  height and does not snap, while position displacement remains exact across signed seams.
- Controlled valid partitions converge on the same final motion and replay hash.
- Focused tests, the short process gate, and `runseal :guard` pass with zero allocation, source I/O,
  GPU copy/readback, fence wait, synchronization, schedule mutation, presentation mutation, frame,
  or renderer work. Experiment 0047 remains the current full GPU/lifecycle evidence.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain and reference Windows workbench. Its
runtime evidence reads the current committed CPU terrain snapshot; it has no renderer or GPU
dependency.

## Evidence

The implementation adds one focused terrain-query translation owner. Its single-use query closure
encodes the required order: negative limit and canonical coordinate overflow fail before lookup;
valid input constructs one normalized candidate and invokes the committed-height query exactly
once; exact contact then decides accepted versus unchanged blocked output. `Runtime` and the strict
`canonical.terrain.body.translate` diagnostic are thin consumers of that same transaction.

All 43 focused `engine-runtime` tests passed, including six new translation tests. They cover
below/equal/above correction limits, zero-limit touching and blocking, exact serialized blocked
identity, signed X/Z seams, downhill separation without snap, vertical-velocity preservation,
accepted partition convergence, validation-before-query, query failure, signed-region overflow,
and unrepresentable resolved contact. The workbench target and TypeScript support also compiled and
checked successfully.

The final fresh-process gate passed in 13,895.5 ms over the current Experiment 0047 cooked snapshot.
At signed region `(2^40, -2^40)` it discovered an adjacent Q9 displacement from local
`(-3904, -3968)` at height 130,048 to `(-3776, -3968)` at height 130,176: an exact 128-Q16 rise.
Correction 128 was accepted with limits 128 and 129, rejected with limit 127 while output remained
identical to input, and replayed with unchanged vertical velocity. Zero-limit touching, separated
downhill motion, a positive region seam, a partitioned return, malformed input, pre-publication
unavailability, negative-limit validation, outside-window lookup, and contact overflow all passed.
Simulation and presentation states were byte-identical before and after the workload; every
reported allocation, source-I/O, GPU, fence, synchronization, frame, renderer, and timeline
mutation count was zero.

The selected evidence and immediate replay both produced SHA-256
`391d3dde3b853590da02f45137cd554bd430be7b0004c1dc639dac2cd2d6d23a`.
`runseal :guard` passed in 4.3 seconds with zero Flavor denies and all repository suites green. The
live canonical wrapper now identifies Experiment 0051 and retains the translation gate. The long
canonical workflow was not executed: no frame, renderer, GPU resource, or lifecycle path changed,
so Experiment 0047 remains the current full evidence until a relevant candidate can invalidate it.
