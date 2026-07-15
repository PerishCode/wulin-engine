# ADR 0061: Bounded Host Elapsed Clock

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

Runtime accepts explicit bounded elapsed nanoseconds but deliberately does not observe wall time.
Connecting a frame loop before defining pause and stall behavior would mix clock policy, window
transport, and simulation mutation into one proof. The concrete reference host is the narrow owner
for platform monotonic sampling policy.

## Decision

- Add one `reference-host::HostClock` state machine around `Instant`.
- The first active sample and first sample after resume establish a baseline and emit `Reset`.
  Active deltas at or below the simulation's 125 ms maximum emit exact nanoseconds.
- A delta above the maximum emits `Stalled` with its exact value and advances the baseline. It is
  never clamped, queued, split, or exposed as ready elapsed.
- Suspension drops the baseline and accumulates no elapsed time. Suspend/resume are idempotent, and
  resume requires a fresh reset sample.
- A monotonic regression or numeric overflow fails before mutation. Typed outcomes and status expose
  the complete policy without exporting a controllable production clock.
- Keep Win32 focus transport, composition-root sampling, Runtime invocation, input commands,
  backlog/catch-up, and presentation binding deferred to independent experiments.

## Consequences

- The host now has a deterministic admission policy for future live elapsed samples, but neither
  composition root consumes it and Runtime remains explicitly driven.
- A stall cannot poison later samples or silently accelerate simulation. Time while suspended cannot
  leak into a resumed schedule.
- Focus transport and live driving must prove their own ordering and rollback before this clock
  becomes an application dependency.

## Evidence

Experiment 0058 added four private reference-host tests covering the exact zero, nominal, maximum,
and maximum-plus-one boundaries; immediate post-stall recovery; idempotent suspend/resume and resume
reset; monotonic-regression rollback; and deterministic replay.

All 14 reference-host tests passed. The fixed replay evidence SHA-256 is
`3a873571ca7a754272eeaecb0dc7fe9d5183703e88a100a1907cc9ae8bacea7d`.
`runseal :init` and `runseal :guard` passed with zero Flavor denies. No process or canonical workflow
was run because the state machine has no live frame, GPU, resource, synchronization, or lifecycle
consumer.
