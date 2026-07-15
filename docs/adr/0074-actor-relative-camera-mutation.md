# ADR 0074: Actor-Relative Camera Mutation

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0071 Actor-Relative Camera Anchor

## Context

The prototype owns one grounded runtime actor and applies fixed gravity before each live frame, but
its camera still remains at the scene default. Horizontal movement under that fixed view would
eventually leave the actor behind. It could also commit an actor outside the current published
render window before the following frame preflight rejects it.

Experiment 0070 deliberately removed the standalone actor projection API and diagnostic event.
Camera following must therefore reuse the sole renderer-internal projection without restoring a
second read surface. It is also narrower than the later problem of coordinating movement with
camera-driven traversal publication.

## Decision

- `Runtime` exposes one actor-relative camera mutation taking a generation-qualified actor handle,
  position offset, target offset, and vertical field of view. It returns only success or failure.
- The runtime reads the current actor, asks the renderer-private projection for its current
  published-window representation, and restores the bounded active-window alias in checked Q9
  arithmetic before converting the origin-relative XZ center to scene meters. Height uses the
  renderer-consistent Q16-to-`f32` conversion.
- `SceneState` constructs and validates one complete candidate camera before replacing its current
  camera. Stale/missing actors, unavailable projection, non-finite results, degenerate view
  vectors, and invalid fields of view preserve the prior camera.
- The prototype owns one fixed rig: position offset `[9, 4, 12]`, target offset `[0, -1, -3]`, and
  vertical field of view `60`. It applies that rig after every simulation opportunity and before
  every live frame.
- The existing camera-state JSON is evidence only; no actor projection fields, inspect event,
  compatibility alias, or alternate projection authority are introduced.

## Consequences

- Prototype frames now keep their camera deterministically anchored to the current capacity-one
  actor while preserving exact generation and current published-window admission.
- The fixed rig is prototype policy, not an engine default, mod setting, smoothing controller, or
  camera action surface.
- Horizontal input, locomotion, traversal enablement, prefetch/rollover exercise, and a combined
  simulation/traversal/frame transaction remain unaccepted. A later experiment must prove
  backpressure before movement can outrun the current published window.
- No shader, pass, GPU resource, upload, copy, barrier, fence, wait, source format, or renderer
  lifecycle changes.

## Evidence

Experiment 0071 records exact alias/recenter/seam unit evidence, transactional camera rejection,
the real prototype rig and frame ordering across direct restarts, bootstrap failure behavior, and
complete Sidecar cleanup. `runseal :canonical-prototype` passed in 35.608 seconds and the repository
guard passed with zero Flavor denies.
