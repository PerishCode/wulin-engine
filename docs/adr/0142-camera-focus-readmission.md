# ADR 0142: Camera Focus Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0139 Camera Focus Readmission

## Context

ADR 0140 retains one fresh camera press ordered before focus loss. ADR 0141 prevents focus cleanup
of an already-held key from repeating the prior action. The remaining lifetime boundary is
readmission: after FocusLost clears held state, a later native keydown must become a fresh press
rather than remaining suppressed by stale state.

## Decision

- Retain FocusLost as a complete held-key lifetime boundary: it clears held E, emits release, and
  creates no new press.
- Admit a later E-down on the same HostInput owner as held=true, pressed=true, and released=false.
- Require the Prototype camera candidate and commit to advance exactly one further clockwise orbit.
- Keep the fresh press/focus-loss and held duplicate-down rules from ADRs 0140 and 0141 unchanged.
- Add no simulation sample input, pending camera intent, controller, queue, report, compatibility
  path, or native process.

## Consequences

- Focus cleanup prevents repetition without leaving Q/E stuck or requiring an owner reset.
- The next physical down transition starts one new sample-scoped camera action.
- Product HostInput, camera policy, main-loop ordering, Runtime, renderer/GPU resources, sources,
  synchronization, process count, and report schema remain unchanged.

## Evidence

All six focused camera tests passed. The new composition test proved the initial orbit `0 -> 1`
commit, release-only FocusLost cleanup with orbit 1 retention, then exact
held=true/pressed=true/released=false facts and an orbit `1 -> 2` commit for the later E-down.

`canonical-prototype-v54` passed on its first run in 175.160 seconds with the unchanged native
session/process matrix and a 447,322-byte report. All 103 engine-runtime, 48 Prototype, and 20
reference-host tests passed; Flavor remained at zero denies and five existing warnings.
