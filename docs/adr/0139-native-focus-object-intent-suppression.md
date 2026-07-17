# ADR 0139: Native Focus Object-Intent Suppression

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0136 Native Focus Object-Intent Suppression

## Context

The maintained focus-discontinuity process proved same-batch Jump and held-locomotion suppression.
Prototype observation and interaction policies also cancel capacity-one pending intents on
Suspended/Reset, but no real native process proved that F and Enter queued immediately before focus
loss could not reach resumed nonzero simulation.

## Decision

- Extend the exact-PID, exact-window-thread atomic focus batch to Space/F/Enter/W followed by
  `WM_KILLFOCUS`.
- Begin from grounded readiness with idle object policies and retain the existing bounded
  suspension, resume, Ready recovery, and two-value Escape completion.
- Require final actor state exactly equal to readiness and final observation/interaction state
  completely idle with zero committed and ineligible counts.
- Interpret the evidence as suppression across Suspended/Reset before resumed nonzero work; do not
  claim immediate edge deletion in the first ingest.
- Add no process, product report, retry, relaxed threshold, or policy behavior.

## Consequences

- One existing process now covers Jump, object observation, object activation, and held locomotion
  at the activation/time discontinuity.
- The exact four-key batch and idle final policies exclude an object query, action attempt,
  acknowledgement, consumption, or exclusion reaching resumed work.
- Product input/action/time behavior, session schema, Runtime, renderer/GPU resources, source
  formats, synchronization, and process count remain unchanged.
- An independent one-off invalid-key clock discontinuity remains an operator-run event, not a retry
  path or weakened acceptance condition.

## Evidence

The first v51 workflow passed the new focus session and later stopped at the independent
invalid-key clock-continuity oracle. No code or threshold changed. The unchanged full rerun passed
in 172.935 seconds.

PID 5004 / thread 25564 posted Space/F/Enter/W at 0.0014/0.0013/0.0012 ms intervals in a 0.0039 ms
atomic batch. Completion recorded one suspend/resume pair, one additional reset, 88 suspended
samples, 156 Ready samples, and 246 live frames. Actor state remained exactly readiness; object
pending/target/acknowledgement/consumption/exclusion were empty, committed/ineligible counts were
zero, and stalls/render blocks were zero. The report was 447,315 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. Flavor remained at zero
denies and five existing warnings.
