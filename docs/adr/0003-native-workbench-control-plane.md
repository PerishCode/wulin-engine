# ADR 0003: Native Workbench Control Plane

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

The first operator-visible engine surface needs a smaller acceptance boundary than
region rendering. Before scene semantics can be discussed through frames, the project
must own one real window whose process lifetime, rendering state, and visible output are
controllable and inspectable through the established local tooling.

## Decision

- `apps/workbench` is the first production composition root. It owns a fixed-size native
  Win32 window, a D3D12 swap chain, and a continuous clear/present loop.
- The main thread exclusively owns HWND and D3D12 objects. The TCP inspect listener runs
  on a background thread and submits typed commands to the main thread through a bounded
  channel.
- Sidecar owns the workbench process lifecycle through the `[app]` target in
  `sidecar.toml`. Readiness is emitted only after the window, renderer, first frame, and
  inspect listener are operational.
- The project-owned inspect protocol initially exposes `workbench.status`,
  `workbench.set_clear_color`, `workbench.pause`, and `workbench.resume`.
- Windows inspect uses a loopback TCP endpoint. It is a local development control plane,
  not a security boundary or a general remote-call interface.
- `runseal :workbench` is the canonical operator entrypoint for lifecycle and typed
  inspect operations.

## Consequences

The repository has a deterministic visible work surface and can close a control-to-frame
to-inspection loop before introducing scene, asset, ECS, UI, or render-graph concepts.
The executable contains workbench-specific composition code only; reusable engine logic
must move into `crates/` when a later accepted capability creates a real reusable owner.

The fixed TCP endpoint permits one default `dev` namespace instance. Concurrent namespace
instances and dynamic inspect endpoints remain out of scope until an actual workflow
requires them.

## Evidence

- The workbench presented a 1280x720 D3D12 swap chain on the reference NVIDIA adapter.
- Sidecar inspect changed the visible clear color and observed matching state, frame
  progress, pause, and resume behavior.
- Sidecar start, status, restart, and stop completed against the stamped Cargo/workbench
  process tree; final status reported no target or broker processes.
- The Windows lifecycle gap discovered during acceptance is addressed by Sidecar 0.5.1.
