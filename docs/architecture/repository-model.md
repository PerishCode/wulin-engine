# Repository Model

## State

Experiments through 0046 and ADR 0049 define the accepted canonical content runtime, reference
host, first prototype composition root, exact CPU terrain query/body contact, and retired
compatibility surface. The runtime remains in `crates/engine-runtime`. It owns camera state, signed
terrain/object streaming, atomic composition, traversal/prefetch/rollover, rendering, presentation
time, exact committed-snapshot terrain queries and caller-owned vertical contact resolution,
neutral frame targets, shaders, probes, and GPU device/resource lifecycle. It has no calibration
scene, split-world control state, body store, or simulation clock. The
format/catalog crates and offline cookers remain independent reusable owners below it.

The runtime also owns the sole mutable presentation timeline and successful-frame commit. The
renderer consumes an immutable pre-commit tick for GPU work and evidence; it cannot pause, set,
step, or advance time. Elapsed time and simulation-step policy remain unpromoted.

`crates/reference-host` owns the concrete Windows single-window/message lifecycle, normalized
keyboard/focus state and bounded journal, strict bootstrap config/path validation, and hidden
canonical-ready driver. It is not a cross-platform abstraction.

`apps/workbench` is the native diagnostic composition root. It retains inspect transport,
operator capture persistence, perception response shaping, diagnostic readiness, pause/failure
shaping, and fault gates. `apps/prototype` is the plain non-diagnostic composition root: configured
canonical startup is mandatory, it continuously frames the same runtime, and Escape only requests
host exit. Simulation stepping, terrain contact consumers, runtime actors, camera actions, and
gameplay interaction remain unpromoted. Directories are created only when they own real files.

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
