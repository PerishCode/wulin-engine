# ADR 0066: Live Prototype Time Driver

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

Prototype now owns one exact grounded retained body, while reference-host owns composed activation
and bounded elapsed admission and Runtime owns one explicit schedule/body dual transaction. These
accepted boundaries still have no live composition root. Adding movement at the same time would
make clock ordering, stall disposition, simulation commit, and gameplay tuning inseparable.

The prototype has no inspect endpoint. Its one readiness line must therefore carry direct evidence
that the live driver reached a meaningful terminal boundary rather than only reporting module state.

## Decision

- In prototype, order each live iteration as message pump, input/exit, activation-aware HostClock
  sample, optional Ready-only dual advance, then frame.
- Use the retained generation handle with an all-zero simulation command. This consumes real time
  and exact terrain contact while introducing no movement tuning.
- Treat Reset, Suspended, and Stalled as no-advance outcomes. HostClock advances the stall baseline;
  the prototype neither catches up nor terminates solely for a stall.
- Keep Runtime advance and frame failures terminal.
- Publish prototype readiness once, after the first nonzero dual commit and its following successful
  frame. Include sample, clock, dual result, zero command, initial body, and live frame count.
- Keep workbench, bootstrap schema, input mapping, gravity, camera, actor/presentation, and renderer
  ownership unchanged.

## Consequences

- Prototype becomes the first real wall-time simulation composition root while remaining visibly
  behavior-neutral.
- Real elapsed/remainder values may differ across launches; structural transaction and body
  invariants become the deterministic acceptance surface.
- Gravity and locomotion can be evaluated later over proven live time/commit ordering rather than
  bundled with host integration.

## Evidence

Experiment 0063 added one prototype-only typed admission test and live structural assertions to the
existing no-inspect lifecycle gate. A 50.75-second fresh-cook run proved two distinct direct
processes reached readiness only after bounded Ready admission, a nonzero tick-zero dual commit with
one query per step and unchanged body motion, and the following successful frame. Reset, suspended,
and stalled no-advance policy is covered by the focused test.

Invalid/missing/corrupt inputs emitted no readiness, Sidecar start/restart/stop remained exact, and
final PID state was empty. `runseal :init` and `runseal :guard` passed with zero Flavor denies. The
full canonical workflow was not run because workbench GPU/resource/synchronization ownership and
evidence were unchanged.
