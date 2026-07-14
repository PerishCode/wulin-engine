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
  and detailed technical decisions under `docs/`.
- The core file index lists only files that currently exist. Add an entry when a core
  file is created and remove or update it when that file moves or is deleted.
- Do not turn temporary experiment commands into repository-wide rules until they are
  stable and repeatable.

## 2. Purpose

This repository builds and validates a modern, lightweight, GPU-oriented game engine,
followed by a large Wulin Zhuan mod that consumes proven engine systems.

The project is an open-source architecture experiment, not a commercial product and
not a general-purpose engine. Its primary objective is to prove that modern workload
organization can make the rendering and simulation profile of this class of MMORPG
structurally inexpensive.

The project follows these principles:

- Prove capability before building content on top of it.
- Advance through explicit experimental gates rather than a feature checklist.
- Judge performance by scaling curves, frame-time stability, data movement, resource
  lifetime, and synchronization behavior instead of a target GPU model.
- Optimize work elimination, batching, GPU residency, and asynchronous execution before
  optimizing isolated instructions.
- Use one reference development platform while the architecture is being validated.
  Broad hardware, vendor, graphics-API, operating-system, and legacy compatibility are
  out of scope unless explicitly promoted later.
- Keep gameplay authoritative on the CPU or server where appropriate while moving
  suitable rendering, animation, visibility, and simulation workloads to the GPU.
- Add Wulin mod content only after the underlying engine capabilities pass their
  experiments. The mod consumes engine capabilities rather than compensating for gaps.

## 3. Directory Conventions

Top-level directories are created only when they contain real work. Empty architecture
scaffolding is discouraged.

| Path | Ownership |
| --- | --- |
| `apps/` | Runnable clients, servers, editors, and product entry points. |
| `crates/` | Reusable engine and shared runtime modules. |
| `experiments/` | Isolated proofs with hypotheses, metrics, gates, and results. |
| `benchmarks/` | Stable regression workloads promoted from successful experiments. |
| `mods/` | Mod-specific code, scripts, data, UI, and configuration. |
| `tools/` | Offline processing, import, inspection, profiling, and developer utilities. |
| `assets/` | Redistributable source and test assets with provenance and licensing. |
| `tests/` | Repository integration and end-to-end tests. |
| `docs/` | Architecture, ADRs, experiment summaries, and operational references. |
| `out/` | Disposable local output, captures, reports, and generated artifacts. |
| `.runseal/` | Repository hooks and explicit operator workflows. |

Additional conventions:

- Keep source assets, cooked runtime assets, and generated experiment output distinct.
- Legacy-format research and import code remain isolated from canonical runtime formats
  and engine ownership.
- A successful experiment may be promoted into `crates/` and `benchmarks/`; failed or
  superseded experiments retain only the evidence needed to explain the decision.
- Generated directories such as `target/`, `out/`, and tool caches are not hand-edited.
- Avoid deep nesting until ownership boundaries justify it.

## 4. Current Runtime Boundary

Experiments 0031-0041 and the current ADR set through 0044 define one live content runtime
with explicit object presentation authority, deterministic frame-driven presentation time,
one offline-cooked external geometry/material/rig source, and one deterministic object-shadow
path:

- signed `i64` terrain packs (`.wlt`);
- signed schema-3 object packs (`.wlr`) with explicit authored local IDs and presentation;
- source-addressed 50-slot terrain and triple-plane object caches;
- atomic terrain-first canonical composition after an idle workbench shell;
- fixed arbitrary-Q8 grounding, terrain LOD, skeletal, surface, and occlusion execution;
- one runtime-owned 4,800-unit fixed-quantum clock with a 31,002,560-frame exact period and
  pause/set/step controls;
- one pinned glTF source cooked outside the runtime into imported archetype 7;
- one pinned PNG/PBR material cooked into reserved surface material/layer 63;
- one pinned 24-joint/three-clip skin cooked into imported fixed rig bank 1;
- exact source-duration phase mapping over the existing 64 sampled poses;
- one fixed camera-visible directional hard-shadow map and depth-only object pass;
- one engine-owned `Runtime` facade containing the sole scene, renderer, streaming, composition,
  presentation-time, shader, and GPU lifecycle owners;
- one runtime frame transaction that renders an immutable pre-commit tick and advances only after
  a successful canonical frame;
- one host-owned Win32 keyboard/focus adapter and bounded process-local normalized input journal
  with isolated deterministic replay;
- one compact `source.*` / `canonical.*` inspect vocabulary;
- one non-recursive `runseal :canonical-runtime` acceptance workflow.

Historical experiment READMEs and ADRs remain decision history. Their runtime modes,
formats, controls, and wrappers are not live compatibility surfaces.

## 5. Core File Index

| File | Responsibility |
| --- | --- |
| `README.md` | Public scope, project status, and current commands. |
| `Cargo.toml` | Rust workspace and shared dependency policy. |
| `rust-toolchain.toml` | Pinned Rust toolchain and components. |
| `flavor.toml` | Live source-quality boundaries. |
| `runseal.toml` | Runseal permissions and local resource injection. |
| `sidecar.toml` | Debug-layer workbench lifecycle. |
| `sidecar.benchmark.toml` | Release workbench lifecycle. |
| `docs/architecture/repository-model.md` | Ownership and dependency direction. |
| `docs/adr/README.md` | ADR naming, status, and maintenance rules. |
| `docs/adr/0034-canonical-runtime-convergence.md` | Accepted single-runtime, operator-surface, and attachment contract. |
| `docs/adr/0035-authored-object-presentation.md` | Accepted schema-3 triple-plane object presentation authority and publication contract. |
| `docs/adr/0036-deterministic-temporal-presentation.md` | Superseded initial modulo-64 frame clock and deterministic control contract. |
| `docs/adr/0037-cooked-gltf-geometry.md` | Accepted pinned glTF source, offline canonical cook, imported archetype, and runtime isolation contract. |
| `docs/adr/0038-cooked-gltf-material.md` | Accepted pinned PNG/PBR join, deterministic mip cook, reserved fixed-array material, and runtime isolation contract. |
| `docs/adr/0039-cooked-gltf-skeletal-animation.md` | Accepted pinned skin/clip cook, dual fixed rig banks, rig-aware pose identity, and normalized-space skinning contract. |
| `docs/adr/0040-source-duration-presentation-time.md` | Superseded renderer-owned source-duration clock and integer phase contract. |
| `docs/adr/0041-camera-visible-directional-shadows.md` | Accepted fixed camera-visible object shadow map, indirect depth reuse, and deterministic receiver contract. |
| `docs/adr/0042-canonical-runtime-host-separation.md` | Accepted engine-runtime ownership, facade, host responsibilities, and dependency direction. |
| `docs/adr/0043-runtime-frame-transaction.md` | Accepted runtime timeline ownership, immutable render input, and successful-frame commit contract. |
| `docs/adr/0044-normalized-host-input-journal.md` | Accepted host-native keyboard normalization, bounded journal, focus cleanup, and isolated replay contract. |
| `docs/experiments/README.md` | Experiment evidence and promotion rules. |
| `experiments/0031-canonical-runtime-convergence/README.md` | Accepted convergence workload, evidence, and conclusion. |
| `experiments/0032-authored-object-presentation/README.md` | Accepted explicit cooked archetype, material, orientation, animation, and triple-plane publication evidence. |
| `experiments/0033-deterministic-temporal-presentation/README.md` | Accepted deterministic frame-driven animation time, explicit stepping, held-pair continuity, and zero-data-movement evidence. |
| `experiments/0034-cooked-gltf-geometry/README.md` | Accepted offline glTF geometry cook, canonical payload, imported runtime archetype, and GPU visual evidence. |
| `experiments/0035-cooked-gltf-material/README.md` | Accepted offline Fox base-color/material cook, fixed surface-array integration, and exact GPU visual evidence. |
| `experiments/0036-cooked-gltf-skeletal-animation/README.md` | Accepted offline Fox skin/clip cook, dual-bank GPU deformation, bounded pose reuse, and articulated visual evidence. |
| `experiments/0037-source-duration-playback/README.md` | Accepted deterministic source-duration playback, common-period control, and exact Walk-loop evidence. |
| `experiments/0038-camera-visible-directional-shadows/README.md` | Accepted camera-visible animated-object hard shadows, exact CPU oracle, and bounded resource evidence. |
| `experiments/0039-canonical-runtime-host-separation/README.md` | Accepted behavior-neutral runtime promotion, host separation, and exact regression evidence. |
| `experiments/0040-runtime-frame-transaction/README.md` | Accepted runtime-owned timeline, immutable tick consumption, and successful-frame transaction evidence. |
| `experiments/0041-deterministic-host-input/README.md` | Accepted native keyboard/focus normalization, process-local replay, restart, and host-order evidence. |
| `assets/third-party/khronos-fox/README.md` | Pinned Khronos Fox source provenance, hashes, attribution, and redistributable license record. |
| `crates/engine-runtime/Cargo.toml` | Canonical runtime package and dependency boundary. |
| `crates/engine-runtime/build.rs` | Runtime shader compilation, Agility export linkage, and native SDK staging. |
| `crates/engine-runtime/src/lib.rs` | Public runtime, capture, semantic, and signed-address surface. |
| `crates/engine-runtime/src/runtime.rs` | Sole renderer/scene facade and frame-transaction coordinator. |
| `crates/engine-runtime/src/timeline.rs` | Deterministic presentation timeline state, controls, counters, and successful-frame commit. |
| `crates/meshlet-catalog/build.rs` | Verified build-time glTF geometry/joint/weight cook, normalization, normals, LOD simplification, and canonical payload emission. |
| `crates/meshlet-catalog/src/imported.rs` | Strict canonical imported-geometry/binding payload decoder and metadata owner. |
| `crates/meshlet-catalog/src/procedural.rs` | Retained deterministic fixture generation for procedural archetypes 0 through 6. |
| `crates/animation-catalog/build.rs` | Verified build-time Fox hierarchy, inverse-bind, clip sampling, normalized palette, and canonical payload cook. |
| `crates/animation-catalog/src/imported_rig.rs` | Strict canonical imported-rig payload decoder and metadata owner. |
| `crates/animation-catalog/src/lib.rs` | Dual fixed rig-bank catalog, source-duration clock constants, rig-aware CPU pose oracle, encoding, validation, and hashing. |
| `crates/surface-catalog/build.rs` | Verified build-time Fox material/PNG validation, box reduction, mip generation, and payload emission. |
| `crates/surface-catalog/src/imported_material.rs` | Strict canonical imported-material payload decoder, mip verification, and metadata owner. |
| `crates/region-format/src/global.rs` | Signed schema-3 spatial, identity, and presentation object pack codec. |
| `crates/terrain-format/src/global.rs` | Signed terrain pack codec and exact lookup. |
| `crates/canonical-object-fixture/src/lib.rs` | Deterministic arbitrary-Q8 authored object fixture. |
| `tools/region-cooker/src/main.rs` | Signed schema-3 object cooker CLI with physical triple ordering and controlled presentation profiles. |
| `tools/terrain-cooker/src/main.rs` | Signed terrain cooker CLI. |
| `apps/workbench/src/main.rs` | Native host message/frame loop and pending operator dispatch. |
| `apps/workbench/src/input.rs` | Host-owned normalized key state, bounded record lifecycle, canonical hashing, and isolated replay. |
| `apps/workbench/src/inspect/protocol.rs` | Compact workbench control vocabulary. |
| `apps/workbench/src/inspect/app.rs` | Main-thread control dispatch. |
| `crates/engine-runtime/src/streaming/address.rs` | Signed global window and bounded projection. |
| `crates/engine-runtime/src/streaming/objects/mod.rs` | Bounded schema-3 object I/O transactions. |
| `crates/engine-runtime/src/streaming/terrain/mod.rs` | Bounded signed terrain I/O transactions. |
| `crates/engine-runtime/src/rendering/async_resident/transfer.rs` | Object GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/terrain/transfer.rs` | Terrain GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/composition/mod.rs` | Atomic pair publication and fixed composition. |
| `crates/engine-runtime/src/rendering/composition/traversal.rs` | Latest-wins traversal, prefetch, and rollover policy. |
| `crates/engine-runtime/src/rendering/composition/probe.rs` | Canonical attachment and oracle evidence. |
| `crates/engine-runtime/src/rendering/renderer/frame.rs` | Idle-shell/canonical frame dispatch. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/surface/shadow.rs` | Fixed directional-light projection and shadow probe oracle. |
| `.runseal/wrappers/init.ts` | Toolchain and repository initialization. |
| `.runseal/wrappers/guard.ts` | Repository, runtime/timeline ownership, dependency, and forbidden-symbol gates. |
| `.runseal/wrappers/gpu-lab.ts` | Experiment 0001 operator entry point. |
| `.runseal/wrappers/workbench.ts` | Compact manual workbench control. |
| `.runseal/wrappers/canonical-runtime.ts` | Direct Experiment 0041 acceptance entry point over the converged runtime. |
| `.runseal/support/canonical-runtime.ts` | Non-recursive canonical acceptance support. |
| `.runseal/support/host-input-replay.ts` | Native message, paused record/replay, invalid-operation, and process-restart acceptance support. |
| `.runseal/support/cooked-gltf-presentation.ts` | Imported geometry/material/rig metadata, exact GPU palette, and controlled articulation acceptance support. |
| `.runseal/support/temporal-presentation.ts` | Fixed-quantum duration time, common-period, and held-pair acceptance support. |

## 6. Core Operational Workflows

### 6.1 Cold start

```powershell
runseal :init
runseal :guard
```

`init` validates the canonical repository surface and installs `.runseal/hooks` as the
Git hooks path. `guard` is the authoritative non-GPU repository gate.

### 6.2 Canonical runtime acceptance

```powershell
runseal :canonical-runtime
```

This workflow cooks fresh signed sources and directly validates canonical correctness,
source reordering, movement, aliasing, failure rollback, all four fault gates, reactive
and prepared traversal, rollover, the runtime-owned frame transaction and deterministic
presentation time, deterministic host input and process-restart replay, fixed camera-visible
directional object shadows, a same-process 64-publication resource plateau, and 16 complete
lifecycle cycles. It must not invoke an older experiment wrapper.

Generated evidence belongs under
`out/captures/0041-deterministic-host-input/` and remains ignored.

### 6.3 Manual workbench

```powershell
runseal :workbench start
runseal :workbench input
runseal :workbench input-record-start
runseal :workbench input-record-stop
runseal :workbench input-replay
runseal :workbench terrain-open out/cooked/example/terrain.wlt
runseal :workbench objects-open out/cooked/example/objects.wlr
runseal :workbench schedule 0 0 0 0 2
runseal :workbench probe
runseal :workbench stop
```

The only frame outcomes are `idle-shell` before a pair is published and
`canonical-runtime` afterward. Manual controls do not select renderer modes, fixture
variants, pass order, or local schedules.

### 6.4 Experiment lifecycle

1. State the hypothesis, workload, controlled variables, metrics, pass criteria, and
   evidence path before implementation.
2. Keep the proof isolated until its acceptance criteria pass.
3. Record failures as evidence; do not conceal them with fallback behavior.
4. Promote only proven reusable ownership into `crates/` or `benchmarks/`.
5. Update this file when core ownership or stable workflows change.

### 6.5 Core implementation change

1. Inspect the working tree and relevant owner files.
2. Change the narrowest responsible boundary without compatibility scaffolding.
3. Run focused checks while iterating.
4. Run `runseal :guard` before accepting the change.
5. Run the active GPU experiment workflow when GPU behavior or lifecycle changes.

### 6.6 Mod content workflow

- Add Wulin-specific content only after its engine dependency has passed its experiment.
- Keep Wulin code and data under `mods/wulin/`.
- Do not modify engine behavior solely to reproduce a game-specific quirk without an
  explicit engine-level requirement.
