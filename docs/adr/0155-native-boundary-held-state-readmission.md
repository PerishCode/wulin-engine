# ADR 0155: Native Boundary Held-State Readmission

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0152 Native Boundary Held-State Readmission

## Context

The accepted finite-boundary workflow used one helper to post Shift/W and a later helper to post
A. A v67 rerun finished with X on a cardinal rather than 45-Q9 diagonal-component lattice,
showing that the later phase implicitly depended on W surviving across native helper lifetimes.
The endpoint gate correctly rejected the run, but the action did not fully declare its own input.

## Decision

- Retain the existing child, initial Shift/W prefix, 15-second boundary hold, tangential/stationary
  delays, releases, endpoint gate, and graceful completion.
- At tangential-phase entry, atomically reassert Shift-down/W-down/A-down on the exact visible
  PID/window thread before waiting 500 ms.
- Require prefix length three, positive exact thread identity, a 0..=50 ms span, the existing
  release order, and the same 250..=750 ms delayed Escape.
- Keep repeated-down suppression and fresh-down readmission owned by the current normalized host
  input state.
- Add no product state, query, polling, retry, process, output, alias, compatibility decoder, or
  weakened endpoint/clock gate.

## Consequences

- The second helper explicitly owns every held key required by its phase.
- A retained Shift/W state observes harmless duplicate downs; a cleared state is re-admitted before
  A begins, so both cases converge to the required diagonal Run command.
- The real-process boundary conclusion remains the same, but its native stimulus is no longer
  sensitive to an unstated cross-helper lifetime.
- Product behavior and acceptance cost remain structurally fixed.

## Evidence

Clean-commit `canonical-prototype-v67` passed in 168.853 seconds with a 458,350-byte report.
PID 21676 used window `37227806` and thread 32552. The initial Shift/W batch spanned 0.0011 ms;
the same thread's Shift/W/A reassertion spanned 0.0022 ms before a 514.6863 ms A hold.

The actor finished at local `(-1395,-3738)` Q9 with 31 exact 45-Q9 X steps and at least 22
tangential-only commits, retained Survey/yaw 32,768, and epoch 1,063. Ready/sample reached
`1077/1078` across 1,078 live frames with zero stalls, render blocks, or object effects. Exit was
zero with exactly two values and empty stderr/trailing output.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor remained at zero
denies and five existing warnings. The detached validation commit contained only the four
acceptance changes; product, Runtime, renderer/GPU, source, resource, synchronization, schema, and
process count remained unchanged.
