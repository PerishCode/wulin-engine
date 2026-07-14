# ADR 0046: Reference Platform Host

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

The engine runtime is already independent from its diagnostic workbench, but accepted Win32
window/input and canonical bootstrap code still live inside that first host. Copying those modules
into a prototype would create two subtly different readiness and native-input paths; defining a
general application/platform framework would exceed the evidence.

Experiments 0041 and 0042 now provide exact contracts, and a second real executable would make
their reuse concrete.

## Decision

- One reference-platform host crate owns the single Win32 window/message lifecycle, normalized
  keyboard/focus state and bounded journal, strict schema-1 bootstrap parser/path validation, and
  hidden canonical bootstrap driver.
- `apps/workbench` consumes that crate but remains the sole owner of inspect transport/protocol,
  operator capture/perception persistence, pause/error response shaping, fault injection, and
  acceptance evidence.
- `apps/prototype` is a non-diagnostic composition root over the same reference host and
  `engine-runtime`. Configured canonical startup is mandatory; its only initial input action is
  Escape requesting host exit.
- The reference host is explicitly Windows-specific and single-window. It introduces no platform
  traits, alternate renderer, backend selection, generalized callbacks, resize contract, or
  compatibility surface.
- `engine-runtime` remains below both hosts and has no argument, path, readiness, message-pump, or
  native-input policy.

## Consequences

- The repository gains a plain runnable engine-host skeleton without turning diagnostic controls
  into a product API or selecting simulation/camera semantics early.
- Subsequent spatial-query and fixed-step experiments can add typed state above the same runtime
  without first untangling workbench-native code.
- The host crate will carry some evidence-facing journal/status behavior used only by workbench;
  splitting that policy requires a third consumer or independent pressure, not speculative
  layering in this experiment.

## Evidence

Experiment 0043 passed the 587.5-second direct workflow. Invalid, missing-source, and asynchronous
corrupt-payload prototype launches exited without readiness. Fresh valid processes became
canonical-ready on hidden frames 8 and 9 with the same exact config hash and signed target; the
no-inspect Sidecar lifecycle replaced and removed every owned PID. Ten focused reference-host
tests, exact workbench input/bootstrap behavior, all prior GPU hashes, 32+32 traversal, a
zero-growth 527-handle resource plateau, 16 lifecycle cycles, the repository guard, and final
four-namespace cleanup passed.

Generated evidence is ignored under
`out/captures/0043-thin-prototype-host/`.
