# Experiment 0059: Bounded Host Activation

Status: Accepted

## Hypothesis

The concrete reference window can transport Win32 focus loss/resume as a bounded canonical
transition batch that preserves both final focus state and any between-drain interruption, without
an event backlog, HostClock mutation, application-loop consumption, or Runtime work.

## Scope

Add one reference-host activation reducer behind `window::drain_activation`. `WM_KILLFOCUS` and
`WM_SETFOCUS` update its current native state. Each drain returns zero, one, or two ordered
`HostActivation` values that are semantically equivalent to every native transition since the last
drain.

Do not change input normalization, connect HostClock, alter either application loop, add inspect
controls, sample elapsed, invoke Runtime, or add a cross-platform host abstraction.

## Workload

1. Require the first drain to publish the initial suspended state and a repeated drain to be empty.
2. Drive single resume/loss transitions and suppress duplicate native states.
3. From delivered active state, drive loss+resume before drain and require exactly
   `[Suspended, Resumed]`.
4. From delivered suspended state, drive resume+loss before drain and require exactly
   `[Resumed, Suspended]`.
5. Drive longer alternating bursts and require an order-equivalent batch of at most two transitions
   with the exact final state.
6. Reset the reducer and require the initial suspended contract again.
7. Replay a fixed native/drain sequence twice and require identical batches and SHA-256.
8. Run focused reference-host tests, `runseal :init`, and `runseal :guard`. Do not run process/GPU
   workflows because no application consumes the transport.

## Controlled Variables

- Existing `NativeMessage::FocusLost` input cleanup remains unchanged.
- Win32 message pumping and both application loops remain unchanged.
- Activation reduction stores constant state and flags, never a native-message queue.
- Host clock, simulation schedule/body, frames, presentation, and GPU work remain unchanged.

## Metrics

- Ordered activation values per drain and maximum batch length.
- Delivered/current focus equality after every drain.
- Duplicate suppression, reset state, replay equality, and evidence SHA-256.
- Focused test count, Flavor denies, and guard result.

## Acceptance Criteria

- Every drain is empty or contains at most two canonical ordered transitions.
- Single transitions, duplicates, interrupted same-final-state bursts, and longer bursts preserve
  the exact delivered/current focus contract.
- Reset restores the first-drain suspended state.
- Two deterministic replays produce identical batches and SHA-256.
- Focused tests, `runseal :init`, and `runseal :guard` pass without process or canonical workflows.

## Reference Environment

The experiment uses the pinned Rust toolchain and concrete Windows reference-host crate. No
generated runtime or GPU evidence is required.

## Evidence

Five private reference-host tests cover initial/single/duplicate states, both interrupted orderings,
reset, deterministic replay, and an exhaustive burst sweep. The sweep drives every focus-state
sequence of length one through eight from both delivered states. Each drain returned the exact
expected 0–2 transition batch, applied to the exact final state, and left the following drain empty.
All 19 reference-host tests pass.

The fixed replay preserved initial suspension, simple resume/loss, active loss+resume as
`[Suspended, Resumed]`, and suspended resume+loss as `[Resumed, Suspended]`. Two runs produced
identical typed batches with SHA-256
`eed23eab9230c591d895eaede20bbe19284a0bf309302b8d692ed8c1029738f1`.

`runseal :init` and `runseal :guard` passed with zero Flavor denies. No process or canonical workflow
was run because `drain_activation` remains unconsumed and frame/GPU/resource/synchronization/
lifecycle behavior is unchanged.
