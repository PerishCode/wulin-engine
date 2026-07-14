# ADR 0042: Canonical Runtime Host Separation

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

The accepted renderer, streaming, scene/world, presentation-time, shader, and GPU lifecycle owners
were implemented under `apps/workbench`. This was appropriate while they were experiments, but it
made the diagnostic host the physical owner of the only live engine runtime. Adding time, input,
simulation, or another host on top of that layout would either deepen workbench coupling or create
a second renderer path.

Experiments 0031 through 0038 established enough behavior, failure, resource, and lifecycle
evidence to promote the canonical runtime without redesigning it.

## Decision

- `crates/engine-runtime` owns the accepted scene/world state, signed source streaming, atomic
  composition, traversal/prefetch/rollover, renderer, presentation time, shaders, D3D12 Agility
  binding, GPU resources, probes, and native device lifecycle.
- One `Runtime` facade owns the sole `Renderer` and `SceneState`. It accepts the native HWND and
  fixed extent, advances frames, and exposes only operations already required by the accepted
  workbench control and evidence surfaces.
- `apps/workbench` owns the Win32 window/message pump, inspect transport and protocol, operator
  capture persistence, perception request/response shaping, readiness reporting, and process
  lifecycle. It depends on the runtime crate and does not assemble renderer subsystems directly.
- The runtime remains specific to the accepted Windows/D3D12 reference platform. This decision
  does not define a cross-platform window, graphics, input, or application abstraction.
- The repository guard rejects legacy runtime source under the workbench, reverse imports from the
  runtime to host modules or top-level consumers, a runtime dependency on workbench, and multiple
  renderer definitions.

## Consequences

- A later temporal-execution experiment can evolve engine-owned update/render stepping without
  first adding more state to the workbench host.
- Shader compilation and Agility staging occur with their runtime owner, so a future accepted host
  can consume the same binary GPU path instead of copying workbench build logic.
- The facade intentionally mirrors a broad accepted diagnostic/control surface. It is not yet a
  generalized game API; later experiments must narrow or layer typed time/input/simulation
  contracts based on evidence.
- No native input, automatic source bootstrap, player state, actor plane, new application, or
  rendering behavior is introduced by this decision.

## Evidence

Experiment 0039 passed the direct 579.4-second workflow with exact pre-migration color, PNG,
object-ID, diagnostic, light-matrix, and shadow-depth hashes. All 32 reactive and 32 prepared
crossings and 16 lifecycle cycles passed. The resource run had a 531-handle settled baseline and
peak, zero transient growth, and ended below the settled private-byte and thread baselines.

Generated evidence is ignored under
`out/captures/0039-canonical-runtime-host-separation/`.
