# ADR 0067: Retained Runtime Actor Authority

- Status: Accepted
- Date: 2026-07-15
- Supersedes: ADR 0056 retained-body ownership clause
- Superseded by: None

## Context

Runtime owns one retained terrain body and prototype is its sole live driver. The next dependency
is actor identity, lifetime, transform, and presentation authority. Adding an actor that merely
references the existing body would create two independent generations, partial spawn/despawn
ordering, and an unsupported detached-body state. Defining a second presentation model would also
diverge from the accepted schema-3 archetype/material/yaw/animation authority before a workload
requires different fields.

## Decision

- Replace the retained body slot directly with one capacity-one runtime actor. The actor owns one
  checked nonzero generation handle, exact terrain-body motion, and exact schema-3
  `PresentationRecord`.
- Move presentation field validation onto `PresentationRecord` and use it for both signed pack
  validation and actor spawn.
- Validate presentation before advancing generation or occupying the actor slot. Preserve exact
  stale-handle, rollback, despawn, and respawn behavior.
- Make the sole simulation transaction actor-addressed. It replaces only actor motion after all
  schedule/motion work succeeds and retains actor identity and presentation byte-exactly.
- Have prototype create one imported Fox actor after canonical publication and drive it through the
  accepted Ready-only zero command before frames.
- Delete the retained-body public API and readiness vocabulary. Add no compatibility alias or
  detached body handle.
- Keep runtime actors CPU-only in this decision. GPU presentation binding, mutable animation state,
  collections, gravity, locomotion, camera, and gameplay input remain separate experiments.

## Consequences

- Runtime gains the smallest concrete actor authority without an ECS or duplicate body lifetime.
- Canonical cooked objects and live actors share presentation semantics but remain distinct
  ownership planes until a renderer-binding experiment proves their join.
- Prototype still looks unchanged; this stage makes the later visible actor path depend on one
  proven identity/transform/presentation lifetime.

## Evidence

Experiment 0064 passes 63 engine-runtime, 7 region-format, and 3 prototype tests plus focused clippy
and TypeScript checks. A 27.7-second real workbench lifecycle gate proved presentation-before-
generation validation, exact actor lifecycle/replay, and process reset. A 41.8-second simulation-
actor gate proved fractional/coarse/nominal schedule composition, presentation preservation, full
failure rollback, and retired body-verb rejection. A 32.21-second prototype gate proved exact
generation-one imported actor readiness across two direct processes plus failure and Sidecar
lifecycle cleanup.

`runseal :init` and `runseal :guard` passed. The long canonical workflow was not run because the
actor remains CPU-only and no renderer/GPU/resource/synchronization/canonical lifecycle evidence
can be invalidated by this change.
