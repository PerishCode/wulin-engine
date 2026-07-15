# Repository Model

## State

Experiments through 0054 and ADR 0057 define the accepted canonical content runtime, reference
host, first prototype composition root, exact CPU terrain query/body contact and fixed vertical
motion, exact planar-first terrain advance, deterministic simulation schedule, one retained body
lifecycle plus transactional stored advance, and retired compatibility/history surfaces. The
runtime remains in
`crates/engine-runtime`. It owns camera state, signed
terrain/object streaming, atomic composition, traversal/prefetch/rollover, rendering, presentation
time, the explicit rational 60 Hz simulation schedule, one signed-region/half-open-local-Q9 terrain
position with exact checked translation, exact committed-snapshot terrain queries, caller-owned
vertical contact/motion transactions, neutral frame targets, shaders, probes, and GPU
device/resource lifecycle. It also owns one optional neutral `TerrainBodyMotion` behind a checked
nonzero generation handle. It has no calibration scene, split-world control state, multi-body
store, live wall-clock driver, or runtime-owned step loop. The
format/catalog crates and offline cookers remain independent reusable owners below it.

The runtime owns the sole mutable presentation timeline, successful-frame commit, and simulation
schedule. The renderer consumes an immutable pre-commit tick for GPU work and evidence; it cannot
pause, set,
step, or advance time. Simulation advances only from explicit bounded elapsed nanoseconds and is
independent from presentation. Caller-owned terrain motion is the first explicit one-tick consumer;
the retained slot establishes process-local ownership and spawn/read/despawn lifetime. One explicit
handle-addressed operation now runs the same planar-first tick and commits only after success. No
host samples monotonic time or drives returned batches. Stall splitting,
focus policy, and live
step driving remain unpromoted.

`TerrainPosition` is the sole horizontal identity shared by terrain query, contact, and fixed
motion. Its pure Q9 translation canonicalizes positive, negative, and multi-region displacement
without sampling terrain. Bounded planar contact composition and planar-first vertical ordering are
accepted. Slope policy, input mapping, multi-actor storage, and presentation binding remain
unpromoted.

Exact contact retains one public direct transaction and one 225-body witness embedded in the
generic canonical probe. The accepted one-time 230,400-body dense checkpoint is documentation-only;
its inspect verb, runtime/renderer branch, and coverage mode are forbidden from the live surface.

`crates/reference-host` owns the concrete Windows single-window/message lifecycle, normalized
keyboard/focus state and bounded journal, strict bootstrap config/path validation, and hidden
canonical-ready driver. It is not a cross-platform abstraction.

`apps/workbench` is the native diagnostic composition root. It retains inspect transport,
operator capture persistence, perception response shaping, diagnostic readiness, pause/failure
shaping, and fault gates. `apps/prototype` is the plain non-diagnostic composition root: configured
canonical startup is mandatory, it continuously frames the same runtime, and Escape only requests
host exit. Live schedule/motion driving, runtime actors, camera actions, and
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
