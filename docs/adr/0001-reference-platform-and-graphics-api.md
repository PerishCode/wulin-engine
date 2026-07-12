# ADR 0001: Reference Platform and Graphics API

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

The project is an architecture experiment for a modern GPU-oriented engine. Broad
compatibility is not a success criterion. The reference machine runs Windows 11 with an
NVIDIA GeForce RTX 4070 Ti SUPER, 16 GB of VRAM, and a current native graphics stack.

The project needs explicit control and observation of resource lifetime, barriers,
queues, indirect execution, timestamps, and CPU/GPU synchronization. A portable graphics
abstraction would hide part of the execution model being studied.

## Decision

- Use Windows 11 x64 as the only reference operating system during architecture
  validation.
- Use the installed NVIDIA GPU as the only required hardware target.
- Use Rust for host implementation.
- Use native Direct3D 12 directly through Rust bindings; do not introduce a general RHI.
- Use HLSL compiled to DXIL with DXC.
- Pin a retail DirectX 12 Agility SDK release during the R1 cold start.
- Do not implement Vulkan, `wgpu`, legacy D3D, alternate-vendor, integrated-GPU, or
  reduced-feature fallback paths during the experimental stages.

## Consequences

The project can design around one explicit GPU execution model and use current D3D12
capabilities without lowest-common-denominator constraints. Resource transitions and
synchronization remain visible to experiments.

The resulting engine is intentionally non-portable during validation. Portability may
only enter scope through a later accepted ADR backed by a concrete requirement. Internal
ownership boundaries should remain clean, but must not be generalized for hypothetical
backends.

## Evidence

- Local environment inspection on 2026-07-12 confirmed Windows 11, RTX 4070 Ti SUPER,
  Visual Studio 2022, Windows SDK, Rust MSVC, and a current NVIDIA driver.
- R1 will establish the first executable evidence through the numbered GPU laboratory
  experiment.
