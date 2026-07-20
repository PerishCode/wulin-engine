# ADR 0165: Retired Graceful Transport Report Aliases

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0162 Retired Graceful Transport Report Aliases

## Context

Every successful graceful Prototype launch repeats five host transport results:
exit code zero, empty stderr, the caller-owned expected exit reason, output value
count two, and empty trailing output. Every paired invariant additionally emits
`exactlyTwoValues: true`.

These values were introduced with bounded session completion. The current
runner now enforces all of them before it can return: exact readiness and
completion reads, bounded exit wait, successful status, stdout drain to EOF,
empty trailing output, and empty stderr. Product completion already owns the
semantic outcome and reason.

Across 17 Escape sessions, one window-close session, and all 18 paired
invariants, the aliases occupy exactly 2,022 minified UTF-8 bytes.

## Decision

- Delete the five raw transport result aliases and the paired two-value flag.
- Delete the boundary's redundant report-level exit-code check.
- Rename the live expected-reason parameter so the retired report spelling is
  absent.
- Require the real successful-status, stderr, and trailing-output checks in the
  existing central guard and reject restoration of every retired result shape.
- Advance complete Prototype acceptance from v76 to v77.
- Add no replacement field, compatibility alias, optional decoder, fallback,
  retry, product delay, process change, or relaxed transport gate.

## Consequences

- Raw graceful reports retain the evidence that varies: label, exact PID,
  elapsed time, native input, readiness, and completion.
- Transport success remains enforced once by execution control flow rather than
  copied into every raw launch and paired invariant.
- Product completion remains the sole report owner for outcome and exit reason.
- Current product behavior and all 18 process lifetimes remain unchanged.

## Evidence

`canonical-prototype-v77` passed in 156.947 seconds. Its 372,526-byte report is
3,109 bytes (0.83%) smaller than the 375,635-byte v76 baseline and exceeds the
measured 2,022-byte minified alias weight by 1,087 bytes.

All 18 raw launches retained unique positive PIDs plus exact native input,
readiness, and completion. Product completion retained 17 `escape` reasons and
one `window-close`; all five retired raw fields and the 18 paired
`exactlyTwoValues` flags were absent. The fixed pairwise copy count remained
zero.

Activated recovery retained exact 12 Activated frames, 16 suppression frames,
one commit, cleared target/acknowledgement, and zero render blocks. Every other
current Prototype behavior gate and Sidecar lifecycle passed. All 103
engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor
retained zero denies and five existing warnings. No product, Runtime,
renderer/GPU, source, synchronization, process-count, or resource ownership
changed.
