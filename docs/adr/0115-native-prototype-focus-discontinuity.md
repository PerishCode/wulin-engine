# ADR 0115: Native Prototype Focus Discontinuity

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0112 Native Prototype Focus Discontinuity

## Context

The reference window maps `WM_KILLFOCUS` into both held-input cleanup and a Suspended activation.
The Prototype applies HostClock activation before current-batch actions; Jump, object observation,
and object interaction clear old pending intent on Suspended/Reset. HostClock resumes through Reset
before later Ready samples. These contracts have focused pure tests, but no real Prototype process
currently proves the complete native message-to-frame ordering without an inspect endpoint.

## Decision

- Maintain one exact visible-window/PID real-process gate for post-readiness focus loss and resume.
- Post W down before `WM_KILLFOCUS`, wait for suspended sampling, then post `WM_SETFOCUS`, wait for
  Reset/Ready recovery, and exit with Escape.
- Require exact actor-state equality across the discontinuity, one suspend/resume pair, suspended
  and reset evidence, later Ready progress, and no stale product action state.
- Keep the proof in bounded session completion; add no live query, recurring telemetry, journal, or
  replay surface.

## Consequences

- Native held-input cleanup, composed activation-before-sample ordering, and no-backlog resume gain
  one end-to-end product proof.
- The harness gains explicit suspend/resume window actions but no product control surface.
- This decision does not authorize background simulation, retained elapsed backlog, synthesized
  focus state, action history, engine input ownership, or Runtime/GPU/resource changes.

## Evidence

Experiment 0112 passed `canonical-prototype-v29` in 92.183 seconds. One exact visible Prototype
window received ordered `WM_SETFOCUS`, `WM_KEYDOWN:W`, and `WM_KILLFOCUS`, then a later
`WM_SETFOCUS`. Readiness-to-completion clock deltas were exactly one suspend, one resume, and one
reset with 635 suspended samples, later Ready progress, zero stalls, and no elapsed backlog. The
complete actor state remained exact, object policies remained idle, render blocks stayed zero, and
the process exited normally through the existing two-value Escape completion.
