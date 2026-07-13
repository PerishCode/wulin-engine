# Repository Model

## State

The repository has completed the native workbench cold start through exact-ground,
LOD-rendered terrain/object composition. `crates/region-format`, its offline writer under
`tools/region-cooker`, `crates/meshlet-catalog`, `crates/animation-catalog`, and
`crates/surface-catalog` are promoted by Experiments 0008-0011 and ADRs 0011-0014.
Experiment 0012 and ADR 0015 accept workbench-owned hierarchy, invalidation, query, and
stable compaction contracts. Experiment 0013 and ADR 0016 promote the independent
`crates/terrain-format` and `tools/terrain-cooker` owners while terrain streaming, GPU
expansion, and probes remain workbench-owned. Experiments 0015-0016 and ADRs 0018-0019
accept atomic terrain/object publication and exact arbitrary-position GPU sampling but
keep composition workbench-owned. Experiment 0017 and ADR 0020 accept terrain render
LOD with exact full-resolution object ground and bounded fixture contact error without
promoting a new reusable owner. Other engine systems remain workbench-owned until an
experiment establishes a reusable boundary. Directories are created only when they own
real files.

## Dependency direction

```text
apps --------> crates
  `----------> mods --------> crates

benchmarks --> crates
tools -------> explicitly reusable crates

experiments   remain isolated
    |
    `-- accepted implementation is rewritten into crates

crates must not depend on apps, mods, benchmarks, or experiments
engine crates must not depend on the Wulin mod
```

An experiment may copy or prototype a concept locally. Production modules must not
depend on experiment code. Promotion is a deliberate rewrite around accepted ownership,
contracts, tests, and regression benchmarks.

## Top-level ownership

| Path | Responsibility | Creation gate |
| --- | --- | --- |
| `apps/` | Runnable clients, servers, editors, and composition roots. | First real executable outside an isolated experiment. |
| `crates/` | Reusable engine and shared runtime modules. | An accepted experiment or explicit architecture decision owns reusable code. |
| `experiments/` | Isolated, falsifiable architecture experiments. | The first accepted experiment definition. |
| `benchmarks/` | Stable regression workloads for promoted capabilities. | An experiment passes and is promoted. |
| `mods/` | Mod-owned code, scripts, data, and UI. | Required engine capabilities have passed their gates. |
| `tools/` | Offline import, build, inspection, and developer utilities. | A real workflow requires a reusable tool. |
| `assets/` | Redistributable source and test assets with recorded provenance. | A real experiment or product path requires checked-in assets. |
| `tests/` | Repository-level integration and end-to-end tests. | A contract crosses module or process boundaries. |
| `docs/` | Architecture, ADRs, experiment protocols, and contributor references. | Created in R0. |
| `out/` | Ignored captures, raw reports, logs, and generated output. | Created locally by a tool or experiment. |
| `.runseal/` | Repository hooks and explicit Deno operator wrappers. | Created when ADR 0002 is accepted. |

Root `runseal.toml`, `flavor.toml`, and `sidecar.toml` are consumer-owned contracts for
the personal iteration suite. They do not make the tools part of the engine dependency
graph.

## Naming

- Directories, source files, packages, modules, and identifiers use English.
- Rust packages use kebab-case; Rust modules use snake_case.
- Experiments use `NNNN-kebab-case`, beginning with `0001`.
- ADRs use `NNNN-kebab-case.md`, beginning with `0001`.
- An experiment ID is permanent even when the experiment is rejected or superseded.
- Do not reuse deleted or abandoned IDs.

## Experiment promotion

1. Define the hypothesis and acceptance criteria before implementation.
2. Keep the implementation inside its numbered experiment directory.
3. Record reproducible parameters and summarized evidence.
4. Accept, revise, or reject the hypothesis explicitly.
5. For accepted work, record any architecture decision required for production use.
6. Rewrite the accepted mechanism into the owning crate with correctness tests.
7. Promote the workload into `benchmarks/` to protect the measured property.
8. Remove production dependencies on the experiment before considering promotion done.

## Source and generated data

- Source files, reproducible configuration, and summarized conclusions are versioned.
- `target/`, `out/`, GPU captures, compiled shaders, logs, and crash dumps are generated
  and ignored.
- Runtime asset formats are outputs of an explicit build process, not hand-edited files.
- Proprietary legacy data never becomes an implicit build dependency.
