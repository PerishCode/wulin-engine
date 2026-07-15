# ADR 0060: Transactional Simulation-Body Advance

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The accepted simulation schedule and retained-body batch each commit atomically, but sequencing their
public operations cannot guarantee joint rollback. `SimulationSchedule` is already a small `Copy`
value, so a generic reservation/token API is unnecessary. Runtime can prepare a schedule copy and
local body output, then commit both only after all body work succeeds.

## Decision

- Add one handle-addressed operation over explicit caller-supplied elapsed nanoseconds and one
  controlled per-tick spatial command.
- Validate/copy the body, advance a schedule copy, execute exactly its emitted 0..=8 steps locally,
  then replace the body slot and assign the prepared schedule. Return both outputs together.
- A zero-step elapsed submission commits rational schedule remainder and exact body identity with no
  terrain query. Any handle, elapsed, query, contact, or arithmetic failure commits neither state.
- Preserve actual schedule submission counts: different elapsed partitions may have different
  successful-advance counts while agreeing on tick, remainder, emitted steps, and body state.
- Do not sample wall time, call from frames, define focus/stall/backlog policy, sample input, or bind
  actors/presentation.

## Consequences

- Runtime now has one safe explicit-time spatial transaction; schedule and retained body cannot
  diverge through a partial commit.
- A later live driver still needs its own host clock, elapsed partition, focus/stall, and input policy.
  This decision does not authorize frame-loop invocation.

## Evidence

Experiment 0057 added three private tests for fractional zero-step preparation, coarse/nominal
partition equality, and body-failure schedule-copy preservation. All 62 engine-runtime tests and the
workbench check passed with zero Flavor denies.

The 32.59-second three-process gate proved a one-nanosecond zero-step commit, eight 125 ms submissions
and the exact 60-call nominal partition both ending at tick 60/remainder 0 with byte-identical body
motion and 60 terrain queries, and correct submission counts of 8 versus 60. Evidence SHA-256 is
`d816aa37f7c5ad56d4bfe3c9d062dec4dda276ed9b3e51c838f9a29fa7027c8a`. A valid prepared seven-step
schedule batch whose body failed at step 3 outside the snapshot, plus a step-1 velocity overflow,
both preserved exact schedule and body state. Presentation/source/GPU/frame/renderer/synchronization
evidence remained unchanged or zero. `runseal :init` and `runseal :guard` passed. The long canonical
workflow was not run because production frames, GPU resources, synchronization, and lifecycle paths
remain unchanged.
