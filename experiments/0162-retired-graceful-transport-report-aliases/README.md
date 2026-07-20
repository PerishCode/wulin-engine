# Experiment 0162: Retired Graceful Transport Report Aliases

Status: Accepted

## Hypothesis

The five constant success fields repeated by every graceful Prototype launch
and the paired `exactlyTwoValues` flag can be deleted without weakening process
or product evidence. The shared session runner cannot return until it has read
exact readiness and completion values, observed a successful process exit,
drained stdout to EOF with no non-whitespace tail, and observed empty stderr.

## Scope

- Delete raw `exitCode`, `stderr`, `exitReason`, `outputValueCount`, and
  `trailingOutput` report fields from the shared graceful launch result.
- Delete the paired invariant's static `exactlyTwoValues` flag.
- Delete the finite-boundary validator's report-level exit-code reread.
- Rename the live runner parameter to `expectedExitReason` so the retired report
  spelling has no compatibility surface.
- Make the central Prototype guard reject all retired result spellings and
  require successful exit, stderr, and trailing-output control-flow checks.
- Advance complete Prototype acceptance from v76 to v77.

Process count, native input, PID, elapsed-time, readiness, completion, product
behavior, Runtime, renderer/GPU, source, synchronization, and resource ownership
are out of scope.

## Workload

1. Inventory every producer and consumer of the six report aliases.
2. Confirm each value is constant on every successful return and has no product
   or semantic consumer.
3. Calculate the exact minimum minified report weight across 18 graceful
   raw/invariant pairs.
4. Delete the complete report/check chain without a replacement field.
5. Strengthen the current central guard around the real transport checks.
6. Advance the full workflow revision and run focused/static/full validation.
7. Inspect v77 for zero retired fields, 18 unchanged process identities and
   product payloads, zero pair copies, report reduction, and cleanup.

## Controlled Variables

- The runner still parses exactly one readiness value and one completion value.
- Nonzero exit, timeout, nonempty stderr, and nonempty trailing stdout still
  fail before any result is returned.
- Product completion remains the sole owner of `outcome` and `reason`.
- All 18 native-input/readiness/completion payloads and behavior validators
  remain unchanged.
- No compatibility alias, optional decoder, replacement boolean, fallback,
  retry, product delay, or process consolidation is admitted.

## Metrics

- Removed producer/consumer occurrences; exact minified baseline weight; source
  lines; report bytes/reduction; workflow duration; process/PID/native-input/
  readiness/completion totals; pair-copy count; behavior gates; test totals;
  Flavor findings; and process cleanup.

## Acceptance Criteria

- The graceful runner, raw result shape, invariants, and boundary validator
  contain none of the retired result fields outside the central forbidden guard.
- The shared runner structurally rejects timeout, nonzero exit, nonempty stderr,
  and nonempty trailing stdout before returning.
- v77 contains exactly 18 raw/invariant pairs with no retired report field.
- All 18 launches retain unique PIDs, native input, exact readiness/completion,
  and the expected product completion reason.
- The fixed 16-byte pairwise copy gate remains zero and every current behavior
  gate passes.
- The report shrinks by at least the exact 2,022-byte minified alias weight.
- Product, Runtime, renderer/GPU, source, synchronization, process count, and
  resource ownership diffs remain empty.

## Results

The implementation removes the five raw fields from the shared return value,
deletes `exactlyTwoValues` from the common invariant, and removes the boundary's
secondary exit-code read. The runner still reads exact readiness and completion
values, applies a 10-second exit bound, rejects unsuccessful status, drains
stdout through EOF, rejects a non-whitespace tail, and rejects stderr before it
returns. A nested static guard owns those live control-flow requirements and
forbids every retired graceful result shape; the central guard remains 499
lines.

`canonical-prototype-v77` passed in 156.947 seconds. Its 372,526-byte report is
3,109 bytes (0.83%) smaller than the 375,635-byte v76 baseline, exceeding the
exact 2,022-byte minified alias weight by 1,087 bytes.

- All 18 raw launches have unique positive PIDs and retain native input,
  readiness, and completion.
- Product completion reasons remain 17 exact `escape` values and one exact
  `window-close`.
- None of the 18 raw launches owns a retired transport field; none of the 18
  paired invariants owns `exactlyTwoValues`.
- The fixed pairwise gate remains at zero nontrivial copies.
- Activated recovery retained exact 12 Activated frames, 16 suppression frames,
  one commit, cleared target/acknowledgement, zero render blocks, and the
  accepted bounded frame observer.
- Jump focus retained exact one suspend/resume/reset boundary; finite-boundary,
  Rejected, sustained-capacity, camera, Run, diagonal movement, startup,
  restart, and Sidecar lifecycle gates all passed.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed.
Flavor retained zero denies and five existing warnings. No Prototype product,
Runtime, renderer/GPU, source, synchronization, process-count, or resource
ownership changed.

## Conclusion

Accepted. Graceful transport correctness remains enforced once by the runner's
control flow and product completion payload, while 36 repeated report owners no
longer restate constant success facts.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```
