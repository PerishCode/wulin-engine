# ADR 0148: Retired Forced Process Reports

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0145 Retired Forced Process Reports

## Context

Two terminal startup-failure reports still copied `readinessEmitted=false`. The shared
readiness-only Prototype capture copied forced exit status, empty stderr, empty trailing output,
and `completionEmitted=false`; its caller then rechecked two of those fields and copied another
false summary. These values either repeated direct execution checks or exposed incidental forced
termination metadata.

## Decision

- Delete all nine report/check occurrences from the two failure owners, readiness-only capture, and
  downstream session gate.
- Keep exact startup failure code/stdout/stderr diagnostics and direct rejection of successful
  readiness/session output.
- After readiness-only termination, drain stdout and fail on every extra byte; also fail directly
  on nonempty stderr.
- Return only label, process ID, elapsed time, and the positive readiness value.
- Extend the existing Prototype session guard to reject every retired field in the four current
  owners.
- Advance complete Prototype and Runtime workflows to v60 and v20.
- Add no replacement negative flag, decoder, alias, retry, telemetry, product behavior, or
  Runtime/GPU/source/resource ownership.

## Consequences

- Forced/readiness-only reports contain only positive evidence and current startup diagnostics.
- Normal graceful sessions retain their real two-value framing, exit status, stderr, output count,
  and dynamic trailing-output check.
- Forced exit codes are no longer treated as a portable acceptance contract.
- The next resource cleanup remains scheduled for Experiment 0160.

## Evidence

`canonical-prototype-v60` passed in 175.222 seconds with a 453,981-byte report. Its three
readiness-only captures expose exactly `label/processId/elapsedMs/readiness`; missing/corrupt
failures expose exactly `label/code/elapsedMs/stdout/stderr`. All four retired field names occur
zero times. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor
retained zero denies and five existing warnings.

`canonical-runtime-v20` passed in 328.876 seconds with a 7,529,525-byte report and zero retired
field names. It recorded 1,056 Sidecar invocations, four warm/eight measured publications, stable
502 handles/23 threads, private bytes `411,713,536 -> 412,483,584`, 2/2 lifecycle cycles, and 24
artifacts / 25,346,240 bytes.
