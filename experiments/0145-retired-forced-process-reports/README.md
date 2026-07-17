# Experiment 0145: Retired Forced Process Reports

Status: Accepted

## Hypothesis

Nine forced/readiness-only report and check occurrences can be deleted without weakening current
startup-failure or process-lifetime authority. The two startup-failure paths already reject
successful output directly, while the readiness-only capture can reject stderr and every byte after
the first readiness value instead of copying termination metadata and static absence facts into
acceptance reports.

## Scope

- Delete `readinessEmitted` from both Prototype and configured Runtime bootstrap failure reports.
- Delete `forcedEvidenceExitCode`, copied `stderr`, `trailingOutput`, and `completionEmitted` from
  the shared readiness-only capture result.
- Delete the downstream `completionEmitted`/`trailingOutput` check and
  `forcedReadinessCompletionEmitted` summary copy.
- Make readiness-only capture fail directly on nonempty stderr or any output after readiness.
- Extend the existing Prototype session guard across both failure owners, the capture result, and
  the downstream session gate.
- Advance the complete Prototype and Runtime workflow revisions from v59/v19 to v60/v20.

Product session schema and output, normal graceful-session evidence, startup failure diagnostics,
process count, Runtime, renderer/GPU resources, source formats, synchronization, and workspace
resource cleanup are out of scope. The next scheduled resource cleanup remains Experiment 0160.

## Workload

1. Audit both failure reports, the readiness-only process capture, its downstream check and summary,
   and the current removal guard.
2. Delete the complete nine-occurrence report/check chain without an alias or replacement flag.
3. Retain direct startup stdout-role rejection; after the readiness-only process is terminated,
   drain stdout and reject every extra byte plus any stderr.
4. Run `runseal :guard`, `runseal :canonical-prototype`, and `runseal :canonical-runtime`.
5. Inspect both JSON reports for exact current field shapes and zero retired field names.

## Controlled Variables

- Missing/corrupt startup remains terminal and retains exact code/stdout/stderr diagnostics.
- Readiness-only capture still publishes exactly the first valid Prototype readiness value and then
  terminates that evidence process.
- Normal graceful sessions retain readiness plus completion, exit status, stderr, output count,
  and dynamic trailing-output enforcement.
- Every native input, object, finite-boundary, resource, lifecycle, and process-cleanup gate remains
  live.
- No decoder, optional field, compatibility alias, replacement negative flag, rejection registry,
  telemetry, retry, resource cleanup, or product behavior is added.

## Metrics

- Removed report/check occurrences; exact current result field sets; retired-name report counts;
  Prototype/Runtime workflow duration and report bytes; test counts; Flavor findings; Sidecar
  invocations; warm/measured publications; handle/thread/private-byte checkpoints; lifecycle cycles;
  artifact count/bytes; and process cleanup.

## Acceptance Criteria

- Neither current startup-failure report may contain `readinessEmitted`.
- Readiness-only capture must return only label, process ID, elapsed time, and the positive readiness
  value; it must fail on stderr or any post-readiness output.
- No downstream current owner may read or summarize the deleted fields.
- The existing session guard must reject restoration in all four current ownership locations.
- Prototype and Runtime reports must contain zero occurrences of all four retired field names while
  every existing current gate passes.
- Product, Runtime, renderer/GPU, source, synchronization, resource behavior, schema, and process
  count diffs must remain empty.

## Results

The cleanup deleted two `readinessEmitted` report fields, four readiness-only capture fields, two
downstream field reads, and one summary field: nine report/check occurrences in total. The capture
now drains and rejects extra stdout after forced termination and fails directly on stderr. Both
startup-failure owners retain their current positive-output rejection and exact diagnostic payload.
The existing guard scans all four owners and rejects restoration; no replacement field or
compatibility path exists.

`canonical-prototype-v60` passed on its first run in 175.222 seconds with a 453,981-byte report.
The first, restarted, and object-action baseline captures each contain exactly
`label/processId/elapsedMs/readiness`; missing and corrupt starts each contain exactly
`label/code/elapsedMs/stdout/stderr`. The report contains zero retired field names. All 103
engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero denies and
five existing warnings.

`canonical-runtime-v20` passed in 328.876 seconds with a 7,529,525-byte report and zero retired
field names. It recorded 1,056 Sidecar invocations, four warm and eight measured publications,
stable 502 handles and 23 threads, private bytes `411,713,536 -> 412,483,584`, 2/2 lifecycle
cycles, and 24 artifacts totaling 25,346,240 bytes.

No product, Runtime, renderer/GPU, source, synchronization, resource, or process-count owner
changed. Workspace-local compiler/generated resources were intentionally retained because 0145 is
not a scheduled resource-cleanup boundary.

## Conclusion

Accepted. Forced/readiness-only evidence now reports only positive current state and live failure
diagnostics. Completion absence, empty trailing output, empty stderr, and forced termination status
are enforced at their execution boundary rather than copied through acceptance reports.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :canonical-runtime
runseal :init
```

Generated reports remain ignored under `out/captures/`.
