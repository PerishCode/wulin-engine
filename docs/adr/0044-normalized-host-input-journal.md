# ADR 0044: Normalized Host Input Journal

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

Experiments 0039 and 0040 established one engine runtime facade and one runtime-owned presentation
frame transaction. The native workbench still discards keyboard and focus messages, so no later
simulation or interactive prototype stage has a proven input boundary. Attaching input directly
to camera movement would also choose sampling, speed, focus, and time-step semantics before those
dependencies have evidence.

Experiment 0041 tests the narrower prerequisite: host-native messages become deterministic,
bounded state transitions that can be replayed without renderer or runtime coupling.

## Decision

- The native host owns Win32 message capture and normalization. `engine-runtime` does not depend on
  Win32 input messages, a host event pump, or operator controls.
- One host input transaction is the ordered normalized result of one non-empty native queue drain.
  It is applied after message dispatch and before inspect commands or any runtime frame attempt.
- Keyboard identity is the bounded Win32 virtual-key domain `1..=255`. Repeated down and unmatched
  up messages are suppressed; focus loss releases all held keys in ascending key order.
- A bounded process-local journal records initial held state and normalized transactions without
  wall time, presentation tick, or frame index. Overflow is an explicit failed record.
- Replay uses an isolated synthetic key-state consumer and cannot mutate live input state. Camera,
  actions, simulation sampling, and persistent replay serialization require later experiments.

## Consequences

- Native input transport, ordering, focus cleanup, pause independence, and exact replay can be
  validated before choosing gameplay or simulation semantics.
- The first proof remains keyboard-only and workbench-local. A later thin-host experiment may
  promote or replace the adapter only when reuse is concrete.
- Process-local recording is diagnostic evidence, not yet a user-facing replay file or stable
  compatibility surface.

## Evidence

Experiment 0041 passed the 533.7-second direct workflow. Two controlled native queue drains in
each of two processes reproduced the same 11-message/10-transition record, exact stream and held
state hashes, ordered focus cleanup, and isolated replay while host rendering remained paused.
Five focused tests, all prior GPU hashes and correctness gates, 32 reactive plus 32 prepared
crossings, a zero-growth 64-publication resource plateau, 16 lifecycle cycles, and the repository
guard passed.

Generated evidence is ignored under
`out/captures/0041-deterministic-host-input/`.
