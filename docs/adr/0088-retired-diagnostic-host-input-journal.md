# ADR 0088: Retired Diagnostic Host Input Journal

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0085 Mandatory Host Input Journal Cleanup
- Supersedes: ADR 0044 Normalized Host Input Journal and ADR 0087 Normalized Host Input Edges

## Context

ADR 0044 established both normalized host keyboard state and a bounded process-local diagnostic
journal before any product consumed input. The journal, canonical hashes, replay oracle, native
workbench message posting, inspect controls, and operator commands proved transport/order/focus
semantics in Experiment 0041. That decision explicitly did not make recording a user-facing or
stable persistence surface.

Prototype v0 now directly consumes held W/A/S/D and sample-scoped Escape edges, with real native
process evidence. No product, bootstrap policy, simulation, runtime, renderer, or content path reads
the journal or its status. Keeping the proof scaffold live would preserve allocation, reverse native
message injection, diagnostic protocol, wrapper commands, and a long-workflow branch solely because
they once produced evidence.

## Decision

- Retain one reference-host `HostInput` containing only fixed held, pressed, and released virtual-
  key sets. Reduce each ordered native drain directly: ignore invalid/repeated/unmatched messages,
  release held keys on focus loss, and expire prior edges at every ingest.
- Keep one input owner across prototype bootstrap and live execution. This preserves held input
  received before readiness. Diagnostic workbench execution drains input but retains no persistent
  input state after configured bootstrap.
- Delete the transaction journal, counters, capacity/fault state, status JSON, canonical hashing,
  record/replay lifecycle, and all related tests/dependencies that are not otherwise required.
- Delete `PostedMessage`, native workbench post injection, all `input.*` inspect controls, all
  `runseal :workbench input*` commands, host-input replay support, and the long acceptance report
  field. Old controls become unsupported immediately; add no alias or empty compatibility response.
- Preserve Experiment 0041 as settled history. Mark ADR 0044 superseded rather than rewriting its
  accepted evidence. A maintained guard forbids every retired live file, symbol, verb, and command.

## Consequences

- Live host input state matches actual product needs and has fixed non-dropping storage with no
  journal allocation or diagnostic replay branch.
- The workbench can no longer synthesize native input or inspect/record/replay keyboard state.
  Product input proof remains real window-procedure input through the focused prototype workflow.
- Complete ordered transition streams, counters, hashes, and process-local replay remain historical
  Experiment 0041 evidence, not runtime capability.
- This decision adds no replacement replay, input action system, event queue, input persistence,
  frame/simulation sampling, camera/gameplay behavior, or engine input ownership.

## Evidence

Experiment 0085 removed 814 tracked implementation/test lines and added 66, plus one 93-line
maintained removal guard. `HostInput` is now exactly three 256-bit sets (96 bytes, no drop) and all
20 reference-host tests pass. A real workbench rejected all five retired inspect verbs and four
retired wrapper commands in 11.984 seconds. A deliberate verb reintroduction made the removal guard
fail in 0.7 seconds before build work.

`canonical-prototype-v10` passed in 75.045 seconds. Native W posted before readiness still produced
exact Walk / `deltaZQ9=-32`; native Escape exited 0 with empty stderr, and held-W finite-edge
survival exceeded 15 seconds. `runseal :init` passed. No runtime, renderer/GPU, resource,
synchronization, source-format, lifecycle implementation, asset, or Wulin path changed.
The merge-checkpoint guard passed with zero Flavor denies and five pre-existing warnings.
