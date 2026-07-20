# Experiment 0165: In-Process Native Input Transport

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-20
- Related ADRs: ADR 0168

## Hypothesis

The canonical Prototype acceptance owner can execute its existing exact-PID
Win32 window actions through Deno FFI instead of starting and compiling one
PowerShell/C# helper per action. This should remove at least 10% from the
160.144-second full workflow while preserving all 18 product processes, 35
native action batches, schema-4 evidence, exact message ordering, delayed-action
lower bounds, atomic window-thread prefixes, focus transitions, and graceful
completion behavior.

## Scope

- Replace only `.runseal/support/prototype/input/` action transport.
- Load the fixed reference-platform `user32.dll` and `kernel32.dll` APIs in the
  acceptance process.
- Preserve exact process-qualified window discovery, visibility checks, bounded
  waiting, monotonic key/exit deadlines, and thread suspend/post/resume
  atomicity.
- Preserve the existing public support functions and
  `prototype-native-window-action-v4` report shape.
- Remove the action path's `pwsh` process, dynamic `Add-Type` compilation,
  helper-ready stdout protocol, child stdout/stderr drain, and JSON reparse.
- Replace the historical transport guard with direct in-process FFI ownership
  and a stable prohibition on action-helper process restoration.

The separate Activated-frame composition observer, interactive-desktop
preflight, other repository PowerShell consumers, product Rust, Runtime,
renderer/GPU, source/resource/synchronization ownership, product process count,
and behavior matrix are out of scope.

## Baseline

- `canonical-prototype-v78`: 160.144 seconds, 372,736 bytes, 18 unique product
  PIDs, 17 Escape plus one window-close completion, and zero copied evidence
  subtrees.
- The 18 timed action sessions consume 125.215 seconds. Their product canonical
  readiness work totals 2.581 seconds; fixed action/exit delays total 6.900
  seconds, and the finite-boundary hold remains 15 seconds.
- Ten isolated launches of the exact current PowerShell `Add-Type` definition
  took 6,267.941 ms total, with 597.908 ms P50 and 915.585 ms P95. At 35 actions
  this exposes approximately 21 seconds of repeated transport initialization.
- The focused Activated-frame lane's final pre-change runs were
  13.956/13.716/13.789 seconds with one exact product PID and unchanged shared
  behavior oracles.

## Workload

1. Replace the PowerShell/C# action helper with an in-process Deno FFI owner
   over the same Win32 functions and constants.
2. Add focused tests for argument validation, monotonic deadlines, evidence
   shape, and deterministic pure helpers.
3. Update the structural guard to require FFI load/call/close ownership and
   forbid `Deno.Command("pwsh")`, `Add-Type`, helper readiness markers, and
   child transport in the action path.
4. Run the one-process focused Activated-frame lane for rapid native-action and
   exact-frame validation.
5. Run the unchanged no-argument 18-process full matrix and compare
   process/action, completion, copy, report-size, and elapsed evidence to v78.
6. Run repository guard, init, formatting, diff, and process cleanup.

## Controlled variables

- The Prototype executable, bootstrap content, source packs, product input
  adapter, host window/message loop, and all behavior oracles remain fixed.
- The matrix remains 18 product PIDs and 35 native action batches.
- Window class/title, exact PID match, visibility requirement, 20-second search
  bound, message constants, lParam/wParam values, and message order remain
  fixed.
- Atomic prefixes still suspend the exact window thread, post the complete
  prefix, optionally post focus loss, and resume/close the thread handle in all
  outcomes.
- Key and exit delays remain monotonic lower bounds with the existing 0..1,000
  ms input validation.
- The sustained rejection phase may replace transport-startup time that was
  accidentally required for its 12-frame product contract with one explicit
  measured hold; no other wait is increased.
- Schema-4 evidence fields and downstream invariant ownership remain fixed.
- No retry, product delay, compatibility fallback, second wrapper, new product
  telemetry, or reduced gate is introduced.

## Metrics

- Full and focused workflow elapsed time and report bytes.
- Product PID count, native action count, completion-reason distribution,
  product readiness time, explicit delay totals, and boundary hold duration.
- Exact raw/invariant single ownership and copied-subtree count.
- Native action message order, key intervals, exit intervals, atomic
  prefix/thread IDs, and batch spans.
- External action-helper process and dynamic compilation count.
- Test, Flavor, init, diff, cleanup, and lingering-process results.

## Acceptance criteria

- The action transport starts zero external processes and performs zero dynamic
  `Add-Type` compilations.
- The no-argument full workflow retains exactly 18 unique product PIDs, all 35
  native actions, 17 Escape plus one window-close completion, zero copied
  subtrees, and every existing product/session/object/boundary invariant.
- Every action remains exact-PID/window-qualified; required visible windows are
  observed; messages, delays, atomic prefix/thread evidence, and batch spans
  pass the existing schema-4 validators without threshold relaxation.
- Full elapsed time improves by at least 10% from the 160.144-second v78
  baseline, excluding no process, hold, frame observer, or behavior gate.
- The focused Activated lane retains exact Activated/suppression/commit/block
  and visible-frame completion evidence.
- The separate frame observer remains bounded and PowerShell-backed for this
  experiment; no unrelated PowerShell permission or consumer is changed.
- Any behavior phase exposed as dependent on removed helper-startup time must
  gain one named, reported lower bound and retain its exact final frame
  contract.
- Repository guard, tests, Flavor, init, formatting, diff, and process cleanup
  pass.

## Environment

Use the repository-pinned Rust/Deno toolchains and reference Windows 11
interactive desktop. Deno FFI is limited to the current reference-platform
native input support. Record exact tool versions and full/focused report
evidence.

## Reproduction

```powershell
runseal :canonical-prototype --case=activated-frame
runseal :canonical-prototype
runseal :guard
runseal :init
```

## Results

### Transport and behavior

- The native action owner now loads `user32.dll` and `kernel32.dll` once in the
  acceptance process, executes every existing request directly, and explicitly
  closes both handles in the wrapper `finally` path.
- The action path contains no `Deno.Command`, process spawn, PowerShell,
  `Add-Type`, helper readiness marker, child stdout/stderr protocol, or JSON
  serialization round-trip. The separate full-content frame observer remains
  unchanged.
- Four focused validation tests pass for UTF-16 window-name encoding and pre-FFI
  PID, action-shape, and atomic-delay rejection.
- The full report retains exactly 35 schema-4 actions across 18 unique product
  PIDs: 14 atomic batches, 17 Escape completions, one window-close completion,
  and zero copied evidence subtrees.
- Exact-PID/window visibility, message arrays, key/exit lower bounds, atomic
  prefix lengths, window thread IDs, batch spans, focus suspend/resume, and
  close behavior all pass the existing invariant owners.

### Exposed timing dependency

The first full run exposed one legitimate behavior dependency that the old
transport had hidden. The sustained capacity session reached exact consumption,
exclusion, and target state, but its explicit 250-ms post-rejection hold
produced only 8 of 12 required Rejected frames; four acknowledgement frames
remained. The former per-action PowerShell startup had accidentally supplied the
missing time.

The action owner now reports one named 500-ms sustained rejection hold. The
accepted run measured 514.427 ms and restored exact 12 Activated, 12 Rejected,
one commit, one ineligible attempt, 21 suppression frames, no pending
acknowledgement, and zero render blocks. Sustained validation also moved before
the other 14 session launches, so future divergence fails earlier without
changing successful-run process coverage or evidence.

### Timing and acceptance

- Warm focused Activated-frame acceptance passed in 10.317 seconds versus the
  final 13.789-second pre-change baseline: 3.472 seconds / 25.18% faster. It
  retained one PID, 12 Activated, 13 suppression, one commit, zero blocks, exact
  visible-frame clear, and zero copies.
- The final post-refactor focused smoke passed in 11.268 seconds with the same
  exact evidence.
- `canonical-prototype-v79` passed in 131.814 seconds versus 160.144 seconds:
  28.330 seconds / 17.69% faster.
- The 18 action-session durations fell from 125.215 to 96.135 seconds, a
  29.080-second / 23.22% reduction. Product canonical readiness remained
  effectively fixed at 2.563 seconds versus 2.581 seconds.
- The finite-boundary process retained a 15,013-ms hold, 508 live frames, exact
  tangential Run completion, and graceful Escape.
- The v79 report is 374,336 bytes. Its 1,600-byte increase comes with explicit
  sustained rejection timing; no raw launch/invariant subtree is duplicated.
- The focused/full workflows, repository guard, 15 acceptance-support tests,
  Flavor, Sidecar plans, init, formatting, diff, and process cleanup pass.
  Flavor retains zero denies and the same five existing warnings.
- Reference evidence used Rust/Cargo 1.94.1 and Deno 2.9.1 on the existing
  Windows reference platform.

## Conclusion

Accepted. Direct in-process Win32 FFI is the sole Prototype native-action
transport. The product process matrix and action semantics remain exact while
per-action helper startup is retired.

## Promotion

Promoted to the current runtime boundary and ADR 0168.
