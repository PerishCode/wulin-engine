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
| `docs/adr/0004-frame-artifact-contract.md` | Superseded initial D3D12 capture and generated-artifact contract. |
| `docs/adr/0005-capture-collection-contract.md` | Accepted constrained capture collection and artifact ownership contract. |
| `docs/adr/0006-spatial-and-depth-convention.md` | Accepted coordinate, unit, transform, and reverse-Z convention. |
| `docs/adr/0007-object-id-perception-contract.md` | Accepted integer object-ID attachment and bounded screen-perception contract. |
| `docs/adr/0008-region-addressed-gpu-work.md` | Accepted region-addressed candidate generation, GPU compaction, and indirect work contract. |
| `docs/adr/0009-resident-region-storage.md` | Accepted bounded default-heap region cache, active mapping, and transactional publication contract. |
| `docs/adr/0010-asynchronous-region-publication.md` | Accepted copy-queue ordering, immutable publication, protected slots, and bounded backpressure contract. |
| `docs/adr/0011-cooked-region-storage.md` | Accepted canonical pack, bounded background I/O, reservation, and rollback contract. |
| `docs/adr/0012-gpu-meshlet-scene-execution.md` | Accepted meshlet catalog, GPU cull/LOD, bounded indirect mesh execution, and capability contract. |
| `docs/adr/0013-gpu-skeletal-crowd-execution.md` | Accepted GPU pose reuse, bounded hierarchy evaluation, meshlet skinning, and fixed submission contract. |
| `docs/adr/0014-gpu-surface-visibility-resolve.md` | Accepted compact visibility, deterministic fragment winner, surface reconstruction, and fixed-screen resolve contract. |
| `docs/adr/0015-gpu-conservative-occlusion.md` | Accepted reverse-Z hierarchy, exact invalidation, conservative query, and stable GPU compaction contract. |
| `docs/adr/0016-gpu-streamed-terrain.md` | Accepted fixed terrain payload, global lattice continuity, bounded publication, and fixed GPU expansion contract. |
| `docs/adr/0017-gpu-terrain-lod-stitching.md` | Accepted GPU patch LOD selection, exact coarse-edge projection, fixed submission, and bounded validation contract. |
| `docs/adr/0018-atomic-terrain-object-composition.md` | Accepted matched terrain/object publication, exact integer grounding, and shared attachment composition contract. |
| `docs/adr/0019-gpu-arbitrary-terrain-sampling.md` | Accepted Q8 arbitrary-position triangle interpolation and cross-region boundary contract. |
| `docs/adr/0020-gpu-lod-terrain-composition.md` | Accepted exact-ground, bounded-contact terrain LOD composition contract. |
| `docs/adr/0021-camera-driven-region-traversal.md` | Accepted bounded camera mapping, latest-wins scheduling, blocked failure, and atomic traversal contract. |
| `docs/adr/0022-camera-relative-global-space.md` | Accepted signed global region, bounded split-position conversion, camera-at-origin rendering, and mode-isolation contract. |
| `docs/adr/0023-signed-terrain-addressing.md` | Accepted signed terrain cache identity, bounded local content alias, and transactional publication contract. |
| `docs/adr/0024-signed-atomic-composition.md` | Accepted shared signed terrain/object identity and matched global/local pair publication contract. |
| `docs/adr/0025-signed-camera-traversal.md` | Accepted frozen-origin signed camera traversal, checked extent, and latest-wins pair contract. |
| `docs/adr/0026-signed-terrain-storage.md` | Accepted signed-key terrain V2, source namespace, canonical residency, and alias-projection contract. |
| `docs/adr/0027-camera-relative-terrain-projection.md` | Accepted V2 centered terrain projection, camera translation, frame-local semantics, and signed inverse contract. |
| `docs/adr/0028-canonical-generated-object-composition.md` | Accepted canonical object source/cache, shared projection, stable-key, semantic inverse, and V2 atomic composition contract. |
| `docs/adr/0029-canonical-origin-rollover.md` | Accepted canonical-only safe-band rollover, commit-time camera translation, and old-frame retention contract. |
| `docs/adr/0030-bounded-canonical-traversal-prefetch.md` | Accepted one-target canonical preparation, promotion, stale-work isolation, and bounded cache reuse contract. |
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
| `experiments/0002-deterministic-visual-loop/README.md` | Experiment 0002 hypothesis, capture protocol, evidence, and accepted conclusion. |
| `experiments/0003-spatial-calibration-scene/README.md` | Experiment 0003 spatial hypothesis, workload, evidence, and accepted conclusion. |
| `experiments/0004-object-id-perception/README.md` | Experiment 0004 object-ID hypothesis, bounded-region evidence, and accepted conclusion. |
| `experiments/0005-gpu-region-compaction/README.md` | Experiment 0005 logical-world scaling workload, distributions, and accepted conclusion. |
| `experiments/0006-resident-region-streaming/README.md` | Experiment 0006 resident cache movement workload, transfer evidence, and accepted conclusion. |
| `experiments/0007-async-region-publication/README.md` | Experiment 0007 held-copy frame-continuity workload, evidence, and accepted conclusion. |
| `experiments/0008-cooked-region-io/README.md` | Experiment 0008 cooked format, bounded background I/O, failure rollback, and accepted evidence. |
| `experiments/0009-gpu-meshlet-scene/README.md` | Accepted Experiment 0009 real meshlet geometry, GPU LOD, oracle, sweep, and indirect-dispatch evidence. |
| `experiments/0010-gpu-skeletal-crowds/README.md` | Accepted Experiment 0010 GPU pose reuse, hierarchy, skinning, oracle, visual, and release timing evidence. |
| `experiments/0011-gpu-surface-resolve/README.md` | Accepted Experiment 0011 visibility payload, exact surface oracle, sweep, visual, and release timing evidence. |
| `experiments/0012-gpu-conservative-occlusion/README.md` | Accepted Experiment 0012 hierarchy, bound proof, stable compaction, exact-output, invalidation, and work-elimination evidence. |
| `experiments/0013-gpu-streamed-terrain/README.md` | Accepted Experiment 0013 canonical terrain payload, bounded residency, exact shared edges, failure rollback, and fixed mesh evidence. |
| `experiments/0014-gpu-terrain-lod-stitching/README.md` | Accepted Experiment 0014 GPU patch LOD selection, exact transition-edge projection, and work-reduction evidence. |
| `experiments/0015-atomic-terrain-object-composition/README.md` | Accepted Experiment 0015 matched publication, exact GPU grounding, shared depth/object-ID composition, and order-invariance evidence. |
| `experiments/0016-gpu-arbitrary-terrain-sampling/README.md` | Accepted Experiment 0016 exact arbitrary-position triangle sampling, boundary continuity, compatibility, and timing evidence. |
| `experiments/0017-gpu-lod-terrain-composition/README.md` | Accepted Experiment 0017 exact-ground, bounded-contact terrain LOD composition and timing evidence. |
| `experiments/0018-camera-driven-region-traversal/README.md` | Accepted Experiment 0018 camera boundaries, bounded latest-wins traversal, failure, continuity, and timing evidence. |
| `experiments/0019-camera-relative-global-space/README.md` | Accepted Experiment 0019 signed global regions, exact split conversion, far-anchor/rebase invariance, rejection, and timing evidence. |
| `experiments/0020-signed-terrain-addressing/README.md` | Accepted Experiment 0020 signed terrain residency, local alias, overlap, rollback, compatibility, and timing evidence. |
| `experiments/0021-signed-atomic-composition/README.md` | Accepted Experiment 0021 shared signed terrain/object cache identity, atomic holds, rollback, and timing evidence. |
| `experiments/0022-signed-camera-traversal/README.md` | Accepted Experiment 0022 signed boundaries, latest-wins pair scheduling, failure, restart, and timing evidence. |
| `experiments/0023-signed-terrain-storage/README.md` | Accepted Experiment 0023 signed pack identity, canonical residency, alias rebind, rollback, and timing evidence. |
| `experiments/0024-camera-relative-terrain/README.md` | Accepted Experiment 0024 V2 centered projection, semantic inversion, alias invariance, holds, and timing evidence. |
| `experiments/0025-canonical-object-composition/README.md` | Accepted Experiment 0025 canonical object identity, shared projection, V2 composition, source independence, holds, and timing evidence. |
| `experiments/0026-canonical-origin-rollover/README.md` | Accepted Experiment 0026 safe-band rollover, atomic camera translation, holds, failure recovery, and timing evidence. |
| `experiments/0027-canonical-traversal-prefetch/README.md` | Accepted Experiment 0027 one-target cache preparation, promotion, stale/failure isolation, rollover, and timing evidence. |
| `crates/meshlet-catalog/Cargo.toml` | Deterministic static meshlet catalog package and dependency boundary. |
| `crates/meshlet-catalog/src/lib.rs` | Eight-archetype, three-LOD geometry generation, meshlet partitioning, validation, encoding, and hashing. |
| `crates/meshlet-catalog/tests/catalog.rs` | Catalog determinism, reducing-LOD, and mesh-shader bound regression contract. |
| `crates/animation-catalog/Cargo.toml` | Deterministic skeletal fixture package and dependency boundary. |
| `crates/animation-catalog/src/lib.rs` | Animation catalog encoding, validation, hashing, and CPU pose evaluation contract. |
| `crates/animation-catalog/src/affine.rs` | Explicit row-major affine composition, transforms, encoding, and pose variation. |
| `crates/animation-catalog/src/builder.rs` | Deterministic hierarchy, bind data, clip samples, and packed skin-stream generation. |
| `crates/animation-catalog/tests/catalog.rs` | Hierarchy, skin influence, pose evaluation, and deterministic catalog regressions. |
| `crates/surface-catalog/Cargo.toml` | Deterministic surface fixture package and dependency boundary. |
| `crates/surface-catalog/src/lib.rs` | Normal/UV stream, expanded primitive map, generated material texture array, validation, encoding, and hashing. |
| `crates/surface-catalog/tests/catalog.rs` | Surface bounds, complete mip layout, deterministic encoding, and hash regressions. |
| `crates/terrain-format/Cargo.toml` | Canonical fixed terrain-pack package boundary and reusable dependencies. |
| `crates/terrain-format/src/lib.rs` | Terrain format V1/V2 module boundary and shared fixed-payload contract. |
| `crates/terrain-format/src/global.rs` | Signed-key V2 pack codec, source namespace, exact lookup, checksum, and edge validation. |
| `crates/terrain-format/src/payload.rs` | Fixed 4 KiB terrain payload validation, encoding, decoding, and zero-padding contract. |
| `crates/terrain-format/tests/pack.rs` | Terrain round-trip, malformed pack, padding, checksum, and shared-edge rejection contract. |
| `crates/terrain-format/tests/global_pack.rs` | V2 determinism, signed edge, exact-key binding, corruption, padding, and offset rejection contract. |
| `crates/region-format/Cargo.toml` | Canonical region-format package boundary and reusable dependencies. |
| `crates/region-format/src/lib.rs` | Versioned pack writer/reader, explicit record codec, index validation, and chunk verification. |
| `crates/region-format/tests/pack.rs` | Canonical round-trip and malformed metadata/payload rejection contract. |
| `tools/region-cooker/Cargo.toml` | Offline deterministic region-cooker package boundary. |
| `tools/region-cooker/src/main.rs` | Canonical sparse Experiment 0008 pack generation and manifest output. |
| `tools/terrain-cooker/Cargo.toml` | Offline deterministic terrain-cooker package boundary. |
| `tools/terrain-cooker/src/main.rs` | Deterministic local V1 or signed-global V2 terrain generation, edge proof, pack writing, and manifest output. |
| `apps/workbench/Cargo.toml` | Native workbench package and Windows API feature boundary. |
| `apps/workbench/build.rs` | Workbench Agility SDK staging and pinned DXC shader compilation. |
| `apps/workbench/shaders/calibration.hlsl` | Camera-relative calibration rasterization, stable scene-local material semantics, color, and object-ID shader. |
| `apps/workbench/shaders/region_load.hlsl` | Procedural region reset, cull/compact, indirect draw, and semantic-ID shaders. |
| `apps/workbench/shaders/resident_load.hlsl` | Persistent instance compaction, indirect rendering, and semantic-ID shaders. |
| `apps/workbench/shaders/async_resident.hlsl` | Descriptor-indexed per-slot compaction, canonical region-local projection, indirect rendering, and semantic-ID shaders. |
| `apps/workbench/shaders/meshlet_scene.hlsl` | GPU object culling, LOD, visible compaction, amplification, mesh emission, and semantic-ID shaders. |
| `apps/workbench/shaders/skeletal_scene.hlsl` | GPU canonical projection, stable-key animation, exact grounding, pose evaluation, meshlet skinning, and semantic-ID shaders. |
| `apps/workbench/shaders/surface_resolve.hlsl` | Deterministic visibility winner, compact payload emission, skeletal surface reconstruction, material resolve, and samples. |
| `apps/workbench/shaders/occlusion.hlsl` | Conservative hierarchy query, fixed classify/prefix/stable-scatter compaction, and reverse-Z mip construction. |
| `apps/workbench/shaders/terrain.hlsl` | Fixed region/patch seam oracles, GPU patch LOD, exact transition projection, mesh expansion, material color, and semantic-ID emission. |
| `apps/workbench/src/main.rs` | Workbench composition, Win32/frame loop, pending frame operations, and error propagation. |
| `apps/workbench/src/capture.rs` | Color/object-ID artifacts, encoding, hashes, manifests, and capture ownership. |
| `apps/workbench/src/inspect/mod.rs` | Workbench control-plane module boundary and narrow exports. |
| `apps/workbench/src/inspect/server.rs` | Project-owned SidecarRuntime transport, event framing, and response delivery. |
| `apps/workbench/src/inspect/protocol.rs` | Typed workbench control vocabulary, payload decoding, and protocol errors. |
| `apps/workbench/src/inspect/app.rs` | Main-thread control dispatch, pending frame operations, and established stream transaction entrypoints. |
| `apps/workbench/src/inspect/status.rs` | Workbench, renderer capability, and active workload status projection. |
| `apps/workbench/src/inspect/composition_control.rs` | Typed atomic composition schedule, fixture, mode, traversal, pass-order parsing, and control dispatch. |
| `apps/workbench/src/inspect/surface_control.rs` | Typed surface, material, mip, occlusion history, and probe control dispatch. |
| `apps/workbench/src/inspect/terrain_control.rs` | Typed terrain pack, local/global schedule, mode, and I/O/copy gate control dispatch. |
| `apps/workbench/src/inspect/world_control.rs` | Typed calibration-only global anchor, render-origin rebase, reset, status, and probe dispatch. |
| `apps/workbench/src/load.rs` | Region address space, load configuration, workload counts, and procedural semantics. |
| `apps/workbench/src/resident.rs` | Resident cache planning, local/canonical deterministic records, LRU eviction, and stream reports. |
| `apps/workbench/src/streaming/mod.rs` | Workbench streaming ownership boundary and narrow module exports. |
| `apps/workbench/src/streaming/async_resident.rs` | Protected 50-slot local/global/canonical keys, source-aware region-local payloads, stable seeds, reservations, and reports. |
| `apps/workbench/src/streaming/async_resident/canonical.rs` | Canonical object source namespace, signed-region cache key construction, stable seed, and serialization owner. |
| `apps/workbench/src/streaming/cooked/mod.rs` | Cooked pack controller, bounded transaction status, gate, and failure rollback evidence. |
| `apps/workbench/src/streaming/cooked/worker.rs` | Single background pack reader, bounded channels, chunk verification, and I/O metrics. |
| `apps/workbench/src/streaming/terrain/mod.rs` | V1/V2 terrain pack controller, source identity, bounded transaction status, I/O gate, and rollback evidence. |
| `apps/workbench/src/streaming/address.rs` | Shared signed global windows, checked format-V1 local alias mapping, and exact boundary tests. |
| `apps/workbench/src/streaming/terrain/worker.rs` | Single background terrain reader, exact signed-key lookup, bounded channels, payload verification, and I/O metrics. |
| `apps/workbench/src/perception.rs` | Pixel-region validation, ID analysis, semantic joins, samples, and diagnostic colors. |
| `apps/workbench/src/scene/mod.rs` | Calibration objects, scene state, camera-relative matrices, world controls, probes, and spatial manifest. |
| `apps/workbench/src/scene/camera.rs` | Validated scene camera value, default pose, region translation, and view-projection owner. |
| `apps/workbench/src/window.rs` | Win32 window class, native handle, and console shutdown lifecycle. |
| `apps/workbench/src/world.rs` | Signed region coordinates, split-position normalization, bounded render conversion, transactions, and exact oracle. |
| `apps/workbench/src/rendering/mod.rs` | Workbench rendering subsystem boundary and narrow application exports. |
| `apps/workbench/src/rendering/renderer/mod.rs` | D3D12 device, swap-chain resources, capabilities, and GPU synchronization ownership. |
| `apps/workbench/src/rendering/renderer/frame.rs` | Per-frame camera traversal observation, standalone/composed dispatch, capture, present, and probe submission path. |
| `apps/workbench/src/rendering/renderer/modes.rs` | Standalone/composed mode transitions and explicit isolation of canonical generated-object snapshots. |
| `apps/workbench/src/rendering/composition/mod.rs` | Source-aware terrain/object scheduling, atomic frame publication, staged validation, and rollback orchestration. |
| `apps/workbench/src/rendering/composition/global.rs` | Shared signed window expansion, local content projection, and global pair mapping evidence. |
| `apps/workbench/src/rendering/composition/contact.rs` | Requested-only exact selected-LOD surface and full-resolution grounding residual oracle. |
| `apps/workbench/src/rendering/composition/fixture.rs` | Legacy/canonical instance materialization, signed modular Q8 placement, stable seeds, and exact terrain sampling. |
| `apps/workbench/src/rendering/composition/probe.rs` | Canonical object/terrain semantic joins, exact grounding, pair mapping, shared submission, and timing evidence. |
| `apps/workbench/src/rendering/composition/schedule.rs` | Local/global/canonical pair reservation, source selection, stream submission, cancellation, and schedule response owner. |
| `apps/workbench/src/rendering/composition/state.rs` | Source-bound pair coordinator, transaction state changes, and composition/traversal status projection. |
| `apps/workbench/src/rendering/composition/traversal.rs` | Local/signed camera mapping, V1 frozen or V2 dynamic basis, single desired slot, blocked failure, and automatic pair scheduling. |
| `apps/workbench/src/rendering/composition/traversal/control.rs` | Renderer-facing traversal/prefetch enablement, pending promotion, and automatic pair dispatch. |
| `apps/workbench/src/rendering/composition/traversal/rollover.rs` | Canonical safe-band target derivation, commit event, cumulative delta, and pending camera translation owner. |
| `apps/workbench/src/rendering/composition/traversal/prefetch.rs` | Canonical motion lookahead, prepared target, promotion, failure isolation, and status evidence. |
| `apps/workbench/tests/private/composition_traversal.rs` | Private frozen-origin compatibility and canonical rollover policy regressions. |
| `apps/workbench/tests/private/scene.rs` | Private camera-relative calibration transform regression. |
| `apps/workbench/src/rendering/device.rs` | Reference adapter selection, debug-layer enablement, and common transitions. |
| `apps/workbench/src/rendering/gpu_capture.rs` | D3D12 copy footprint, persistent readback, and tight four-byte pixel extraction. |
| `apps/workbench/src/rendering/load/pipeline.rs` | Procedural load root signatures, PSOs, and indirect command signature. |
| `apps/workbench/src/rendering/load/renderer.rs` | Procedural GPU compaction, indirect recording, timestamp probes, and readback. |
| `apps/workbench/src/rendering/load/mod.rs` | Procedural load rendering ownership boundary and narrow exports. |
| `apps/workbench/src/rendering/resident/pipeline.rs` | Synchronous resident root signatures, PSOs, and indirect command signature. |
| `apps/workbench/src/rendering/resident/renderer.rs` | Synchronous resident state, command recording, completion, and GPU probes. |
| `apps/workbench/src/rendering/resident/resources.rs` | Shared resident resources, stream copies, barriers, viewport, and readback helpers. |
| `apps/workbench/src/rendering/resident/mod.rs` | Synchronous resident rendering ownership boundary and shared exports. |
| `apps/workbench/src/rendering/async_resident/pipeline.rs` | Descriptor-table asynchronous resident compute and graphics pipelines. |
| `apps/workbench/src/rendering/async_resident/renderer.rs` | Immutable local/global/canonical snapshot publication, rendering, and GPU probes. |
| `apps/workbench/src/rendering/async_resident/renderer/global.rs` | Legacy and source-aware canonical generated-object reservation and global-window projection. |
| `apps/workbench/src/rendering/async_resident/renderer/status.rs` | Async renderer mode, gate, snapshot, descriptor, protected-slot, and shutdown controls. |
| `apps/workbench/src/rendering/async_resident/transfer.rs` | Source-bound copy queue reservations, fences, gate, upload arena, slot states, and transaction lifecycle. |
| `apps/workbench/src/rendering/async_resident/transfer/lifecycle.rs` | Async gate release, copy-fence idle wait, event cleanup, and drop lifecycle. |
| `apps/workbench/src/rendering/async_resident/transfer/status.rs` | Asynchronous reservation, copy, gate, and publication status projection. |
| `apps/workbench/src/rendering/async_resident/resources.rs` | Asynchronous region descriptor heap and per-slot SRV construction. |
| `apps/workbench/src/rendering/async_resident/mod.rs` | Asynchronous resident rendering ownership boundary and narrow export. |
| `apps/workbench/src/rendering/meshlet_scene/pipeline.rs` | Meshlet compute/graphics root signatures, PSOs, and indirect mesh command signature. |
| `apps/workbench/src/rendering/meshlet_scene/resources.rs` | Immutable catalog upload, bounded execution buffers, counters, timestamps, and readback. |
| `apps/workbench/src/rendering/meshlet_scene/renderer.rs` | Meshlet mode configuration, command recording, GPU probe decoding, and status. |
| `apps/workbench/src/rendering/meshlet_scene/oracle.rs` | Deterministic CPU workload oracle for GPU aggregate validation. |
| `apps/workbench/src/rendering/meshlet_scene/mod.rs` | GPU meshlet scene ownership boundary and narrow exports. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/mod.rs` | Skeletal crowd rendering boundary and narrow exports. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/oracle.rs` | Deterministic legacy/canonical projected CPU aggregate oracle for skeletal workload validation. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/pipeline.rs` | Skeletal compute and mesh root signature, PSOs, and indirect command signatures. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/probe.rs` | Skeletal counters, timestamp decoding, palette samples, and oracle comparison. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/buffers.rs` | Common skeletal UAV and readback buffer allocation policy. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/renderer.rs` | Skeletal mode controls, shared camera/object projection, fixed recording, and resource transitions. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/report.rs` | Skeletal status, probe readback, optional grounded oracle input, and settings projection. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/resources.rs` | Animation uploads, bounded pose/palette resources, descriptors, queries, and readbacks. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface_bridge.rs` | Narrow skeletal resource and command-recording bridge consumed by surface resolve. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/mod.rs` | Surface visibility and resolve ownership boundary with narrow exports. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/occlusion/mod.rs` | Bounded hierarchy, filtered-list, counters, stable-prefix, and probe-readback resource ownership. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/occlusion/oracle.rs` | Exhaustive fixture-bound proof and CPU conservative-query aggregate oracle. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/occlusion/probe.rs` | Hierarchy, candidate mask, stable order, aggregate, and CPU-oracle evidence decoding. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/oracle.rs` | CPU reconstruction of payloads, skinning, surface attributes, material texels, and laboratory lighting. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/pipeline.rs` | Narrow surface root signatures, visibility mesh PSO, resolve PSO, and indirect command signature. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/probe.rs` | Surface counters, samples, hashes, timing decode, and oracle comparison. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/renderer.rs` | Surface mode controls, visibility and resolve recording, transitions, capture, and status. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/renderer/recording.rs` | Occlusion, indirect visibility, resolve, hierarchy, probe-copy, and state-transition command recording. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/resources/mod.rs` | Surface catalog, candidate map, statistics, sample, timestamp, and readback resource ownership. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/resources/descriptors.rs` | Surface SRV/UAV descriptor layout and shader-visible heap construction. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/resources/targets.rs` | Visibility, deterministic winner, resolved color, depth, and semantic target ownership. |
| `apps/workbench/src/rendering/meshlet_scene/skeletal/surface/resources/upload.rs` | Immutable surface buffer and texture-array upload helpers. |
| `apps/workbench/src/rendering/terrain/mod.rs` | Terrain renderer, immutable snapshot, V2 projection derivation, staging/commit, and fixed mesh/LOD validation recording. |
| `apps/workbench/src/rendering/terrain/cache.rs` | Protected terrain LRU with separate legacy and signed-global/content-binding cache identities. |
| `apps/workbench/src/rendering/terrain/control.rs` | Renderer-owned terrain streamer completion, scheduling, publication, and mode controls. |
| `apps/workbench/src/rendering/terrain/copy_timing.rs` | Copy-queue timestamp heap, bounded readback, frequency, and GPU duration decoding. |
| `apps/workbench/src/rendering/terrain/descriptors.rs` | Raw per-slot terrain SRVs and region/LOD statistics UAV descriptor heap construction. |
| `apps/workbench/src/rendering/terrain/lod.rs` | Independent CPU patch LOD, projected camera/region geometry, rational-edge, hashing, validation, and regression oracle. |
| `apps/workbench/src/rendering/terrain/pipeline.rs` | Terrain region/LOD compute and mesh root signature, PSOs, and shader contract. |
| `apps/workbench/src/rendering/terrain/probe.rs` | Local/global mapping, canonical semantic inversion, geometry, edge, resource, hash, and timing oracle projection. |
| `apps/workbench/src/rendering/terrain/projection.rs` | Shared V1 passthrough or V2 centered camera, terrain/object position, LOD, and semantic projection. |
| `apps/workbench/src/rendering/terrain/state.rs` | Terrain mode, LOD settings, published-state projection, descriptors, gates, and idle controls. |
| `apps/workbench/src/rendering/terrain/transfer.rs` | Dedicated copy queue, protected slots, upload arena, gates, fences, and frame publication. |
| `apps/workbench/src/rendering/cooked.rs` | Cooked I/O completion, reservation cancellation, and GPU submission orchestration. |
| `apps/workbench/src/rendering/calibration/object_id_target.rs` | Persistent `R32_UINT` semantic render-target resource and descriptor ownership. |
| `apps/workbench/src/rendering/calibration/scene_renderer.rs` | Calibration graphics PSO, reverse-Z depth, procedural geometry, and scene draws. |
| `apps/workbench/src/rendering/calibration/mod.rs` | Calibration rendering ownership boundary and narrow scene-renderer export. |
| `runseal.toml` | Explicit local resources, Deno policy, and repository environment injection. |
| `flavor.toml` | Consumer-owned code-shape scan scope and rule adjustments. |
| `sidecar.toml` | Local runtime identity, native workbench app target, readiness, and inspect endpoint. |
| `sidecar.benchmark.toml` | Release workbench identity and isolated benchmark Sidecar namespace. |
| `.runseal/deno.json` | Deno compiler and formatter policy for repository wrappers. |
| `.runseal/deno.lock` | Frozen Deno dependency resolution for repository wrappers. |
| `.runseal/hooks/pre-commit` | Git pre-commit entrypoint delegating to `runseal :guard`. |
| `.runseal/wrappers/init.ts` | Stable tool validation and repository hook installation. |
| `.runseal/wrappers/guard.ts` | Canonical Rust, Flavor, and Sidecar validation workflow. |
| `.runseal/wrappers/gpu-lab.ts` | Canonical Experiment 0001 bootstrap and execution workflow. |
| `.runseal/wrappers/object-id.ts` | Canonical Experiment 0004 object-ID perception and cleanup workflow. |
| `.runseal/wrappers/region-load.ts` | Canonical Experiment 0005 region scaling distributions and visual regression workflow. |
| `.runseal/wrappers/resident-stream.ts` | Canonical Experiment 0006 movement, eviction, restart, and resident evidence workflow. |
| `.runseal/wrappers/async-region.ts` | Canonical Experiment 0007 held-copy, publication, eviction, restart, and evidence workflow. |
| `.runseal/wrappers/cooked-region.ts` | Canonical Experiment 0008 recook, held-I/O, incremental reads, corruption, restart, and evidence workflow. |
| `.runseal/wrappers/meshlet-scene.ts` | Canonical Experiment 0009 meshlet catalog, GPU oracle, visual, sweep, movement, and restart evidence workflow. |
| `.runseal/wrappers/terrain-lod.ts` | Canonical Experiment 0014 GPU terrain LOD, exact stitch oracle, sweep, movement, restart, and timing workflow. |
| `.runseal/wrappers/composition.ts` | Canonical Experiment 0015 atomic publication, exact grounding, shared attachment, order, failure, movement, restart, and timing workflow. |
| `.runseal/wrappers/terrain-sampling.ts` | Canonical Experiment 0016 arbitrary-position triangle sampling, boundary, movement, restart, compatibility, and timing workflow. |
| `.runseal/wrappers/lod-composition.ts` | Canonical Experiment 0017 exact-ground, bounded-contact terrain LOD composition workflow. |
| `.runseal/wrappers/region-traversal.ts` | Canonical Experiment 0018 boundary, clamp, corridor, held latest-wins, failure, teleport, restart, and timing workflow. |
| `.runseal/wrappers/global-space.ts` | Canonical Experiment 0019 far-anchor, rebase, exact oracle, rejection, restart, compatibility, and timing workflow. |
| `.runseal/wrappers/global-terrain.ts` | Canonical Experiment 0020 signed terrain alias, movement, hold, rejection, restart, compatibility, and timing workflow. |
| `.runseal/wrappers/global-composition.ts` | Canonical Experiment 0021 signed pair alias, movement, three-hold, rejection, restart, compatibility, and timing workflow. |
| `.runseal/wrappers/global-traversal.ts` | Canonical Experiment 0022 far boundary, latest-wins, blocked failure, restart, compatibility, and timing workflow. |
| `.runseal/wrappers/signed-terrain-storage.ts` | Canonical Experiment 0023 V2 recook, alias rebind, source switch, rollback, restart, and timing workflow. |
| `.runseal/wrappers/camera-relative-terrain.ts` | Canonical Experiment 0024 alias-extreme projection, LOD, hold, restart, and timing workflow. |
| `.runseal/wrappers/canonical-object-composition.ts` | Canonical Experiment 0025 generated-object identity, V2 composition, alias, source, hold, failure, restart, and timing workflow. |
| `.runseal/wrappers/canonical-origin-rollover.ts` | Canonical Experiment 0026 normalization, axis rollover, hold, failure, restart, and release sweep workflow. |
| `.runseal/wrappers/canonical-traversal-prefetch.ts` | Canonical Experiment 0027 compatibility, lookahead, promotion, stale-work, failure, rollover, and release workflow. |
| `.runseal/support/cooked-region.ts` | Experiment 0008 structured evidence, pack corruption, hashing, and comparison helpers. |
| `.runseal/support/composition.ts` | Experiments 0015-0018 stable composition, grounding, contact, LOD, and timing validation support. |
| `.runseal/support/global-terrain.ts` | Experiment 0020 Sidecar lifecycle, global/local mapping, transaction, capture, and distribution validation helpers. |
| `.runseal/support/global-composition.ts` | Experiment 0021 shared pair mapping, half-report, hold, probe, and attachment validation helpers. |
| `.runseal/support/global-traversal.ts` | Experiment 0022 frozen-origin target mapping, traversal status, camera movement, and publication helpers. |
| `.runseal/support/signed-terrain-storage.ts` | Experiment 0023 V2 cooking, canonical-content, semantic-join, corruption, and failure helpers. |
| `.runseal/support/camera-relative-terrain.ts` | Experiment 0024 alias camera, canonical projection, semantic inversion, frame, and hold helpers. |
| `.runseal/support/canonical-object-composition.ts` | Experiment 0025 object identity, semantic join, complete-frame, pair movement, hold, and failure helpers. |
| `.runseal/support/canonical-origin-rollover.ts` | Experiment 0026 dynamic-basis status, exact target, camera, event, and publication validation helpers. |
| `.runseal/support/canonical-origin-rollover-evidence.ts` | Experiment 0026 pack fixture, retained stable-seed overlap, capability, and timing evidence helpers. |
| `.runseal/support/canonical-origin-rollover-scenarios.ts` | Experiment 0026 normalization, boundary, held latest-wins, failure, and disable/catch-up scenario owner. |
| `.runseal/support/canonical-traversal-prefetch.ts` | Experiment 0027 prefetch setup, status, count, target, and completion evidence helpers. |
| `.runseal/support/canonical-traversal-prefetch-evidence.ts` | Experiment 0027 control/prepared release sweeps, timing distributions, probes, and captures. |
| `.runseal/support/canonical-traversal-prefetch-scenarios.ts` | Experiment 0027 direction, promotion, stale-work, failure, rollover, and disable scenario owner. |
| `.runseal/support/traversal.ts` | Experiment 0018 bounded status, region mapping, and logical revisit evidence helpers. |
| `.runseal/support/workbench/composition.ts` | Local/global composition workbench CLI validation and typed Sidecar dispatch. |
| `.runseal/support/workbench/terrain.ts` | Terrain-specific workbench CLI argument validation and typed Sidecar event dispatch. |
| `.runseal/support/workbench/world.ts` | Signed world-coordinate workbench CLI validation and typed Sidecar event dispatch. |
| `.runseal/wrappers/skeletal-crowds.ts` | Canonical Experiment 0010 debug correctness, release timing, sweep, visual, movement, and restart workflow. |
| `.runseal/support/skeletal-crowds.ts` | Experiment 0010 structured validation, environment capture, fixtures, and distribution helpers. |
| `.runseal/wrappers/surface-resolve.ts` | Canonical Experiment 0011 debug correctness, release timing, surface sweeps, determinism, movement, and restart workflow. |
| `.runseal/support/surface-resolve.ts` | Experiment 0011 payload, sample oracle, artifact, environment, and distribution validation helpers. |
| `.runseal/wrappers/occlusion.ts` | Canonical Experiment 0012 hierarchy, invalidation, stable compaction, sweep, timing, movement, and restart workflow. |
| `.runseal/support/occlusion.ts` | Experiment 0012 fixed submission, hierarchy, order, oracle, resource, and evidence validation helpers. |
| `.runseal/wrappers/terrain.ts` | Canonical Experiment 0013 cook, seam, radius, boundary, movement, hold, corruption, restart, and timing workflow. |
| `.runseal/support/terrain.ts` | Experiments 0013-0014 stable probes, canonical hashes, captures, resources, and region/LOD seam validation helpers. |
| `.runseal/wrappers/visual-loop.ts` | Canonical Experiment 0002 deterministic capture and cleanup workflow. |
| `.runseal/wrappers/spatial-scene.ts` | Canonical Experiment 0003 spatial rendering and inspection workflow. |
| `.runseal/wrappers/workbench.ts` | Canonical workbench lifecycle and typed inspect workflow. |

## 5. Core Operational Workflows

### 5.1 Cold start

The R0 repository baseline is defined by the core files indexed above. R1 accepted a
Rust-based native D3D12 GPU laboratory on the single reference platform recorded in ADR
0001. ADR 0003 accepts the first operator-visible workbench cold start. Experiment 0002
and ADRs 0004-0005 accept deterministic renderer-owned frame artifacts. Experiment 0003
and ADR 0006 accept the calibration scene's spatial and depth vocabulary. Experiment
0004 and ADR 0007 accept deterministic object-ID and bounded screen-region perception.
Experiment 0005 and ADR 0008 accept region-addressed GPU compaction and bounded indirect
submission independent of total logical world extent. Experiment 0006 and ADR 0009
accept bounded default-heap region residency, active-slot indirection, incremental
uploads, deterministic eviction, and transactional cache publication. Experiment 0007
and ADR 0010 accept dedicated copy-queue transfer, immutable frame-boundary publication,
protected active slots, and explicit bounded backpressure. Experiment 0008 and ADR 0011
accept a versioned canonical region pack, offline-only writing, indexed on-demand chunk
validation, one bounded background worker, cache reservation before materialization, and
pre-copy rollback on I/O failure.
Experiment 0009 and ADR 0012 accept a deterministic bounded meshlet catalog, GPU object
culling and LOD selection, amplification and mesh shader execution, exact validation
against a CPU oracle, and one indirect mesh dispatch whose CPU submission shape is
independent of logical-world extent and emitted geometry count.
Experiment 0010 and ADR 0013 accept GPU animated-object classification, shared and unique
pose compaction, bounded 128-bone hierarchy evaluation, four-weight meshlet skinning,
and a fixed five-stage submission independent of visible character, pose, bone, and
geometry counts.
Experiment 0011 and ADR 0014 accept candidate-addressed compact visibility, a
rasterizer-ordered deterministic equal-depth winner, exact skinned surface
reconstruction, and one fixed-screen resolve dispatch independent of geometry, pose,
material, and occupancy counts.
Experiment 0012 and ADR 0015 accept a prior-compatible reverse-Z minimum hierarchy,
exhaustively proven fixture bounds, full-signature history invalidation, fixed
100/1/100-group stable GPU compaction, and one filtered indirect visibility dispatch.
This accepts exact work elimination; ROV-path total timing is not a promoted performance
claim.
Experiment 0013 and ADR 0016 accept an independent fixed 4 KiB terrain payload, global
integer-lattice same-resolution continuity, bounded background I/O and protected copy
publication, exact CPU/GPU shared-edge validation, terrain semantic perception, and one
fixed 400-group mesh dispatch. That experiment did not accept terrain LOD,
cross-resolution stitching, composition, grounding, collision, or committed-per-region
allocation as a final policy.
Experiment 0014 and ADR 0017 accept three GPU-selected patch resolutions, exact
coarse-edge clip-space projection, bounded CPU/GPU rational-edge validation, and fixed
submission independent of selected distribution and emitted geometry. This accepts
cross-resolution geometric continuity and work elimination, not lower GPU time,
material/normal continuity, a final LOD policy, composition, grounding, or collision.
Experiment 0015 and ADR 0018 accept one matched terrain/object publication token,
transactional two-sided staging and rollback, deliberately distinct physical region
mappings, exact 25,600-value integer GPU grounding against a CPU oracle, and terrain
plus skeletal rasterization into one reverse-Z depth and object-ID attachment set. The
accepted output is byte-identical for terrain-first and object-first execution. This
does not accept arbitrary-position interpolation, slope alignment, terrain LOD in the
composed path, collision, or a general scene graph.
Experiment 0016 and ADR 0019 accept deterministic 1/512-meter arbitrary XZ positions,
exact Q16 interpolation over the emitted terrain triangles, requested GPU/CPU oracle
agreement, and same-position continuity across all active region boundaries. The path
retains the accepted fixed submission and formats. It does not accept terrain LOD
composition, sampling outside the owning region, slope frames, collision, navigation,
or a general scene query.
Experiment 0017 and ADR 0020 accept terrain render LOD inside composition while exact
full-resolution ground remains camera-independent. A requested Q18 oracle bounds the
canonical visible contact approximation to 0.125 meter, and LOD adds one fixed terrain
dispatch without changing five-stage skeletal submission. This does not accept a
general error policy, authored terrain tolerance, geomorphing, slope frames, collision,
navigation, or reusable scene queries.
Experiment 0018 and ADR 0021 accept half-open camera-to-region ownership, an immutable
world/radius traversal basis, one in-flight pair plus one latest-wins desired config,
complete-old-snapshot rendering, and blocked failure without per-frame retry. This
accepts bounded atomic traversal, not predictive prefetch, latency hiding, hysteresis,
cancellation, floating origins, or a general streaming graph.
Experiment 0019 and ADR 0022 accept signed 64-bit CPU region coordinates, half-open
split positions, integer-before-float subtraction, an eight-region render bound,
transactional calibration world controls, camera-at-origin GPU transforms, and stable
scene-local material semantics. This accepts a coordinate/rendering boundary, not a
format/cache-key migration, signed streaming address space, automatic rebase policy,
network coordinates, or a general floating-origin system.
Experiment 0020 and ADR 0023 accept signed global terrain keys as logical cache identity,
checked mapping through a bounded local format-V1 content alias, exact overlap/revisit
behavior, and transactional publication of complete global/local assignments. The pack,
GPU placement, semantic ID, shader, and fixed submission contracts remain local and
unchanged.
Experiment 0021 and ADR 0024 promote that mapping into one shared streaming address
owner and add signed/content-bound generated-object cache identity. Terrain and object
halves reserve the same immutable window, validate complete staged assignments, and
publish one matched global/local pair while independent I/O/copy holds keep the old pair
visible. This accepts manual signed atomic composition for the deterministic object
fixture, not cooked-object global lookup, automatic traversal/rebase, or a new world
format.
Experiment 0022 and ADR 0025 allow the accepted camera mapping and one-in-flight plus
latest-wins policy to carry an optional signed target through both halves. The signed
origin is frozen per session, every legal local center is checked for `i64` range at
enable, and automatic publication remains one matched global/local pair. This accepts
camera traversal inside one explicit signed alias window, not automatic rebase,
prefetch, cooked-object global lookup, or an unbounded world.
Experiment 0023 and ADR 0026 accept terrain pack V2 with exact signed keys, one
content-derived source namespace, and cache identity by namespace plus global region.
Local IDs remain bounded placement/semantic projections, so alias rebinding preserves
canonical slots with zero I/O while different sources cannot false-hit. This accepts
signed terrain storage and residency, not V2 composition, automatic rebase,
camera-relative terrain transforms, or global semantic indirection.
Experiment 0024 and ADR 0027 accept one V2 projection derived from committed source
identity: the CPU translates camera and LOD into a fixed centered window while the
existing GPU mapping consumes bounded projected region IDs. Frame-local object IDs
invert exactly through a signed CPU table, so every legal local alias produces identical
terrain attachments without changing HLSL or root constants. This removes the terrain
side of the origin-rollover blocker, not generated-object projection, V2 atomic
composition, persistent object identity, or automatic rebase.
Experiment 0025 and ADR 0028 accept canonical procedural object identity by object source
plus signed region, alias-independent region-local payloads and stable seeds, one shared
terrain/object projection, and atomic V2 composition. Fixed-window aliases retain both
caches with zero transfer, terrain namespace changes do not invalidate generated
objects, and three independent holds preserve the complete old pair. This removes the
generated-object and V2 composition blockers for origin rollover, but does not accept
automatic rollover policy, authored objects, or persistent public object identity.
Experiment 0026 and ADR 0029 accept canonical-only safe-band origin rollover. Traversal
derives exact signed targets before choosing a bounded basis, then publishes the matched
terrain/object pair, basis, and opposite camera translation at one commit boundary.
Resident canonical entries survive rebinding, three independent holds expose only the
old coordinate frame, and failures cannot mutate basis or camera. This accepts bounded
continuous coordinate rollover, not prefetch, authored objects, persistent public IDs,
collision, navigation, networking, or a general floating-origin framework.
Experiment 0027 and ADR 0030 accept one adjacent canonical traversal prefetch. Camera
motion inside a fixed four-meter boundary band may run the existing matched transaction,
retain its immutable cache population, and discard speculative active mappings. A
completed target makes later demand 25/0; an in-flight exact target promotes without
early publication. Stale and failed work cannot block or publish demand, and the bound
remains one transaction plus one latest target. This does not accept general path
prediction, authored objects, collision, navigation, or networking.

The workbench is a composition root, not permission to create broad engine scaffolding.
Do not begin ECS, assets, or general graphics architecture until a numbered experiment
defines its hypothesis, workload, and criteria.

Canonical commands from the repository root:

```powershell
runseal :init
runseal :guard
runseal :gpu-lab correctness
runseal :gpu-lab benchmark
runseal :visual-loop
runseal :spatial-scene
runseal :object-id
runseal :region-load
runseal :resident-stream
runseal :async-region
runseal :cooked-region
runseal :meshlet-scene
runseal :skeletal-crowds
runseal :surface-resolve
runseal :occlusion
runseal :terrain
runseal :terrain-lod
runseal :composition
runseal :terrain-sampling
runseal :lod-composition
runseal :region-traversal
runseal :global-space
runseal :global-terrain
runseal :global-composition
runseal :global-traversal
runseal :signed-terrain-storage
runseal :camera-relative-terrain
runseal :canonical-object-composition
runseal :canonical-origin-rollover
runseal :canonical-traversal-prefetch
runseal :workbench start
runseal :workbench status
runseal :workbench inspect
runseal :workbench color 0.08 0.42 0.24
runseal :workbench pause
runseal :workbench capture operator-check
runseal :workbench perception operator-perception
runseal :workbench perception-region operator-region 560 240 160 200
runseal :workbench camera
runseal :workbench camera-set -9 5 10 0 1 -3 60
runseal :workbench camera-reset
runseal :workbench scene
runseal :workbench world
runseal :workbench world-relocate 1099511627776 -1099511627776
runseal :workbench world-rebase 1099511627780 -1099511627780
runseal :workbench world-probe
runseal :workbench world-reset
runseal :workbench load-config 128
runseal :workbench resident
runseal :workbench resident-stream 64 64
runseal :workbench async
runseal :workbench async-schedule 64 64
runseal :workbench async-gate-arm
runseal :workbench async-gate-release
runseal :workbench cooked
runseal :workbench cooked-open out/cooked/0008-cooked-region-io/regions-a.wlr
runseal :workbench cooked-schedule 64 64
runseal :workbench cooked-gate-arm
runseal :workbench cooked-gate-release
runseal :workbench meshlet
runseal :workbench meshlet-config 255
runseal :workbench meshlet-enable
runseal :workbench meshlet-disable
runseal :workbench surface
runseal :workbench surface-config 64 0
runseal :workbench surface-enable
runseal :workbench surface-disable
runseal :workbench occlusion-enable
runseal :workbench occlusion-disable
runseal :workbench occlusion-reset
runseal :workbench terrain
runseal :workbench terrain-open out/terrain/0013-gpu-streamed-terrain/terrain.wlt
runseal :workbench terrain-schedule 64 64 2
runseal :workbench terrain-global-schedule 1099511627776 -1099511627776 1099511627776 -1099511627776 2
runseal :workbench terrain-enable
runseal :workbench terrain-disable
runseal :workbench terrain-lod
runseal :workbench terrain-lod-config 2 6 auto
runseal :workbench terrain-lod-enable
runseal :workbench terrain-lod-disable
runseal :workbench terrain-io-gate-arm
runseal :workbench terrain-io-gate-release
runseal :workbench terrain-copy-gate-arm
runseal :workbench terrain-copy-gate-release
runseal :workbench composition
runseal :workbench composition-schedule 64 64
runseal :workbench composition-enable
runseal :workbench composition-disable
runseal :workbench composition-traversal-enable
runseal :workbench composition-traversal-disable
runseal :workbench composition-prefetch-enable
runseal :workbench composition-prefetch-disable
runseal :workbench composition-order terrain-first
runseal :workbench composition-fixture arbitrary-q8
runseal :workbench load-probe
runseal :workbench load-disable
runseal :workbench resume
runseal :workbench restart
runseal :workbench stop
```

Correctness mode requires the Windows optional capability
`Tools.Graphics.DirectX~~~~0.0.1.0`. Benchmark mode intentionally runs without the debug
layer and must report that validation is disabled.

`sidecar.toml` is the interactive debug-layer workbench contract.
`sidecar.benchmark.toml` is the release measurement contract and uses a separate Sidecar
namespace; canonical experiment wrappers must stop and verify both namespaces.

The wrappers use installed stable-channel Flavor, Runseal, and Sidecar CLIs. Sibling
source checkouts are references, not runtime dependencies. The workbench accepts the
canonical `--sidecar-stamp` argument and exposes only the typed events recorded in the
accepted workbench ADRs and experiments.

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
