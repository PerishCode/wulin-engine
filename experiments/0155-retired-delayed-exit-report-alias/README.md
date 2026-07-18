# Experiment 0155: Retired Delayed-Exit Report Alias

Status: Accepted

## Hypothesis

The three `delayedExit` object-action report fields can be deleted without weakening native input
or process-completion authority. Each boolean is uniquely restated by the retained exact ordered
messages and exit interval after the validator has already enforced the requested delay.

## Scope

- Delete `delayedExit` from the sustained-consumption, Activated recovery, and Rejected action
  report projections.
- Keep both current validator modes: sustained consumption continues without Escape, while the two
  terminal actions post Escape after the existing bounded dwell.
- Rename the private validator expectation parameter so the retired report token has no executable
  alias.
- Extend the existing 500-line Prototype session guard across both current object report owners.
- Advance the complete Prototype workflow revision from v69 to v70.

Native input messages, timing bounds, process/window/thread selection, product output, object
policy, session schema, Runtime, renderer/GPU resources, source formats, synchronization, process
count, and workspace resources are out of scope. The next scheduled resource cleanup remains
Experiment 0160.

## Workload

1. Inventory every producer and consumer of `delayedExit`.
2. Record the v69 report count, exact adjacent evidence, and byte baseline.
3. Delete all report producers without adding a replacement field, alias, or decoder.
4. Retain the private two-mode validator and every exact message/delay/interval check.
5. Make the stable current-owner guard reject restoration.
6. Run formatting, type checking, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :init`.
7. Inspect the v70 report for zero retired tokens and unchanged object/process outcomes.

## Controlled Variables

- Sustained consumption still posts F/Enter without Escape and requires a null exit interval.
- Activated recovery still posts Enter-up/F-down/Enter-down/Escape after the missing-target hold
  and requires a 250..=750 ms exit interval.
- Rejected action still posts F/Enter/Escape and requires the same bounded exit interval.
- Every source-qualified identity, object count, feedback lifetime, frame count, native thread
  batch, graceful completion, and cleanup gate remains live.
- No compatibility registry, optional-field decoder, replacement interpretation flag, retry,
  fallback, telemetry, product delay, relaxed threshold, or resource cleanup is admitted.

## Metrics

- Retired source/report occurrence counts; report bytes; workflow duration; exact ordered messages,
  batch spans, and exit intervals; Activated/Rejected/sustained object and frame outcomes; test
  totals; Flavor findings; and process cleanup.

## Acceptance Criteria

- Neither current object-action report owner may emit or reference `delayedExit`.
- The only current source spelling must be the central guard's forbidden token.
- The v70 report must contain zero retired-field occurrences.
- Sustained consumption must retain F/Enter without Escape and a null exit interval.
- Activated recovery and Rejected action must retain their exact Escape-bearing sequences and
  250..=750 ms exit intervals.
- Existing object identities, action counts, 12-frame feedback lifetimes, suppression, process
  completion, tests, and stable guards must pass unchanged.
- Product, Runtime, renderer/GPU, source, synchronization, resource, and process-count diffs must
  remain empty.

## Results

The v69 audit found exactly three report occurrences and no semantic consumer. Sustained
consumption reported `false` beside F/Enter without Escape and a null interval. Activated recovery
and Rejected action reported `true` beside exact Escape-bearing sequences and bounded numeric
intervals. All three aliases were deleted. The private validator still receives one explicit
expectation and enforces both current behaviors; its parameter was renamed without exporting a new
field. The existing 500-line guard now scans both object report owners and rejects the retired
spelling.

`canonical-prototype-v70` passed on its first full run in 172.820 seconds. Its 459,410-byte report
contains zero `delayedExit` occurrences, down by 81 bytes from the 459,491-byte v69 baseline.

- Sustained consumption used thread 29224, a 0.0013 ms batch and key interval, exact
  F/Enter-without-Escape ordering, and a null exit interval.
- Activated recovery used thread 32040, a 0.0023 ms atomic batch, 0.0012/0.0011 ms key intervals,
  and a 271.2995 ms exit interval after the exact Enter-up/F-down/Enter-down/Escape sequence.
- Rejected action used thread 30680, a 0.0012 ms batch and key interval, exact
  F/Enter/Escape ordering, and a 269.0739 ms exit interval.

The Activated child retained one committed and one ineligible action, 12 Activated frames, two
suppression frames, 346 live frames, and zero render blocks. The Rejected child retained one
ineligible action, 12 Rejected frames, 90 live frames, and zero render blocks. The sustained child
retained one committed and one post-readiness ineligible action, 12 Activated and 12 capacity
Rejected frames, 233 suppression frames, 309 live frames, and zero render blocks.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor retained zero
denies and five existing warnings. No Prototype, Sidecar, Wulin, or Runseal process remained.

## Conclusion

Accepted. Exit behavior remains enforced by exact native messages and bounded intervals at the
current validator, while the historical boolean restatement has no report, alias, decoder, or
fallback surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
