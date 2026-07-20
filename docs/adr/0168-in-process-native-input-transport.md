# ADR 0168: In-Process Native Input Transport

- Status: Accepted
- Date: 2026-07-20
- Experiment: 0165 In-Process Native Input Transport

## Context

The canonical Prototype workflow retains 18 real product processes and 35 exact-PID native window
actions. Each action currently starts a fresh PowerShell process and dynamically compiles the same
C# Win32 declarations before finding the target window.

The accepted v78 workflow takes 160.144 seconds. Ten isolated launches of the exact current native
definition measured 597.908 ms P50 and 6,267.941 ms in aggregate, exposing approximately 21 seconds
of repeated initialization across 35 actions. This cost is acceptance transport rather than product
startup, behavior observation, or process isolation: canonical product readiness totals only 2.581
seconds across the 18 action sessions.

The current reference platform is Windows, the acceptance wrapper already owns the native action
policy, and Deno exposes direct FFI for the same fixed `user32.dll` and `kernel32.dll` calls. The
separate full-content frame observer has different capture and lifetime responsibilities and is not
part of this decision.

## Decision

- Execute Prototype native window actions in the acceptance Deno process through direct Win32 FFI.
- Preserve exact PID/window/visibility qualification, bounded search, monotonic key/exit deadlines,
  message constants/order, and schema-4 evidence.
- Preserve atomic prefix behavior by opening and suspending the exact window thread, posting the
  complete prefix and optional focus loss, and always resuming and closing the thread handle.
- Remove the per-action PowerShell child, C# `Add-Type` compilation, helper-ready stdout handshake,
  JSON serialization/reparse, and child stdout/stderr protocol.
- Keep the separate Activated-frame composition observer and other repository PowerShell consumers
  unchanged.
- Require a structural guard that forbids restoration of an external action helper or dynamic native
  compilation.

## Consequences

- The full workflow should retain all 18 product processes and 35 native action batches while
  eliminating 35 acceptance helper processes.
- The action support becomes explicitly tied to the existing reference Windows platform and Deno's
  FFI surface. This does not broaden product or engine platform support.
- Native library handles live only in the acceptance process and must be closed at wrapper
  termination; no persistent service, input queue, replay journal, or product state is introduced.
- Failure diagnostics move from child exit/stdout/stderr evidence to direct typed acceptance errors
  while existing action evidence remains unchanged.

## Evidence

Experiment 0165 retained 35 schema-4 native actions across 18 unique product PIDs, 14 atomic
batches, 17 Escape plus one window-close completion, exact behavior/frame oracles, and zero copied
evidence subtrees. The action path starts no external process and contains no PowerShell, dynamic
compilation, helper handshake, or child transport.

The full workflow fell from 160.144 to 131.814 seconds, a 28.330-second / 17.69% reduction. The 18
action-session durations fell by 29.080 seconds while product canonical readiness remained fixed.
Warm focused Activated-frame acceptance improved from 13.789 to 10.317 seconds.

Removing the implicit helper delay exposed an incomplete sustained rejection at 8 of 12 frames.
Replacing it with one explicit reported 500-ms action-owned hold restored the exact 12 Activated /
12 Rejected contract. Focused/full workflows, structural guard, 15 acceptance-support tests, Flavor,
Sidecar plans, init, and cleanup passed.
