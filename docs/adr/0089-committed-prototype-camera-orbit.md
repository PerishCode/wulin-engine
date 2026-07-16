# ADR 0089: Committed Prototype Camera Orbit

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0086 Committed Prototype Camera Orbit

## Context

The prototype applies one fixed actor-relative camera rig before every frame. The runtime already
owns checked actor projection and atomic complete-camera replacement, while the reference host now
provides bounded sample-scoped key edges. A camera action must not mutate application orbit state
before the runtime accepts its candidate or introduce another camera/projection authority.

Jump and object interaction are not selected here. Jump still requires separate grounded, fixed-step
intent, impulse, and backpressure decisions; object interaction lacks a CPU gameplay object authority.
A discrete keyboard orbit is the smallest product action that can prove the missing application
candidate/commit boundary using only accepted dependencies.

## Decision

- The prototype owns four exact quarter-turn actor-relative rigs and one committed orbit index.
- Q requests one counter-clockwise candidate, E one clockwise candidate, and same-sample opposite
  edges cancel. Held state does not drive camera steps.
- Candidate preparation is pure. The prototype calls the existing generation-qualified
  `Runtime::set_actor_relative_camera` with the complete candidate and commits the orbit index only
  after success, before rendering the frame.
- The default state preserves the existing rig. Quarter turns rotate only XZ; Y offsets and vertical
  field of view remain fixed.
- Focused real-process evidence posts E only after the exact prototype window is visible and checks
  the committed rig/camera through the one-time readiness record. The rotated camera must drive the
  corresponding exact traversal desire through the existing bounded latest-wins state machine; an
  in-flight initial target may create depth-one queued replacement but no new traversal policy.
- The focused fixture cooks the rotated `[+1,-1]` center in addition to its existing base,
  `[+1,+1]` traversal, and corrupt centers. Product source policy and the manual 441-region sandbox
  are unchanged; unavailable source remains fatal evidence rather than a fallback.
- Add no engine input/camera controller, pointer transport, free-look, smoothing, collision,
  configuration, inspect route, projection readback, or alternate camera mutation.

## Consequences

- Prototype gains one bounded camera action without frame-rate repetition or duplicated native key
  normalization.
- Application camera policy cannot advance ahead of the runtime scene camera.
- Camera-derived traversal observes the committed view. A camera action may replace one in-flight
  initial target through the already accepted depth-one latest-wins queue.
- Camera behavior remains prototype policy; runtime owns only validation, projection, and scene
  mutation.
- Jump, gameplay object authority, arbitrary camera controls, sustained source service, and Wulin
  policy remain later experiments.

## Evidence

Experiment 0086 passes 18 prototype policy tests and `canonical-prototype-v11` in 77.671 seconds
with 77 engine-runtime and 20 reference-host tests. The process-qualified visible-window E witness
committed orbit 1 with exact rig `[12,4,-9] / [-3,-1,0]`, exact anchored camera, stationary Survey,
zero render blocks, and exact `[+1,-1]` traversal desire. Default/restart/W/Escape/finite-boundary,
failure, and Sidecar cleanup evidence remained exact.

Two rejected gate assumptions improved the accepted boundary: traversal is expected to follow the
new camera rather than remain unchanged, and the focused fixture must contain the rotated center
rather than tolerate missing source. Repeated real-process samples proved direct schedule and the
existing depth-one latest-wins replacement as the two valid readiness states. No runtime, renderer,
GPU resource, synchronization, format, asset, or source-policy implementation changed.
`runseal :init` and the final repository guard passed with zero Flavor denies. Camera readiness
evidence owns its own support module rather than leaving host process orchestration above the
repository's source-length boundary.
