# Experiment 0150: Retired Post-Readiness Report Flags

Status: Accepted

## Hypothesis

The static `actionAfterReadiness: true` acceptance field can be deleted from every current
Prototype owner without weakening input ordering or behavior authority. The shared session runner
already reads and validates the product readiness value before it dispatches any post-readiness
action, so copying that structural fact into nested reports is historical startup-action baggage.

## Scope

- Delete all 16 `actionAfterReadiness` report producers across boundary, camera, object, focus,
  forward-release, Run-transition, opposed-locomotion, and diagonal session owners.
- Delete all 11 positive source-shape expectations from the Prototype session guard.
- Retain one central static guard over the 11 current owners to reject any restoration.
- Advance the complete Prototype workflow revision from v64 to v65.

Native input sequencing, exact-PID/window selection, action timing, process count, product output,
session schema, behavior oracles, Runtime, renderer/GPU resources, source formats, synchronization,
and workspace resource cleanup are out of scope. The next scheduled resource cleanup remains
Experiment 0160.

## Workload

1. Inventory every producer and consumer of `actionAfterReadiness`.
2. Verify that no product or semantic consumer reads the field and that shared `gracefulExit`
   ordering is the live action-after-readiness authority.
3. Delete the complete report/check chain without a replacement flag, alias, or decoder.
4. Extend the existing Prototype session guard across all 11 current report owners.
5. Run formatting, type checking, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :init`.
6. Inspect the generated report for zero retired-field occurrences and unchanged current session
   outcomes.

## Controlled Variables

- Every post-readiness action remains dispatched only after the exact child readiness value has
  been parsed.
- All native keys/messages, timing bounds, actor/camera/presentation/object/clock/frame oracles,
  and graceful completion rules remain unchanged.
- The 16 normal session processes retain exactly readiness plus completion, exit zero, empty
  stderr, and empty trailing output.
- The finite-boundary process retains its exact endpoint and standard two-value completion.
- No replacement negative fact, compatibility registry, optional-field path, retry, fallback,
  product delay, or relaxed threshold is admitted.

## Metrics

- Removed producer/check occurrences and owner coverage; baseline/final report field occurrences;
  report bytes; workflow duration; normal-session count and output/exit/stderr state;
  finite-boundary completion; test totals; Flavor findings; and process cleanup.

## Acceptance Criteria

- None of the 11 current Prototype report owners may contain `actionAfterReadiness`.
- The sole remaining source occurrence must be the central guard's forbidden token.
- No replacement report flag or compatibility branch may exist.
- The v65 report must contain zero retired-field occurrences.
- All 16 normal sessions must retain two values, exit zero, empty stderr, and empty trailing output;
  the boundary process must retain two values and exit zero.
- Every previous Prototype behavior gate, `runseal :guard`, and `runseal :init` must pass without a
  product, Runtime, renderer/GPU, source, resource, synchronization, or process-count change.

## Results

The audit found 27 live source occurrences: 16 static report producers and 11 guard expectations.
No product or semantic consumer existed. All 27 were deleted directly. The existing central
Prototype session guard now scans camera repeat/re-press, Run release/re-press, opposed locomotion,
diagonal Walk/Run, forward release, focus, finite boundary, and object gates for any return of the
retired token. The token remains only in that forbidden check.

The v64 baseline report contained 18 copied field occurrences and was 456,292 bytes.
`canonical-prototype-v65` passed on its first run in 162.812 seconds with a 455,655-byte report:
zero retired-field occurrences and a 637-byte reduction. All 16 normal sessions retained exactly
two values, exit zero, empty stderr, and empty trailing output. The finite-boundary process also
retained two values and exit zero. Readiness-only baselines remained exactly
`label/processId/elapsedMs/readiness`.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero
denies and five existing warnings. The existing v20 Runtime report already contained zero
occurrences because Runtime checkpointing consumes readiness-only Prototype captures rather than
the full session matrix, so no Runtime owner changed.

## Conclusion

Accepted. Post-readiness ordering is enforced once at the execution boundary instead of being
copied into 18 report locations. All current semantic evidence remains live, and one static guard
prevents the historical flag from returning without introducing a replacement compatibility
surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
