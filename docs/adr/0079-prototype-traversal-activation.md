# ADR 0079: Prototype Traversal Activation

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0076 Prototype Traversal Activation

## Context

The canonical runtime already owns and validates camera-driven composition traversal, but the plain
prototype left it disabled. Its fixed actor-relative camera starts at X/Z `9/12`; under the accepted
16-meter signed-region basis this is already one region diagonally beyond the bootstrap center.
Leaving traversal disabled therefore prevented the application from exercising a required runtime
policy even before sustained actor movement existed.

The prototype needs evidence that it requests the correct target. It does not need a second inspect
surface or a duplicate proof of engine-owned completion, replacement, rollback, rollover, and
resource behavior.

## Decision

- The prototype enables composition traversal exactly once after canonical bootstrap and actor
  spawn, before the window becomes visible. It does not enable traversal prefetch.
- The first camera-driven schedule must be token 2 at local center `(65,65)`, global center
  `base + (1,1)`, with one session, desired change, attempt, and schedule.
- Readiness carries the existing traversal member from `Runtime::composition_status` after the
  successful readiness-producing frame. It adds no prototype control or inspect endpoint.
- Async progress at that snapshot may contain zero publications and no published target, or one
  publication of the same token and target. Maintained acceptance normalizes that timing and still
  rejects queue, block, failure, prefetch, rollover, or any different schedule.
- The focused cook includes the overlapping traversal center so the scheduled target has valid
  signed terrain and object input. Engine workflows remain authoritative for eventual completion
  and all traversal algorithms.

## Consequences

- The plain prototype now exercises one real camera-derived traversal schedule as part of normal
  startup instead of stopping at the bootstrap publication.
- Readiness gains bounded observation of an existing runtime state but no mutation route, fallback,
  compatibility alias, or timing-dependent completion requirement.
- Sustained movement across further region boundaries and application policy under a long-running
  pending publication remain separate experiments.
- Runtime API, traversal implementation, renderer/GPU resources, synchronization, source formats,
  camera rig, movement tuning, and Wulin content are unchanged.

## Evidence

Experiment 0076 records the final `canonical-prototype-v5` run. It passed in 36.119 seconds with
three independent processes scheduling token 2 at exact local `(65,65)` and global
`(1099511627777,-1099511627775)`, publication count zero at readiness, no queue/block/failure,
no prefetch or rollover, exact restart/native-W normalized evidence, and complete Sidecar cleanup.
