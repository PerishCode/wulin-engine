# ADR 0141: Held Camera Focus Cleanup

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0138 Held Camera Focus Cleanup

## Context

Experiment 0137 established that a fresh E-down ordered before FocusLost in one ingest remains one
sample-scoped camera press. A distinct path exists when E was already held before the ingest:
HostInput suppresses duplicate down transitions, while FocusLost releases every held key. The
camera policy consumes only pressed edges, so this cleanup path should not repeat the prior camera
action.

## Decision

- Retain HostInput duplicate-down suppression for an already-held Q/E key.
- Retain focus cleanup as the sole transition in that ingest: held becomes false, pressed remains
  false, and released becomes true.
- Require the Prototype camera candidate and commit to retain the existing orbit when no new press
  exists.
- Keep the fresh press/focus-loss rule from ADR 0140 unchanged.
- Add no simulation sample input, pending camera intent, controller, queue, report, compatibility
  path, or native process.

## Consequences

- A focus loss cannot manufacture a repeated camera step from an already-held key or its duplicate
  native keydown message.
- Fresh presses and held-key cleanup remain distinct, exact HostInput states.
- Product HostInput, camera policy, main-loop ordering, Runtime, renderer/GPU resources, sources,
  synchronization, process count, and report schema remain unchanged.

## Evidence

All five focused camera tests passed. The new composition test proved the initial orbit `0 -> 1`
commit, then exact held=false/pressed=false/released=true facts for duplicate E-down plus
FocusLost, followed by an orbit-1 cleanup candidate and commit.

`canonical-prototype-v53` passed on its first run in 170.211 seconds with the unchanged native
session/process matrix and a 447,346-byte report. All 103 engine-runtime, 47 Prototype, and 20
reference-host tests passed; Flavor remained at zero denies and five existing warnings.
