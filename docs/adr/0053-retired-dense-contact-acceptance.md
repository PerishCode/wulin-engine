# ADR 0053: Retired Dense Contact Acceptance

- Status: Accepted
- Date: 2026-07-15
- Supersedes: ADR 0049's live dense diagnostic decision
- Superseded by: None

## Context

ADR 0049 accepted exact caller-owned terrain contact using one explicit 230,400-body checkpoint and
one 225-body routine transition witness. Its stated intent was to keep dense work behind an explicit
acceptance action rather than repeat it in generic probes.

The checkpoint is settled in Experiment 0046, but its full inspect/runtime/renderer branch remains
live and the current canonical workflow invokes it every run. That executable history adds a second
coverage mode, an operator verb, runtime plumbing, support assertions, and recurring work after the
resolver and bounded witness are already accepted.

Experiment 0050 is the required divisible-by-five cleanup milestone and tests complete removal of
that historical acceptance surface.

## Decision

- Delete the dedicated dense contact inspect verb and its workbench, runtime, renderer, and support
  call chain without replacement.
- Collapse terrain query/contact probe implementation to its sole current 225-body witness; remove
  coverage mode selection, dense revision/count branches, and conversion plumbing.
- Preserve direct exact contact, invalid-input gates, and the witness's accepted counts, hashes, and
  transition/lifecycle role.
- Treat Experiment 0046's dense report as sufficient historical evidence. Do not keep a feature
  flag, hidden command, deprecated alias, or offline reimplementation.
- Add both process-level `unknown_event` evidence and a repository forbidden-symbol gate so the
  retired surface cannot silently return.

## Consequences

- The live runtime and canonical workflow contain one regression-sized contact witness instead of
  both current and historical coverage modes.
- Routine validation performs 76,800 fewer duplicate terrain queries and 230,400 fewer contact
  resolutions while preserving the generic 76,800-query sweep and its 225 contact transitions.
- Reproducing the old dense checkpoint would require a new experiment with a current hypothesis;
  historical hashes remain available for reasoning but not as a compatibility contract.
- Frame, renderer, GPU resource, lifecycle, contact semantics, and public contact API remain
  unchanged, so focused process and repository gates are proportionate acceptance evidence.

## Evidence

Experiment 0050 removed the complete dedicated path and reduced live executable/support code by 46
lines after adding a stable recurrence guard. All 37 focused runtime/workbench tests passed. A
9.33-second fresh-process gate proved the retired verb returns `unknown_event` before and after
publication, all three direct contact classes remain exact, and the 225-body witness retains 75
results per class, 75 corrections, zero mismatch, result SHA-256
`2cd0d7110b580d58d3835f38e44a77ed3339ba028225e6cf7e2a5da590464306`, and identity SHA-256
`16446f145eecf59e79dda0a30f8193bf9240b5d93103b95c7c0c6c4aa7e15c9a`.

`runseal :guard` passed with zero Flavor denies and now rejects recurrence of the old live symbols.
The compacted canonical wrapper retains the unknown-event and bounded-witness gates. Because no
frame, renderer, GPU resource, or lifecycle execution changed, the full workflow was not repeated
solely for this CPU diagnostic deletion.
