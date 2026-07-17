# ADR 0135: Native Forward Release

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0132 Native Forward Release

## Context

Current real-process sessions proved held locomotion, Run modifier release/re-press, opposite-axis
release, and focus-loss cleanup, but none proved a normal direction-key release returning Walk to
Survey. The maintained plain Escape process duplicated exit/session framing already exercised by
every graceful session and exposed no distinct product state.

## Decision

- Replace the plain idle Escape session with exact-PID post-readiness W-down, delayed W-up, delayed
  Escape, and one final completion oracle.
- Require at least 250 ms of held Walk and at least 250 ms of stationary work after release.
- Atomically post the initial W press to the exact visible window thread and retain schema 4.
- Require exact 32-Q9 negative-Z movement followed by final Survey clip 0 with retained committed W
  yaw 49,152, stable actor identity/region, continuous clock, idle object state, zero render blocks,
  and exactly two output values.
- Delete the old `escape`/`escapeInvariant` report shape directly; add no alias or extra process.

## Consequences

- Normal `WM_KEYUP:W` now has a live product completion witness distinct from focus cleanup and
  modifier/opposite-axis release.
- Plain idle Escape no longer owns a standalone report field, while Escape exit and the two-value
  contract remain exercised by the replacement and all other graceful sessions.
- Stationary presentation is explicitly understood to retain the last successfully advanced
  facing; it does not reset to spawn yaw.
- Product input, locomotion, presentation, session schema, Runtime, renderer/GPU resources, source
  formats, synchronization, and process count remain unchanged.

## Evidence

The first v47 run rejected an oracle that expected final Survey yaw 0. Existing product tests and
Experiment 0078 require committed W yaw 49,152 to persist while stationary, so only the oracle was
corrected.

Final `canonical-prototype-v47` passed in 144.561 seconds. PID 20436 received the exact W-down,
W-up, Escape sequence on thread 10864 with 255.9837 ms held and 252.1398 ms stationary intervals.
The actor completed 15 exact 32-Q9 steps at `(0,-480)`, retained its identity/region, and finished
as Survey clip 0/yaw 49,152 with zero vertical velocity and zero render blocks. All 103
engine-runtime, 45 Prototype, and 20 reference-host tests passed; Flavor remained at zero denies
and five existing warnings, and the product/Runtime/GPU/source/resource diff was empty.
