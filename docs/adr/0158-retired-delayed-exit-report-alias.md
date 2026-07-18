# ADR 0158: Retired Delayed-Exit Report Alias

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0155 Retired Delayed-Exit Report Alias

## Context

The current object-action acceptance path already validates exact native message ordering,
requested exit delay, measured exit interval, PID/window/thread ownership, and graceful process
completion. Its sustained-consumption, Activated recovery, and Rejected projections nevertheless
restated those facts through three `delayedExit` booleans. No semantic or product consumer read the
field.

## Decision

- Delete all three `delayedExit` report producers.
- Retain both current private validator modes and every exact message/delay/interval check.
- Rename the internal validator expectation parameter so the retired report spelling has no
  executable alias.
- Make the existing 500-line Prototype session guard scan both current object report owners and
  reject restoration.
- Advance complete Prototype acceptance directly to v70.
- Add no replacement flag, compatibility alias, optional decoder, registry, fallback, retry,
  telemetry, product delay, or relaxed threshold.
- Change no product, Runtime, renderer/GPU, source, synchronization, resource, or process-count
  owner.

## Consequences

- Reports expose exact ordered messages and exit intervals without a derived boolean.
- Sustained consumption remains non-terminal; Activated recovery and Rejected action retain their
  bounded delayed Escape.
- The historical report field has one static absence authority and no compatibility path.
- Experiment 0155 performs no workspace resource cleanup; the scheduled resource boundary remains
  Experiment 0160.

## Evidence

`canonical-prototype-v70` passed first-run in 172.820 seconds. Its 459,410-byte report contains zero
retired-field occurrences, 81 bytes smaller than the 459,491-byte v69 baseline.

Sustained consumption retained exact F/Enter ordering with a null exit interval. Activated recovery
retained Enter-up/F-down/Enter-down/Escape with a 271.2995 ms interval. Rejected action retained
F/Enter/Escape with a 269.0739 ms interval. The three current children preserved their existing
committed/ineligible counts, 12-frame Activated/Rejected lifetimes, suppression evidence, zero
render blocks, exact source-qualified identities, and clean completion.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor retained zero
denies and five existing warnings.
