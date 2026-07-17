# ADR 0111: Bounded Prototype Session Completion

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0108 Bounded Prototype Session Completion

## Context

Prototype readiness is deliberately emitted once, on the first successful nonzero simulation
commit and canonical frame. The maintained acceptance reader terminates the process after that
line, so no exact live contract can prove a second action, movement after target acquisition,
acknowledgement completion, or other post-readiness behavior. Repeated readiness, an inspect
route, or periodic state output would recreate diagnostic telemetry that earlier cleanup removed.

## Decision

- A successful Prototype session has at most two stdout values: sequence-one canonical readiness
  and one sequence-two completion after graceful exit.
- Completion is emitted only after the message loop ends and the Runtime is idle. It carries the
  same process identity, the exact exit reason, final actor and host-clock state, bounded frame
  counters, and current object observation/interaction state already owned by the Prototype.
- Completion is a terminal immutable value. There is no event stream, periodic snapshot, request
  path, inspect verb, retained event history, replay surface, or product artifact write.
- Bootstrap failure, fatal runtime failure, and forced process termination do not emit successful
  completion.
- The first sustained consumer proves one post-readiness capacity-exhausted action: the already
  consumed qualified identity and committed count remain exact while the existing ineligible
  count advances once.

## Consequences

- Real multi-event Prototype behavior can be accepted without turning Runtime or the renderer into
  product-observation owners.
- Session completion is bounded by process lifetime and cannot drive or query a running session.
- This decision does not authorize a general telemetry schema, action result history, registry,
  inventory, rewards, dispatch, respawn, persistence, networking, or Wulin semantics.

## Evidence

Two pure report tests cover both graceful reasons, the exact final capacity-one state, and checked
frame-total overflow. `canonical-prototype-v25` passes in 81.332 seconds: forced readiness and
finite-boundary termination emit no completion; normal Escape emits exactly two values; and one
sustained process moves from live frame 5 to 970 while a post-readiness Enter changes only the
ineligible count from zero to one. The consumed qualified ID 496 remains exact, acknowledgement
and target are empty, suppression projects for 954 frames, and no event history or copied object
state is reported.

`runseal :guard` passes with zero Flavor denies after the Prototype object policies were grouped
under one source owner and the former process helper was absorbed into the session acceptance
owner. `canonical-runtime-v16` passes in 252.427 seconds with source/window, rollback, traversal,
resource, and lifecycle gates unchanged; 5 warm/8 measured publications retain 495 handles and 21
threads with +1,257,472 private bytes, and the 24-file report contains 25,346,292 bytes.
