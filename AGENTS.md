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

Experiments 0031-0054 and the current ADR set through 0057 define one live content runtime
with explicit object presentation authority, deterministic frame-driven presentation time,
one explicit deterministic simulation schedule, one caller-owned fixed terrain-motion consumer,
one caller-owned bounded planar translation and planar-first combined tick, one retained terrain
body lifecycle and transactional stored advance, one canonical
translatable terrain position, one offline-cooked external geometry/material/rig source, and one
deterministic object-shadow path:

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
  presentation-time, simulation-schedule, shader, and GPU lifecycle owners;
- one runtime frame transaction that renders an immutable pre-commit tick and advances only after
  a successful canonical frame;
- one runtime-owned rational 60 Hz simulation schedule driven only by explicit bounded elapsed
  nanoseconds, independent from frames and presentation, with no live clock or internal step loop;
- one caller-owned exact vertical terrain-body motion transaction that consumes exactly one fixed
  tick through checked semi-implicit integration and committed-snapshot contact, without mutating
  retained state, horizontal velocity, locomotion controller, or gameplay tuning;
- one caller-owned bounded planar terrain-body translation that composes exact canonical position
  and committed-snapshot contact, preserves vertical velocity, returns unchanged input when upward
  correction exceeds the explicit limit, and never snaps downhill;
- one caller-owned planar-first terrain-body advance that reuses accepted destination terrain,
  queries retained origin only after a distinct blocked candidate, and then executes exactly one
  fixed vertical step so downhill and blocked intent both progress in the same tick;
- one runtime-owned optional retained `TerrainBodyMotion` with capacity one, checked nonzero
  generation handles, exact spawn/read/despawn semantics, and no multi-body or actor policy;
- one handle-addressed retained planar-first advance that validates before query, commits only the
  complete successful copied-value output under the same generation, and preserves state on every
  validation, query, arithmetic, or contact failure;
- one signed-region/half-open-local-Q9 `TerrainPosition` shared by query/contact/motion, with exact
  checked positive, negative, and multi-region planar translation and no compatibility alias;
- one bounded 225-body contact transition witness in the generic canonical probe; the historical
  230,400-body checkpoint has no live inspect verb, runtime branch, or coverage mode;
- one host-owned Win32 keyboard/focus adapter and bounded process-local normalized input journal
  with isolated deterministic replay;
- one optional strict schema-1 bootstrap document that selects both sources and one signed global
  target, hides async progress, and emits readiness only after a canonical frame;
- one concrete Windows reference-host owner for the single window/message lifecycle, normalized
  input journal, bootstrap parser, and canonical-ready driver;
- one mandatory-bootstrap, non-diagnostic prototype composition root over the same runtime, with
  Escape limited to host exit;
- one exact read-only CPU terrain-height query over the committed snapshot, addressed by signed
  region plus half-open local Q9 and independent from camera, render LOD, source I/O, and GPU work;
- one caller-owned exact vertical terrain-body contact transaction with strict
  separated/touching/penetrating classification, minimum upward correction, and no runtime body
  mutation, gravity, or locomotion policy;
- one clear-only diagnostic idle shell with neutral reverse-Z depth and semantic frame targets,
  no calibration scene, and no split-world control surface;
- one compact `input.*` / `simulation.*` / `camera.*` / `source.*` / `canonical.*` inspect
  vocabulary;
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
| `sidecar.bootstrap.toml` | Configured canonical-readiness workbench lifecycle. |
| `sidecar.prototype.toml` | Non-diagnostic configured prototype lifecycle. |
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
| `docs/adr/0045-canonical-bootstrap-readiness.md` | Accepted strict bootstrap schema, terminal failure, hidden progress, and canonical-ready contract. |
| `docs/adr/0046-reference-platform-host.md` | Accepted concrete Windows host ownership, workbench/prototype separation, and non-diagnostic composition contract. |
| `docs/adr/0047-canonical-terrain-query.md` | Accepted signed fixed-point CPU terrain-height query and published-snapshot failure contract. |
| `docs/adr/0048-idle-shell-compatibility-removal.md` | Accepted calibration/split-world removal, clear-only idle shell, and neutral frame-target ownership. |
| `docs/adr/0049-exact-terrain-body-contact.md` | Accepted caller-owned exact terrain contact, minimum correction, bounded witness, and deferred simulation policy. |
| `docs/adr/0050-runtime-fixed-simulation-schedule.md` | Accepted explicit rational fixed schedule, transactional bounds, and presentation-independent time contract. |
| `docs/adr/0051-caller-owned-fixed-terrain-motion.md` | Accepted caller-owned one-tick vertical motion, exact contact composition, and deferred live-driving contract. |
| `docs/adr/0052-canonical-terrain-position-translation.md` | Accepted query-neutral terrain position, Euclidean seam normalization, and checked translation contract. |
| `docs/adr/0053-retired-dense-contact-acceptance.md` | Accepted removal of the historical dense contact command/mode and retention of one bounded witness. |
| `docs/adr/0054-bounded-terrain-body-translation.md` | Accepted caller-owned exact planar terrain-body translation, explicit step-up bound, and atomic blocked-output contract. |
| `docs/adr/0055-planar-first-terrain-body-advance.md` | Accepted planar-first one-tick terrain-body composition, destination reuse, and blocked-origin vertical progress. |
| `docs/adr/0056-retained-terrain-body-lifecycle.md` | Accepted single-slot runtime body ownership, generation lifetime, stale-handle rejection, and deferred actor storage. |
| `docs/adr/0057-transactional-retained-body-advance.md` | Accepted retained read-compute-commit ordering, unchanged generation, and exact failure rollback. |
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
| `experiments/0042-declarative-runtime-bootstrap/README.md` | Accepted configured source/target startup, no-ready failure, canonical readiness, and restart evidence. |
| `experiments/0043-thin-prototype-host/README.md` | Accepted shared reference host, plain prototype startup, terminal failure, and lifecycle evidence. |
| `experiments/0044-exact-canonical-terrain-query/README.md` | Accepted exact CPU height query, independent oracle, atomic publication, and lifecycle evidence. |
| `experiments/0045-active-compatibility-removal/README.md` | Accepted calibration/world compatibility removal, idle zero-semantic evidence, and exact canonical regression. |
| `experiments/0046-exact-terrain-body-contact/README.md` | Accepted exact terrain body contact, explicit dense proof, compact transition witness, and lifecycle evidence. |
| `experiments/0047-deterministic-simulation-schedule/README.md` | Accepted exact 60 Hz rational schedule, partition/replay, rollback, independence, and lifecycle evidence. |
| `experiments/0048-fixed-terrain-body-motion/README.md` | Accepted one-tick terrain-body motion, schedule-partition replay, rollback, and zero-non-CPU-work evidence. |
| `experiments/0049-exact-terrain-position-translation/README.md` | Accepted canonical terrain position, exact signed seam translation, overflow rollback, and oracle-sweep evidence. |
| `experiments/0050-retired-dense-contact-surface/README.md` | Accepted dense contact history removal, retired-verb rejection, and bounded-witness preservation evidence. |
| `experiments/0051-bounded-terrain-body-translation/README.md` | Accepted exact planar body translation, bounded upward correction, blocked identity, downhill separation, and replay evidence. |
| `experiments/0052-planar-first-terrain-body-advance/README.md` | Accepted planar-first combined tick, one/two-query ordering, same-tick downhill, blocked-origin progress, and replay evidence. |
| `experiments/0053-retained-terrain-body-lifecycle/README.md` | Accepted single retained body, exact lifecycle rollback, generation invalidation, process reset, and replay evidence. |
| `experiments/0054-transactional-retained-body-advance/README.md` | Accepted handle-addressed stored advance, commit-after-success, exact rollback, and replay evidence. |
| `assets/third-party/khronos-fox/README.md` | Pinned Khronos Fox source provenance, hashes, attribution, and redistributable license record. |
| `crates/engine-runtime/Cargo.toml` | Canonical runtime package and dependency boundary. |
| `crates/engine-runtime/build.rs` | Runtime shader compilation, Agility export linkage, and native SDK staging. |
| `crates/engine-runtime/src/lib.rs` | Public runtime, capture, semantic, and signed-address surface. |
| `crates/engine-runtime/src/runtime/mod.rs` | Sole renderer/scene facade, frame-transaction coordinator, simulation schedule/retained-body owner, and committed-terrain transaction entry point. |
| `crates/engine-runtime/src/runtime/retained_body.rs` | Single retained terrain-body slot, nonzero generation handle, lifecycle, and checked motion replacement. |
| `crates/engine-runtime/src/region.rs` | Signed global region value and checked offset owner. |
| `crates/engine-runtime/src/timeline/mod.rs` | Presentation and simulation timeline ownership boundary. |
| `crates/engine-runtime/src/timeline/presentation.rs` | Deterministic presentation state, controls, counters, and successful-frame commit. |
| `crates/engine-runtime/src/timeline/simulation.rs` | Exact rational simulation accumulator, checked transaction, typed batch, and isolated long-duration probe. |
| `crates/engine-runtime/src/terrain_query/mod.rs` | Exact height query, caller-owned body, and minimum-correction contact transaction. |
| `crates/engine-runtime/src/terrain_query/advance.rs` | Planar-first translation/vertical composition, destination-height reuse, ordered blocked-origin query, and final tick output. |
| `crates/engine-runtime/src/terrain_query/motion.rs` | Caller-owned fixed vertical motion, checked one-tick integration, and grounded composition. |
| `crates/engine-runtime/src/terrain_query/position.rs` | Canonical signed-region/local-Q9 terrain position and checked Euclidean translation. |
| `crates/engine-runtime/src/terrain_query/translation.rs` | Caller-owned exact planar body candidate, one-query contact composition, step-up bound, and atomic output decision. |
| `crates/reference-host/src/window.rs` | Concrete single-window Win32 lifecycle, message pump, native input capture, and close signaling. |
| `crates/reference-host/src/input.rs` | Normalized key state, bounded record lifecycle, canonical hashing, isolated replay, and held-state query. |
| `crates/reference-host/src/bootstrap.rs` | Strict arguments/config/pack paths and hidden canonical-ready bootstrap driver. |
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
| `apps/prototype/src/main.rs` | Mandatory-bootstrap non-diagnostic composition root, continuous frame loop, and host-exit input consumer. |
| `apps/workbench/src/main.rs` | Diagnostic composition root, frame loop, and pending operator dispatch. |
| `apps/workbench/src/inspect/protocol.rs` | Compact workbench control vocabulary. |
| `apps/workbench/src/inspect/protocol/terrain.rs` | Strict terrain query, motion, advance, and retained-body lifecycle payload decoding. |
| `apps/workbench/src/inspect/app.rs` | Main-thread control dispatch. |
| `apps/workbench/src/inspect/app/retained_body.rs` | Strict retained-body lifecycle/advance dispatch and zero-non-CPU-work evidence response. |
| `crates/engine-runtime/src/streaming/address.rs` | Signed global window and bounded projection. |
| `crates/engine-runtime/src/streaming/objects/mod.rs` | Bounded schema-3 object I/O transactions. |
| `crates/engine-runtime/src/streaming/terrain/mod.rs` | Bounded signed terrain I/O transactions. |
| `crates/engine-runtime/src/rendering/async_resident/transfer.rs` | Object GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/terrain/transfer.rs` | Terrain GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/composition/mod.rs` | Atomic pair publication and fixed composition. |
| `crates/engine-runtime/src/rendering/composition/traversal.rs` | Latest-wins traversal, prefetch, and rollover policy. |
| `crates/engine-runtime/src/rendering/composition/probe.rs` | Canonical attachment and oracle evidence. |
| `crates/engine-runtime/src/rendering/composition/probe/terrain_query.rs` | Dense query/contact oracle evidence and compact body-contact transition witness. |
| `crates/engine-runtime/src/rendering/frame_targets.rs` | Neutral reverse-Z depth and semantic render-target ownership. |
| `crates/engine-runtime/src/rendering/renderer/frame.rs` | Clear-only idle-shell/canonical frame dispatch. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/surface/shadow.rs` | Fixed directional-light projection and shadow probe oracle. |
| `.runseal/wrappers/init.ts` | Toolchain and repository initialization. |
| `.runseal/wrappers/guard.ts` | Repository/runtime ownership, dependency, and retired compatibility-symbol gates. |
| `.runseal/wrappers/gpu-lab.ts` | Experiment 0001 operator entry point. |
| `.runseal/wrappers/workbench.ts` | Compact manual workbench control. |
| `.runseal/wrappers/canonical-runtime.ts` | Direct Experiment 0054 acceptance entry point over the converged runtime. |
| `.runseal/support/canonical-runtime.ts` | Non-recursive canonical acceptance support. |
| `.runseal/support/compatibility-removal.ts` | Clear-only idle capture and retired inspect-verb rejection evidence. |
| `.runseal/support/terrain/contact.ts` | Exact contact rejection, direct classification, and bounded-witness acceptance support. |
| `.runseal/support/guard/contact-removal.ts` | Forbidden-symbol gate for the retired dense contact command and runtime coverage mode. |
| `.runseal/support/terrain/motion.ts` | Fixed-step trajectory, schedule-partition replay, rollback, restart, and independence acceptance support. |
| `.runseal/support/terrain/translation.ts` | Real-snapshot bounded translation, blocked identity, downhill, seam, replay, rollback, and independence acceptance support. |
| `.runseal/support/terrain/advance.ts` | Real-snapshot planar-first ordering, query reuse/order, same-tick downhill, grouped replay, rollback, and independence support. |
| `.runseal/support/terrain/retained-body.ts` | Retained-body lifecycle, stale-handle rollback, generation replay, restart reset, and independence support. |
| `.runseal/support/terrain/retained-advance.ts` | Retained read-compute-commit, query ordering, failure rollback, replay, and independence support. |
| `.runseal/support/simulation-schedule.ts` | Partition, replay, rollback, process reset, and temporal-independence acceptance support. |
| `.runseal/support/host-input-replay.ts` | Native message, paused record/replay, invalid-operation, and process-restart acceptance support. |
| `.runseal/support/runtime-bootstrap.ts` | Configured failure, canonical-ready, exact restart, and cleanup acceptance support. |
| `.runseal/support/prototype-host.ts` | Prototype no-ready failure, exact readiness, restart, and no-inspect lifecycle support. |
| `.runseal/support/terrain/query.ts` | Exact single-query rejection, seam, triangle, and dense snapshot acceptance support. |
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
presentation time, deterministic host input and process-restart replay, configured canonical
readiness, shared reference-host ownership, prototype startup/restart/cleanup, fixed camera-visible
directional object shadows, exact CPU terrain-height query/body contact and oracle evidence, a
bounded contact transition witness, the explicit simulation schedule and its partition/replay,
rollback, restart, frame, and presentation-independence gates, exact fixed terrain-body motion and
schedule-partition replay, bounded planar terrain-body translation and replay, planar-first combined
advance, query reuse/order, and grouped replay, a same-process
clear-only idle attachment capture, retired-control rejection, 64-publication resource plateau,
and 16 complete lifecycle cycles. It must not invoke an older experiment wrapper.

Generated evidence belongs under
`out/captures/0054-transactional-retained-body-advance/` and remains ignored.

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

The only frame outcomes are clear-only `idle-shell` before a pair is published and
`canonical-runtime` afterward. The idle shell has no scene or semantic object. Manual controls do
not select renderer modes, fixture variants, pass order, or local schedules.

### 6.4 Plain prototype

```powershell
# With out/cooked/bootstrap/runtime.json prepared:
sidecar start --config sidecar.prototype.toml
sidecar stop --config sidecar.prototype.toml
```

The prototype has no inspect endpoint or idle-shell mode. It shows the same canonical runtime only
after configured content is ready; window close, Escape, and Sidecar stop are its current lifecycle
controls. Camera actions, live simulation/motion driving, and runtime actors are not part of this
workflow.

### 6.5 Experiment lifecycle

1. State the hypothesis, workload, controlled variables, metrics, pass criteria, and
   evidence path before implementation.
2. Keep the proof isolated until its acceptance criteria pass.
3. Record failures as evidence; do not conceal them with fallback behavior.
4. Promote only proven reusable ownership into `crates/` or `benchmarks/`.
5. Update this file when core ownership or stable workflows change.

### 6.6 Core implementation change

1. Inspect the working tree and relevant owner files.
2. Change the narrowest responsible boundary without compatibility scaffolding.
3. Run focused checks while iterating.
4. Run `runseal :guard` before accepting the change.
5. Run the active GPU experiment workflow when GPU behavior or lifecycle changes.

### 6.7 Mod content workflow

- Add Wulin-specific content only after its engine dependency has passed its experiment.
- Keep Wulin code and data under `mods/wulin/`.
- Do not modify engine behavior solely to reproduce a game-specific quirk without an
  explicit engine-level requirement.
