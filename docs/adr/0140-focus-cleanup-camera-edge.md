# ADR 0140: Focus-Cleanup Camera Edge

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0137 Focus-Cleanup Camera Edge

## Context

HostInput preserves pressed and released facts for every state-changing transition in one ingest,
including a key down followed by FocusLost. Prototype camera candidates consume sample-scoped Q/E
presses and commit before each frame, while Jump and object intents defer completion to nonzero
simulation and explicitly cancel on Suspended/Reset. Applying the latter cancellation semantics to
camera without evidence would change an established discrete, frame-driven action.

## Decision

- Retain the current HostInput rule: E-down followed by FocusLost in one ingest yields held=false,
  pressed=true, and released=true for that sample.
- Retain the current camera rule: the preserved press prepares one clockwise candidate and may
  commit orbit 0 to orbit 1 through the existing runtime-before-policy commit boundary.
- Require an empty next ingest to expire both edges and prevent another camera step.
- Keep simulation-bound action cancellation separate; add no sample input, pending camera intent,
  controller, queue, report, or native process.

## Consequences

- Message order remains meaningful: a discrete camera press preceding focus loss is not silently
  reclassified as a pending simulation action.
- Focus cleanup still prevents held repetition, and edge expiry bounds the camera action to one
  sample.
- Product HostInput, camera policy, main-loop ordering, Runtime, renderer/GPU resources, sources,
  synchronization, process count, and report schema remain unchanged.

## Evidence

All four focused camera tests passed. The new composition test proved exact held/pressed/released
facts, orbit `0 -> 1` commit, empty-ingest edge expiry, and no repeated step using the real
HostInput and camera policy types.

`canonical-prototype-v52` passed on its first run in 176.068 seconds with the unchanged native
session/process matrix and a 447,306-byte report. All 103 engine-runtime, 46 Prototype, and 20
reference-host tests passed; Flavor remained at zero denies and five existing warnings.
