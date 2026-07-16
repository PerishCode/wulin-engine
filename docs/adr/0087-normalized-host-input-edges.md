# ADR 0087: Normalized Host Input Edges

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0084 Normalized Host Input Edges

## Context

The reference host already owns exact keyboard normalization: repeated downs and unmatched ups are
suppressed, focus loss releases held keys, and each non-empty native drain yields an ordered bounded
transaction. Applications can query only the final held set. Continuous locomotion fits that
surface, but a one-shot action must currently repeat on every loop while a key remains held or
duplicate previous-key and focus semantics outside the host.

The accepted transitions already contain the missing press/release facts. Exposing those facts is
the narrow prerequisite shared by lifecycle actions, camera steps, jump, and interaction without
selecting any of those later product policies or introducing another event queue.

## Decision

- `HostInput` owns two bounded virtual-key sets describing whether an accepted down or up transition
  occurred in the most recent `ingest` sample. Expose them through `was_pressed` and
  `was_released`, while `is_held` remains the final continuous state.
- Clear both edge sets at the start of every ingest, including an empty drain. An empty drain is a
  sample boundary but remains absent from the canonical journal and all transaction counters.
- Populate edges only from the existing normalized transitions. Same-drain down/up may make both
  queries true; duplicates, unmatched messages, invalid keys, and key zero do not. Focus cleanup
  produces release edges for previously held keys.
- Keep live diagnostic status, the v1 journal revision, and canonical stream/held hash domains
  unchanged. Do not latch an old edge merely so an asynchronous inspect request can observe it.
- Replay remains an isolated journal oracle and must not mutate held or sample-edge state.
- Prototype Escape consumes only its normalized press edge. W/A/S/D continues to consume held
  state. Host input does not own action meaning, buffering, frame/simulation binding, or gameplay.

## Consequences

- Applications gain deterministic one-shot keyboard facts without copying host normalization or
  retaining an unbounded/ordered event surface.
- Edge lifetime is deliberately one host ingest, not one rendered frame or fixed simulation step.
  Consumers run after ingest in the existing host order and must act in that sample.
- An asynchronous diagnostic request may observe no edge after the next empty ingest. Exact edge
  semantics belong to focused state tests and real application consumers, not a second latched
  status lifetime.
- A key can be both pressed and released in one sample. The queries preserve occurrence, while
  `is_held` preserves final state; consumers needing complete intra-sample order require a later
  explicit contract.
- This decision adds no action mapper, command buffer, persistent replay, pointer/gamepad/text
  transport, camera controller, gameplay interaction, or engine input system.

## Evidence

Experiment 0084 passed 24 reference-host, 16 prototype, and 77 engine-runtime tests. A dependency-free
native workbench witness reproduced the exact existing 2-transaction/11-message/10-transition record,
held hash, and stream hash in two processes while empty ingests created no journal work.

`canonical-prototype-v10` passed in 75.608 seconds. Its added real process reached readiness, received
native Escape / virtual key 27, and exited with code 0 in 4.316 seconds with empty stderr. Existing
native-W locomotion and 15.008-second finite-edge survival also passed. The initial inspect-based edge
probe observed the correct empty state after a later empty ingest, which rejected a proposed latched
status/revision surface before acceptance. The merge-checkpoint guard passed with zero Flavor denies.
