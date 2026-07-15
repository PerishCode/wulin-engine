# ADR 0062: Bounded Win32 Activation

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The reference window already maps `WM_KILLFOCUS` into input cleanup, but it has no focus-resume
authority for a future live clock. Queueing every native transition creates a new backlog. Publishing
only the latest focus boolean loses a loss/resume interruption between application drains and could
allow a stale elapsed baseline to survive.

## Decision

- Let the concrete reference window capture both `WM_KILLFOCUS` and `WM_SETFOCUS` into a private
  activation reducer, independently from the existing input queue.
- Track current native focus, last delivered focus, and whether either changed. Do not store native
  activation messages.
- The first drain publishes current state. An unchanged later drain is empty, an opposite final
  state emits one transition, and a burst returning to the delivered state emits the two ordered
  transitions that preserve its interruption.
- Bound every drain to at most two typed `HostActivation` values regardless of native burst length.
- Reset activation state on window creation and teardown.
- Keep HostClock mutation, application-loop consumption, inspect controls, Runtime invocation, and
  cross-platform host abstraction deferred.

## Consequences

- A later composition root can observe both final focus state and an otherwise-hidden interruption
  without accepting an unbounded event queue.
- Activation transport does not alter key normalization: `WM_KILLFOCUS` still releases held keys
  through the existing input authority.
- The transport alone neither samples time nor defines how activation transitions drive HostClock.

## Evidence

Experiment 0059 added five private reference-host tests. They cover initial/single/duplicate states,
both interrupted orderings, reset, deterministic replay, and every focus-state sequence of length
one through eight from both delivered states. Every burst reduced to the exact expected 0–2
transition batch and final state.

All 19 reference-host tests passed. The deterministic activation replay SHA-256 is
`eed23eab9230c591d895eaede20bbe19284a0bf309302b8d692ed8c1029738f1`.
`runseal :init` and `runseal :guard` passed with zero Flavor denies. No process or canonical workflow
was run because neither application consumes activation transport.
