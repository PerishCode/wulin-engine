# AGENTS.md

## 1. AGENTS.md Meta Rules

### 1.1 Scope and precedence

- This file applies to the repository root and every descendant directory.
- A nested `AGENTS.md` may add or narrow rules for its own subtree.
- Higher-priority system, developer, and current user instructions override this file.
- When instructions conflict, follow the highest-priority instruction and preserve the
  intent of the remaining rules where possible.

### 1.2 Repository operating rules

- Inspect the relevant code, documentation, and current working tree before changing
  files. Do not overwrite or revert unrelated work.
- Keep changes scoped to the active experiment or accepted project stage.
- Do not introduce speculative abstractions, compatibility layers, fallback paths, or
  portability work without an explicit requirement backed by an experiment.
- Prefer measurable evidence over architectural preference. A subsystem is not ready
  to become a dependency of the next stage until its acceptance criteria pass.
- Keep engine concerns and Wulin mod concerns separate. Game-specific workarounds must
  not leak into the engine core.
- Do not commit generated output, caches, captures, build artifacts, proprietary game
  assets, or credentials.
- Repository paths, code identifiers, and code comments use English. Project-facing
  documentation may use Chinese when that communicates the intent more precisely.
- Add comments only when they explain a non-obvious invariant, constraint, or tradeoff.

### 1.3 Maintaining this file

- Update this file in the same change whenever repository-wide directory ownership,
  core-file locations, or required operating workflows change.
- Keep this file concise and operational. Put design rationale, experiment reports,
  and detailed technical decisions under `docs/` once those locations exist.
- The core file index must list only files that currently exist. Add an entry when a
  core file is created and remove or update it when that file moves or is deleted.
- Do not turn temporary experiment commands into repository-wide rules until they are
  stable and repeatable.

## 2. Purpose

This repository exists to build and validate a modern, lightweight, GPU-oriented game
engine, followed by a large Wulin Zhuan mod that consumes the proven engine systems.

The project is an open-source architecture experiment, not a commercial product and
not a general-purpose engine. Its primary objective is to prove that modern workload
organization can make the rendering and simulation profile of this class of MMORPG
structurally inexpensive.

The project follows these principles:

- Prove capability before building content on top of it.
- Advance through explicit experimental gates rather than a feature checklist.
- Judge performance by scaling curves, frame-time stability, data movement, resource
  lifetime, and synchronization behavior instead of a target GPU model.
- Optimize work elimination, batching, GPU residency, and asynchronous execution
  before optimizing isolated instructions.
- Use one reference development platform while the architecture is being validated.
  Broad hardware, vendor, graphics-API, operating-system, and legacy compatibility are
  out of scope unless explicitly promoted into scope later.
- Keep gameplay authoritative on the CPU or server where appropriate while moving
  suitable rendering, animation, visibility, and simulation workloads to the GPU.
- Add Wulin mod content only after the underlying engine capabilities have passed their
  experiments. The mod must consume engine capabilities rather than compensate for
  missing ones.

## 3. Directory Conventions

Top-level directories are created only when they contain real work. Empty architecture
scaffolding is discouraged.

| Path | Ownership |
| --- | --- |
| `apps/` | Runnable clients, servers, editors, and other product entry points. |
| `crates/` | Reusable engine and shared runtime modules. Executables must not own reusable core logic. |
| `experiments/` | Isolated architectural proofs. Each experiment defines a hypothesis, workload, metrics, pass criteria, and result. |
| `benchmarks/` | Stable regression workloads promoted from successful experiments. |
| `mods/` | Mod-specific code, scripts, data, UI, and configuration. Wulin-specific behavior belongs under `mods/wulin/`. |
| `tools/` | Offline asset processing, import, inspection, profiling support, and developer utilities. |
| `assets/` | Redistributable source and test assets with clear provenance and licensing. |
| `tests/` | Repository-level integration and end-to-end tests. Unit tests stay beside their implementation. |
| `docs/` | Architecture, ADRs, experiment summaries, operational references, and contributor documentation. |
| `out/` | Disposable local experiment output, captures, reports, and generated artifacts; never a source-of-truth directory. |
| `.runseal/` | Repository hooks and Deno wrappers for explicit operator workflows. |

Additional conventions:

- Keep source assets, cooked runtime assets, and generated experiment output distinct.
- Legacy-format research and import code must remain isolated from canonical runtime
  formats and engine ownership.
- A successful experiment may be promoted into `crates/` and `benchmarks/`; failed or
  superseded experiments should retain only the evidence needed to explain the decision.
- Generated directories such as `target/`, `out/`, and tool-specific caches are not
  hand-edited.
- Avoid deep directory nesting until ownership boundaries justify it.

## 4. Core File Index

The repository has completed the R1 technical cold start. This index intentionally
contains only files that exist.

| File | Responsibility |
| --- | --- |
| `AGENTS.md` | Repository purpose, global agent rules, directory ownership, core-file index, and operating workflows. |
| `README.md` | Public project entry point, scope, status, and licensing summary. |
| `.gitignore` | Repository-wide generated-output and local-tool exclusions. |
| `.gitattributes` | Text normalization and binary-file classification. |
| `.editorconfig` | Baseline editor behavior for source and documentation. |
| `LICENSE-MIT` | MIT license terms offered by the project. |
| `LICENSE-APACHE` | Apache License 2.0 terms offered by the project. |
| `docs/architecture/repository-model.md` | Directory ownership, dependency direction, and experiment-promotion model. |
| `docs/adr/README.md` | ADR naming, status, and maintenance rules. |
| `docs/adr/0000-template.md` | Required structure for new architecture decision records. |
| `docs/adr/0001-reference-platform-and-graphics-api.md` | Accepted reference platform and graphics API decision. |
| `docs/adr/0002-personal-iteration-suite.md` | Accepted Flavor, Runseal, and Sidecar consumer boundary. |
| `docs/adr/0003-native-workbench-control-plane.md` | Accepted native window, Sidecar lifecycle, and inspect threading boundary. |
| `docs/experiments/README.md` | Experiment identity, evidence, output, and promotion rules. |
| `docs/experiments/0000-template.md` | Required structure for a new experiment definition and conclusion. |
| `Cargo.toml` | Rust Workspace definition and shared dependency policy. |
| `Cargo.lock` | Exact dependency resolution for reproducible experiment builds. |
| `rust-toolchain.toml` | Pinned Rust toolchain and required components. |
| `experiments/0001-gpu-lab/README.md` | Experiment 0001 hypothesis, protocol, status, results, and reproduction commands. |
| `experiments/0001-gpu-lab/Cargo.toml` | Isolated GPU laboratory package and Windows API feature set. |
| `experiments/0001-gpu-lab/build.rs` | DXC shader build and Agility SDK runtime staging. |
| `experiments/0001-gpu-lab/scripts/bootstrap.ps1` | Pinned, hash-verified Agility SDK acquisition. |
| `experiments/0001-gpu-lab/src/main.rs` | D3D12 compute, measurement, validation, and report implementation. |
| `experiments/0001-gpu-lab/src/agility_exports.c` | Process exports selecting the pinned Agility SDK. |
| `experiments/0001-gpu-lab/shaders/fill.hlsl` | Deterministic Experiment 0001 compute workload. |
| `apps/workbench/Cargo.toml` | Native workbench package and Windows API feature boundary. |
| `apps/workbench/build.rs` | Workbench Agility SDK export build and runtime staging. |
| `apps/workbench/src/main.rs` | Win32 window, main-thread control ownership, and operator-visible runtime state. |
| `apps/workbench/src/renderer.rs` | D3D12 swap chain, clear/present loop, and explicit GPU synchronization. |
| `apps/workbench/src/inspect.rs` | Project-owned SidecarRuntime event server and typed control protocol. |
| `runseal.toml` | Explicit local resources, Deno policy, and repository environment injection. |
| `flavor.toml` | Consumer-owned code-shape scan scope and rule adjustments. |
| `sidecar.toml` | Local runtime identity, native workbench app target, readiness, and inspect endpoint. |
| `.runseal/deno.json` | Deno compiler and formatter policy for repository wrappers. |
| `.runseal/deno.lock` | Frozen Deno dependency resolution for repository wrappers. |
| `.runseal/hooks/pre-commit` | Git pre-commit entrypoint delegating to `runseal :guard`. |
| `.runseal/wrappers/init.ts` | Stable tool validation and repository hook installation. |
| `.runseal/wrappers/guard.ts` | Canonical Rust, Flavor, and Sidecar validation workflow. |
| `.runseal/wrappers/gpu-lab.ts` | Canonical Experiment 0001 bootstrap and execution workflow. |
| `.runseal/wrappers/workbench.ts` | Canonical workbench lifecycle and typed inspect workflow. |

## 5. Core Operational Workflows

### 5.1 Cold start

The R0 repository baseline is defined by the core files indexed above. R1 accepted a
Rust-based native D3D12 GPU laboratory on the single reference platform recorded in ADR
0001. ADR 0003 accepts the first operator-visible workbench cold start.

The workbench is a composition root, not permission to create broad engine scaffolding.
Do not begin scene, ECS, asset, or graphics-pipeline work until the next numbered
experiment defines and accepts its hypothesis, workload, and criteria.

Canonical commands from the repository root:

```powershell
runseal :init
runseal :guard
runseal :gpu-lab correctness
runseal :gpu-lab benchmark
runseal :workbench start
runseal :workbench status
runseal :workbench inspect
runseal :workbench color 0.08 0.42 0.24
runseal :workbench pause
runseal :workbench resume
runseal :workbench restart
runseal :workbench stop
```

Correctness mode requires the Windows optional capability
`Tools.Graphics.DirectX~~~~0.0.1.0`. Benchmark mode intentionally runs without the debug
layer and must report that validation is disabled.

The wrappers use installed stable-channel Flavor, Runseal, and Sidecar CLIs. Sibling
source checkouts are references, not runtime dependencies. The workbench accepts the
canonical `--sidecar-stamp` argument and exposes only the typed events recorded in ADR
0003.

### 5.2 Experiment lifecycle

1. State one falsifiable architectural hypothesis.
2. Define the representative workload, controlled variables, recorded metrics, and
   pass/fail criteria before implementation.
3. Implement the smallest isolated system capable of testing that hypothesis.
4. Run repeatable measurements with fixed input, fixed camera or seed where relevant,
   warm-up, and synchronization controls documented.
5. Record environment metadata including hardware, driver, build mode, revision, and
   experiment parameters.
6. Report distributions and scaling curves, not only average FPS. Include relevant CPU
   and GPU timings, upload volume, allocation behavior, memory use, and synchronization.
7. Decide explicitly: promote, revise and repeat, or reject.
8. Promote passing work into the engine only with a stable regression benchmark that
   preserves the proven property.

### 5.3 Core implementation change

1. Identify the owning module and the experiment or accepted requirement authorizing
   the change.
2. Preserve existing public contracts unless the change explicitly replaces them.
3. Keep hot-path data ownership, lifetime, threading, and CPU/GPU synchronization
   visible in the implementation.
4. Run the narrowest relevant correctness checks, then the affected regression
   benchmarks.
5. Update architecture decisions, experiment conclusions, commands, and the core file
   index when their source of truth changes.

### 5.4 Benchmark execution

- Use release/optimized builds and disable presentation pacing when measuring raw
  throughput.
- Separate CPU simulation, render preparation, GPU execution, streaming, and
  presentation measurements.
- Report at least median, P95, and P99 frame or task times when enough samples exist.
- Sweep workload size to expose the cost curve and saturation point.
- Treat validation errors, device loss, hidden fallback behavior, unbounded memory
  growth, and unexplained synchronization as failures even when frame rate appears high.
- Keep summarized conclusions under version control when the documentation structure
  exists. Store bulky raw captures and generated output under ignored local paths.

### 5.5 Mod content workflow

- Mod implementation begins only after its required engine capabilities have passed
  their experimental gates.
- Mod data and scripts use documented engine-facing contracts; they must not depend on
  engine internals or experiment-only APIs.
- Original proprietary assets, code, credentials, and redistributable content without
  clear permission do not enter the repository.
- Legacy import, if later authorized, converts into canonical engine formats and remains
  optional to the engine and mod source trees.
