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
- Keep temporary verification scripts dependency-free and self-contained. They must not import
  `.runseal/support`; reusable Runseal capability belongs behind an explicit maintained wrapper.
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

This section is the sole changing live capability ledger. The repository model owns stable
structure and dependency rules and must not duplicate a stage snapshot.

Experiments 0031-0093 and the current ADR set through 0096 define one live content runtime
with explicit object presentation authority, deterministic frame-driven presentation time,
one explicit deterministic simulation schedule, private fixed terrain-motion/translation/advance
contracts consumed by one retained runtime-actor lifecycle plus a sole transactional schedule/actor
advance, one prototype live host-time driver, one canonical translatable terrain
position, one offline-cooked external
geometry/material/rig source, and one deterministic object-shadow path:

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
  nanoseconds, independent from frames and presentation, with one prototype Ready-only driver;
- private pure terrain-body motion, bounded planar translation, and planar-first advance contracts
  with focused tests but no copied-value inspect command or public `Runtime` mutation method;
- one runtime-owned optional `RuntimeActor` with capacity one, checked nonzero generation handles,
  exact spawn/read/despawn semantics, exact schema-3 presentation, one bounded animation epoch in
  the existing presentation-tick domain, and one prototype consumer;
- one renderer-internal immutable actor render projection that maps the frame's copied actor through
  the enabled/pending published composition into bounded window-relative Q9/Q16 input without
  float global coordinates, a public projection transaction, or a second scene path;
- one 56-byte self-contained GPU visible record carrying grounded window position, authored height,
  semantic region, frame-resolved actor-local animation phase, pose, and exact two-word source
  identity; streamed instances and ground values are skeletal-cull inputs only and are not rebound
  downstream;
- one fixed actor candidate after the 25,600 streamed candidates, backed by one two-frame upload
  resource and consumed by the existing skeletal, surface, shadow, and occlusion path without a GPU
  copy or additional synchronization;
- one private 0..=8 terrain-body motion batch that executes only local motion, applies one checked
  caller-supplied vertical velocity delta before its first emitted step, retains only the final
  existing step's exact optional grounded witness, and preserves exact zero-step/single-tick/
  rollback tests without an independent live mutation route;
- one sole caller-supplied typed motion/presentation simulation command and actor transaction that
  validates presentation before work, prepares a schedule copy and local motion batch, preserves
  the complete actor and consumes no initial velocity delta on zero emitted steps, applies that
  delta exactly once before the first nonzero-batch step, resets the animation epoch only for a committed
  animated-state/rig/clip transition, preflights the complete nonzero-step candidate against
  the published and non-prefetch pending render windows when canonical composition is enabled,
  preserves published-window failure,
  and returns typed advanced or pending render-blocked outcomes; only advanced commits actor and
  schedule together while preserving the handle, a block reports prepared step/query work
  without mutation or a speculative contact witness, the transition reports `None` for zero steps
  or the final committed step's exact grounded result, Runtime owns no wall clock, and prototype is
  its first live bounded-elapsed caller;
- one signed-region/half-open-local-Q9 `TerrainPosition` shared by query/contact/motion, with exact
  checked positive, negative, and multi-region planar translation and no compatibility alias;
- one bounded 225-body contact transition witness in the generic canonical probe; the historical
  230,400-body checkpoint has no live inspect verb, runtime branch, or coverage mode;
- one host-owned Win32 keyboard/focus adapter and fixed normalized input state with exact held plus
  most-recent-ingest pressed/released sets, repeat/unmatched/invalid suppression, focus cleanup,
  and empty-ingest edge expiry without a journal or replay branch;
- one reference-host monotonic admission state machine that applies each ordered activation batch
  before exact bounded sampling, with prototype consumption, stall recovery, reset, and rollback;
- one concrete Win32 activation reducer that maps arbitrary focus-loss/resume bursts into at most
  two order-equivalent typed transitions without an event queue;
- one optional strict schema-2 bootstrap document that selects both sources, one signed global
  target, and one inclusive signed playable-region rectangle containing that target, hides async
  progress, and emits readiness only after a canonical frame;
- one concrete Windows reference-host owner for the single window/message lifecycle, fixed
  normalized held/edge input boundary, bootstrap parser, canonical-ready driver, and composed time
  policy;
- one mandatory-bootstrap, non-diagnostic prototype composition root over the same runtime, with
  one grounded imported-Fox actor, Ready-only fixed gravity plus fixed W/A/S/D integer locomotion
  with held Shift selecting exact 64/45-Q9 Run components instead of the retained 32/23-Q9 Walk,
  one exact current-candidate quarter-orbit integer rotation from local input into world XZ before
  playable-boundary admission,
  one capacity-one grounded Space Jump intent that selects a fixed 4,369-Q16 batch-entry velocity
  delta, observes activation/time discontinuity before current-batch action admission, retains only
  the action bit across fractional work, stalls, and pending-window backpressure, consumes it on the
  next committed nonzero actor transition, and updates eligibility from its exact final grounded
  witness,
  transactional Survey-while-stationary/Walk-or-imported-Run-while-moving clip selection plus exact committed
  eight-way locomotion facing that retains the last admitted yaw while stationary, and local
  phase-zero Survey spawn/Walk/Run transition over the renderer's sole presentation clock,
  a 0.5-meter step-up bound, independent maximum-eight-step per-axis reduction against the
  bootstrap-authored playable rectangle before the strict runtime transaction, one committed
  four-state Q/E quarter-orbit actor-relative camera policy applied before each frame, explicit
  no elapsed retry/backlog on render block, readiness after a nonzero commit/frame, one-time
  post-spawn composition traversal with prefetch disabled and compact status evidence, one top-level
  current actor authority equal to committed simulation
  output with no spawn-time terrain/actor compatibility snapshot, and the normalized Escape press
  edge limited to host exit;
- one accepted post-v0 finite-edge policy whose maintained operator declares inclusive `[-6,6]²`
  playable bounds inside cooked `[-8,8]²` centers, whose focused real process remains live under
  explicitly activated held locomotion, and which adds no engine boundary mode, source-index
  inference, compatibility decoder, product telemetry, or weakened runtime source/query failure;
- one accepted post-v0 host input-edge boundary that exposes sample-scoped `was_pressed` and
  `was_released` facts beside continuous `is_held`, expires them on empty ingest, and proves the
  first live consumer through a real Escape press and clean prototype exit without an action queue;
- one mandatory post-v0 cleanup that deletes the process-local input journal, status/hashes/replay,
  diagnostic native-post adapter, five inspect verbs, four wrapper commands, and long-report field;
  workbench retains no input state after bootstrap, while prototype preserves one fixed input owner
  across bootstrap for pre-ready held input, with a guard rejecting every retired live surface;
- one accepted post-v0 camera action whose prototype-owned four-state exact quarter-orbit policy
  consumes Q/E press edges, prepares a complete actor-relative rig, commits its index only after the
  sole checked runtime camera mutation succeeds, and drives the corresponding exact traversal desire
  through the existing bounded latest-wins state machine without an engine input/camera controller;
- one accepted post-v0 actor transaction input that checked-adds a required caller-owned vertical
  velocity delta once at nonzero fixed-step batch entry, consumes nothing on zero steps, preserves
  schedule/actor/render-admission rollback, and introduces no jump verb, retained intent, default,
  alias, independent mutation route, or prototype behavior;
- one accepted post-v0 actor contact witness that reports no value for zero fixed steps and the
  final existing planar-first step's exact grounded result only for a committed nonzero transition;
  failures and render-blocked candidates expose no witness, RuntimeActor stores no contact flag,
  and prototype acceptance consumes the exact value without adding action policy or another query;
- one accepted post-v0 prototype Jump policy that admits Space only from its last exact committed
  grounded witness, holds at most one intent across fractional work, stalls, and typed render
  backpressure without elapsed backlog, clears stale intent on Reset/Suspended before current-batch
  action admission, supplies the existing fixed vertical delta exactly once, consumes only on a
  committed nonzero transition, and adds no engine input/action state or jump presentation;
- one mandatory post-v0 cleanup that deletes the duplicate standalone simulation-schedule Runtime
  forwarder and inspect verb plus the recurring eight-request retired-control report; exact schedule
  state remains available only through the canonical aggregate/frame probe and per-transaction
  actor advance, with one current unknown-event witness and a stable removal guard;
- one accepted post-v0 Run modifier whose prototype derives a private gait fact only from held Shift
  plus final admitted nonzero W/A/S/D, selects exact fixed 64/45-Q9 displacement and the existing
  imported Run clip in the sole actor transaction, and adds no retained gait state, host/engine
  action state, acceleration, horizontal velocity, alternate movement path, or new asset;
- one accepted post-v0 camera-relative locomotion policy that uses the current pure Q/E camera
  candidate to quarter-rotate exact local Walk/Run into world XZ before boundary admission, authors
  facing from that final world command, and still commits orbit state only after the existing checked
  runtime camera mutation without another controller or cross-subsystem transaction;
- one accepted post-v0 exact CPU object authority that moves each verified schema-3 triple page into
  the existing source-addressed 50-slot cache, shares immutable active-page references through the
  same GPU copy completion and atomic pair publication, and exposes strict committed-snapshot lookup
  by signed region plus authored local ID with no query allocation, source I/O, GPU work, second
  scene, spatial selection, interaction policy, or persistent gameplay identity;
- one accepted plain Prototype v0 stage boundary over that exact self-contained finite single-actor
  loop; it does not claim sustained product traversal, a source service, finite-edge behavior,
  gameplay interaction, multiple actors, networking, or Wulin content;
- one exact read-only CPU terrain-height query over the committed snapshot, addressed by signed
  region plus half-open local Q9 and independent from camera, render LOD, source I/O, and GPU work;
- one caller-owned exact vertical terrain-body contact transaction with strict
  separated/touching/penetrating classification, minimum upward correction, and no runtime body
  mutation, gravity, or locomotion policy;
- one clear-only diagnostic idle shell with neutral reverse-Z depth and semantic frame targets,
  no calibration scene, and no split-world control surface;
- one compact `actor.*` / `simulation.actor.advance` / `camera.*` / `source.*` / `canonical.*`
  inspect vocabulary with no standalone simulation-schedule status alias;
- one non-recursive `runseal :canonical-prototype` host/application workflow, one non-recursive
  `runseal :canonical-actor` actor GPU workflow, one `runseal :canonical-frame`
  focused GPU regression workflow, one `runseal :canonical-resources` same-process plateau
  workflow, and one non-recursive `runseal :canonical-runtime` end-to-end acceptance workflow;
- one self-contained `runseal :prototype` manual operator that deterministically cooks a finite
  zero-origin 289-center/441-region sandbox, writes strict bootstrap, and delegates the existing
  non-diagnostic Sidecar lifecycle without an acceptance-artifact prerequisite.

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
| `sidecar.prototype.toml` | Underlying non-diagnostic configured prototype lifecycle. |
| `docs/architecture/repository-model.md` | Stable ownership/dependency rules and current-boundary authority pointer. |
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
| `docs/adr/0044-normalized-host-input-journal.md` | Superseded original host normalization and diagnostic journal decision. |
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
| `docs/adr/0058-retired-caller-owned-terrain-transactions.md` | Accepted removal of copied-value terrain mutation surfaces, retention of private transaction contracts, and typed canonical setup ownership. |
| `docs/adr/0059-transactional-retained-body-batch.md` | Accepted bounded retained batch execution, copy-once/commit-once rollback, and deferred schedule composition. |
| `docs/adr/0060-transactional-simulation-body-advance.md` | Accepted explicit elapsed schedule/body preparation, dual commit, partition equality, and complete rollback. |
| `docs/adr/0061-bounded-host-elapsed-clock.md` | Accepted exact bounded monotonic host sampling, explicit stall recovery, suspension reset, and rollback. |
| `docs/adr/0062-bounded-win32-activation.md` | Accepted bounded Win32 focus reduction, interrupted ordering, duplicate suppression, and reset. |
| `docs/adr/0063-retired-independent-simulation-controls.md` | Accepted removal of independent schedule/body mutation, live schedule probe, and redundant process gates. |
| `docs/adr/0064-composed-host-time-admission.md` | Accepted ordered activation-before-sample host time composition, candidate commit, and independent clock-control removal. |
| `docs/adr/0065-prototype-body-bootstrap.md` | Accepted prototype-owned grounded retained-body bootstrap, readiness evidence, and terminal failure ordering. |
| `docs/adr/0066-live-prototype-time-driver.md` | Accepted prototype activation/time/simulation/frame ordering, typed no-advance outcomes, and post-commit readiness. |
| `docs/adr/0067-retained-runtime-actor-authority.md` | Accepted direct actor motion/presentation ownership, generation lifecycle, and retained-body API retirement. |
| `docs/adr/0068-neutral-canonical-operator-identity.md` | Accepted neutral canonical report/collection ownership and stable history-label rejection. |
| `docs/adr/0069-bounded-actor-render-projection.md` | Accepted exact integer actor-to-window projection and deferred GPU binding boundary. |
| `docs/adr/0070-self-contained-visible-record.md` | Accepted self-contained grounded GPU visible record and downstream source-page isolation. |
| `docs/adr/0071-frame-safe-actor-gpu-admission.md` | Accepted fixed actor candidate, exact generation identity, frame-slotted upload, and single GPU path. |
| `docs/adr/0072-prototype-gravity-admission.md` | Accepted prototype-owned fixed gravity, Ready-only admission, and grounded stability contract. |
| `docs/adr/0073-retired-standalone-actor-projection.md` | Accepted removal of the standalone projection API/verb/gate and retention of one internal frame path. |
| `docs/adr/0074-actor-relative-camera-mutation.md` | Accepted generation-qualified actor-relative camera mutation, exact internal anchor derivation, and prototype rig ownership. |
| `docs/adr/0075-transactional-actor-render-admission.md` | Accepted pre-commit canonical actor preflight, dual rollback, and sole private projection authority. |
| `docs/adr/0076-typed-actor-render-backpressure.md` | Accepted pending-window-only typed backpressure, published/fatal error preservation, schema-2 outcome, and prototype no-backlog policy. |
| `docs/adr/0077-prototype-fixed-horizontal-locomotion.md` | Accepted fixed W/A/S/D command reduction, bounded step-up, real native-input evidence, and actor-relative camera following. |
| `docs/adr/0078-current-prototype-readiness-authority.md` | Accepted removal of stale spawn terrain/actor readiness payload and one committed current actor authority. |
| `docs/adr/0079-prototype-traversal-activation.md` | Accepted one-time post-spawn prototype traversal, exact first camera target, and compact existing-status evidence. |
| `docs/adr/0080-transactional-actor-presentation-command.md` | Accepted typed motion/presentation command, zero-step preservation, complete candidate commit, and prototype Survey/Walk policy. |
| `docs/adr/0081-committed-prototype-locomotion-facing.md` | Accepted exact eight-way Q16 facing and nonzero-advance committed policy state. |
| `docs/adr/0082-self-contained-prototype-operator.md` | Accepted deterministic finite-sandbox preparation and sole manual prototype wrapper. |
| `docs/adr/0083-live-documentation-authority.md` | Accepted single current-boundary authority and prototype-operator documentation decision. |
| `docs/adr/0084-actor-local-animation-epoch.md` | Accepted transactional actor animation epoch and frame-resolved GPU phase decision. |
| `docs/adr/0085-plain-prototype-v0-stage-boundary.md` | Accepted finite single-actor plain Prototype v0 stage boundary. |
| `docs/adr/0086-explicit-playable-region-boundary.md` | Accepted strict bootstrap rectangle and prototype-owned finite-edge policy. |
| `docs/adr/0087-normalized-host-input-edges.md` | Superseded sample-scoped normalized edge and first product action decision. |
| `docs/adr/0088-retired-diagnostic-host-input-journal.md` | Accepted fixed normalized input state and diagnostic journal retirement decision. |
| `docs/adr/0089-committed-prototype-camera-orbit.md` | Accepted application-owned candidate/commit policy for discrete actor-relative camera orbit. |
| `docs/adr/0090-transactional-actor-vertical-impulse.md` | Accepted batch-entry vertical velocity delta in the sole actor transaction. |
| `docs/adr/0091-committed-actor-grounded-witness.md` | Accepted exact final fixed-step grounded witness on committed actor transitions. |
| `docs/adr/0092-committed-prototype-jump-intent.md` | Accepted capacity-one Space intent and committed prototype jump policy. |
| `docs/adr/0093-retired-standalone-simulation-status.md` | Accepted retirement of the duplicate simulation status inspect chain. |
| `docs/adr/0094-committed-prototype-run-modifier.md` | Accepted stateless held-Shift Run displacement and imported-clip policy. |
| `docs/adr/0095-committed-camera-relative-locomotion.md` | Accepted exact current-camera-candidate quarter rotation of prototype locomotion. |
| `docs/adr/0096-exact-canonical-object-query.md` | Accepted bounded committed CPU object residency and exact authored-triple lookup. |
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
| `experiments/0055-mandatory-terrain-transaction-cleanup/README.md` | Accepted copied-value control-chain removal, typed setup extraction, default wrapper limit, and short live rejection evidence. |
| `experiments/0056-transactional-retained-body-batch/README.md` | Accepted 0..=8 retained batch, partition equality, mid-batch rollback, and time-independence evidence. |
| `experiments/0057-transactional-simulation-body-advance/README.md` | Accepted explicit elapsed dual commit, coarse/nominal equality, and schedule/body rollback evidence. |
| `experiments/0058-bounded-host-elapsed-clock/README.md` | Accepted bounded host elapsed outcomes, stall recovery, suspension reset, and deterministic replay evidence. |
| `experiments/0059-bounded-host-activation/README.md` | Accepted bounded Win32 activation batches, exhaustive burst reduction, reset, and replay evidence. |
| `experiments/0060-mandatory-simulation-control-cleanup/README.md` | Accepted independent simulation-control removal, recurring gate deletion, retired-verb rejection, and dual preservation evidence. |
| `experiments/0061-composed-host-time-admission/README.md` | Accepted activation-before-sample ordering, interruption reset, rollback, and deterministic replay evidence. |
| `experiments/0062-prototype-body-bootstrap/README.md` | Accepted post-publication grounded prototype body, restart equality, failure ordering, and lifecycle evidence. |
| `experiments/0063-live-prototype-time-driver/README.md` | Accepted Ready-only live schedule/body driving, zero-command stability, post-frame readiness, and lifecycle evidence. |
| `experiments/0064-retained-runtime-actor-authority/README.md` | Accepted capacity-one actor identity/motion/presentation authority, direct promotion, and process evidence. |
| `experiments/0065-mandatory-canonical-operator-cleanup/README.md` | Accepted removal of historical canonical operator naming, neutral evidence ownership, and stable guard. |
| `experiments/0066-bounded-actor-render-projection/README.md` | Accepted far-coordinate, seam, alias/rollover, edge, rejection, and replay evidence for one live actor projection. |
| `experiments/0067-self-contained-visible-record/README.md` | Accepted grounded visible-record ownership, exact frame replay, bounded resources, and lifecycle evidence. |
| `experiments/0068-frame-safe-actor-gpu-admission/README.md` | Accepted frame-safe actor admission, exact compaction identity, rollback, resource, and lifecycle evidence. |
| `experiments/0069-prototype-gravity-admission/README.md` | Accepted fixed prototype gravity, grounded actor stability, focused process restart, and lifecycle evidence. |
| `experiments/0070-mandatory-actor-projection-cleanup/README.md` | Accepted standalone projection surface removal, live old-verb rejection, and exact actor-frame preservation. |
| `experiments/0071-actor-relative-camera-anchor/README.md` | Accepted private-projection camera anchor, transactional scene mutation, and exact prototype frame-order evidence. |
| `experiments/0072-transactional-actor-render-admission/README.md` | Accepted shared-window candidate commit, pending-window dual rollback, retained frame, and unchanged GPU evidence. |
| `experiments/0073-typed-actor-render-backpressure/README.md` | Accepted typed blocked outcome, strict fatal-error preservation, retained rendering, and prototype consumption evidence. |
| `experiments/0074-prototype-horizontal-locomotion/README.md` | Accepted fixed horizontal input mapping, native-process movement, grounded actor, and following-camera evidence. |
| `experiments/0075-mandatory-prototype-readiness-cleanup/README.md` | Accepted stale prototype readiness snapshot removal, current-output authority, and compatibility-free schema replacement. |
| `experiments/0076-prototype-traversal-activation/README.md` | Accepted one-time prototype traversal activation, exact diagonal schedule, and no-prefetch process evidence. |
| `experiments/0077-transactional-locomotion-presentation/README.md` | Accepted atomic motion/presentation admission, fractional/block rollback, and prototype Survey/Walk evidence. |
| `experiments/0078-committed-locomotion-facing/README.md` | Accepted exact eight-way locomotion yaw, stationary retention, and native-W transactional evidence. |
| `experiments/0079-self-contained-prototype-operator/README.md` | Accepted source-free cold start, deterministic sandbox, and wrapper-owned Sidecar lifecycle evidence. |
| `experiments/0080-mandatory-live-documentation-authority-cleanup/README.md` | Accepted duplicate state-ledger removal and maintained prototype-operator documentation evidence. |
| `experiments/0081-actor-local-animation-epoch/README.md` | Accepted actor-local Survey/Walk phase origin, GPU resolution, rollback, and prototype evidence. |
| `experiments/0082-plain-prototype-v0-stage-seal/README.md` | Accepted source-free product, focused prototype, and long canonical Prototype v0 stage evidence. |
| `experiments/0083-explicit-playable-region-boundary/README.md` | Accepted schema, per-axis maximum-batch policy, real held-input survival, and operator evidence. |
| `experiments/0084-normalized-host-input-edges/README.md` | Accepted edge lifetime, journal isolation, native record preservation, and real Escape-exit evidence. |
| `experiments/0085-mandatory-host-input-journal-cleanup/README.md` | Accepted diagnostic journal/native-post/operator deletion and product-input preservation proof. |
| `experiments/0086-committed-prototype-camera-orbit/README.md` | Accepted Q/E edge, exact rig, commit ordering, and real-process camera-orbit evidence. |
| `experiments/0087-transactional-actor-vertical-impulse/README.md` | Accepted exactly-once batch-entry velocity delta and transaction rollback proof. |
| `experiments/0088-committed-actor-grounded-witness/README.md` | Accepted exact committed last-step grounded witness and blocked-candidate isolation proof. |
| `experiments/0089-committed-prototype-jump-intent/README.md` | Accepted grounded Space admission, bounded intent lifetime, and committed jump-consumption proof. |
| `experiments/0090-mandatory-simulation-status-cleanup/README.md` | Accepted standalone simulation-status and recurring history-evidence cleanup. |
| `experiments/0091-committed-prototype-run-modifier/README.md` | Accepted exact Shift+W Run displacement, presentation, and native-process proof. |
| `experiments/0092-committed-camera-relative-locomotion/README.md` | Accepted exact four-orbit Walk/Run mapping and same-sample E+W process proof. |
| `experiments/0093-exact-canonical-object-query/README.md` | Accepted exact committed authored-object lookup, atomic CPU/GPU lifetime, and lifecycle evidence. |
| `assets/third-party/khronos-fox/README.md` | Pinned Khronos Fox source provenance, hashes, attribution, and redistributable license record. |
| `crates/engine-runtime/Cargo.toml` | Canonical runtime package and dependency boundary. |
| `crates/engine-runtime/build.rs` | Runtime shader compilation, Agility export linkage, and native SDK staging. |
| `crates/engine-runtime/src/lib.rs` | Public runtime, canonical object query, typed actor-simulation outcome, capture, semantic, and signed-address surface. |
| `crates/engine-runtime/src/runtime/mod.rs` | Sole renderer/scene facade, frame coordinator, committed object/terrain queries, schedule/actor owner, typed canonical render-admitted advance, and actor-relative camera mutation. |
| `crates/engine-runtime/src/runtime/object_query.rs` | Exact committed authored-object result, presentation alias, and fixed per-region identity capacity. |
| `crates/engine-runtime/src/scene/mod.rs` | Canonical camera state plus validated atomic absolute and actor-anchored candidate publication. |
| `crates/engine-runtime/src/runtime/actor.rs` | Capacity-one actor slot, nonzero generation, exact motion/presentation/animation-epoch lifetime, transition identity, and checked complete-state replacement. |
| `crates/engine-runtime/src/runtime/motion_batch.rs` | Private bounded local multi-tick motion, checked batch-entry velocity delta, final grounded witness, query accumulation, and failure context. |
| `crates/engine-runtime/src/runtime/simulation_actor.rs` | Typed motion/presentation/initial-velocity command, prepared schedule/motion composition, complete actor transition with optional final grounded witness, blocked evidence, and rollback tests. |
| `crates/engine-runtime/src/region.rs` | Signed global region value and checked offset owner. |
| `crates/engine-runtime/src/timeline/mod.rs` | Presentation and simulation timeline ownership boundary. |
| `crates/engine-runtime/src/timeline/presentation.rs` | Deterministic presentation state, controls, counters, and successful-frame commit. |
| `crates/engine-runtime/src/timeline/simulation.rs` | Exact rational simulation accumulator, checked transaction, typed batch, and private one-hour proof. |
| `crates/engine-runtime/src/terrain_query/mod.rs` | Exact height query, caller-owned body, and minimum-correction contact transaction. |
| `crates/engine-runtime/src/terrain_query/advance.rs` | Planar-first translation/vertical composition, destination-height reuse, ordered blocked-origin query, and final tick output. |
| `crates/engine-runtime/src/terrain_query/motion.rs` | Caller-owned fixed vertical motion, checked one-tick integration, and grounded composition. |
| `crates/engine-runtime/src/terrain_query/position.rs` | Canonical signed-region/local-Q9 terrain position and checked Euclidean translation. |
| `crates/engine-runtime/src/terrain_query/translation.rs` | Caller-owned exact planar body candidate, one-query contact composition, step-up bound, and atomic output decision. |
| `crates/reference-host/src/window.rs` | Concrete single-window Win32 lifecycle, message pump, native input/activation capture, and close signaling. |
| `crates/reference-host/src/activation.rs` | Constant-state focus-burst reduction and typed bounded activation transitions. |
| `crates/reference-host/src/clock.rs` | Activation-aware bounded monotonic admission, typed outcomes/status, candidate commit, stall recovery, and reset. |
| `crates/reference-host/src/input.rs` | Fixed normalized held and sample-edge key state, suppression, focus cleanup, and empty-ingest expiry. |
| `crates/reference-host/src/bootstrap.rs` | Strict schema-2 arguments/config/pack paths, playable-region validation/evidence, and hidden canonical-ready driver. |
| `crates/meshlet-catalog/build.rs` | Verified build-time glTF geometry/joint/weight cook, normalization, normals, LOD simplification, and canonical payload emission. |
| `crates/meshlet-catalog/src/imported.rs` | Strict canonical imported-geometry/binding payload decoder and metadata owner. |
| `crates/meshlet-catalog/src/procedural.rs` | Retained deterministic fixture generation for procedural archetypes 0 through 6. |
| `crates/animation-catalog/build.rs` | Verified build-time Fox hierarchy, inverse-bind, clip sampling, normalized palette, and canonical payload cook. |
| `crates/animation-catalog/src/imported_rig.rs` | Strict canonical imported-rig payload decoder and metadata owner. |
| `crates/animation-catalog/src/lib.rs` | Dual fixed rig-bank catalog, source-duration clock constants, rig-aware CPU pose oracle, encoding, validation, and hashing. |
| `crates/surface-catalog/build.rs` | Verified build-time Fox material/PNG validation, box reduction, mip generation, and payload emission. |
| `crates/surface-catalog/src/imported_material.rs` | Strict canonical imported-material payload decoder, mip verification, and metadata owner. |
| `crates/region-format/src/global.rs` | Signed schema-3 spatial, identity, and presentation object pack codec. |
| `crates/region-format/src/lib.rs` | Canonical spatial/presentation record ABI, constructors, field bounds, and shared validation. |
| `crates/terrain-format/src/global.rs` | Signed terrain pack codec and exact lookup. |
| `crates/canonical-object-fixture/src/lib.rs` | Deterministic arbitrary-Q8 authored object fixture. |
| `tools/region-cooker/src/main.rs` | Signed schema-3 object cooker CLI with physical triple ordering and controlled presentation profiles. |
| `tools/terrain-cooker/src/main.rs` | Signed terrain cooker CLI. |
| `apps/prototype/src/main.rs` | Non-diagnostic composition root, camera-relative playable-boundary admission, one-time traversal activation, Ready-only typed simulation/frame ordering, committed Run/Jump composition, block accounting, current-actor readiness, and Escape host-exit consumer. |
| `apps/prototype/src/actor.rs` | Prototype-owned grounded spawn motion and fixed gravity policy. |
| `apps/prototype/src/boundary.rs` | Prototype-owned independent maximum-batch per-axis playable-region admission policy. |
| `apps/prototype/src/camera.rs` | Prototype-owned committed four-state Q/E actor-relative camera-orbit policy. |
| `apps/prototype/src/jump.rs` | Prototype-owned capacity-one grounded Space intent, discontinuity, and committed-consumption policy. |
| `apps/prototype/src/locomotion.rs` | Prototype-owned fixed W/A/S/D Walk/Run plus exact current-camera quarter rotation and bounded step-up policy. |
| `apps/prototype/src/presentation.rs` | Prototype-owned imported Survey/Walk/Run and committed eight-way locomotion-facing policy. |
| `apps/prototype/src/time.rs` | Prototype-only HostClock admission plus no-retry/no-backlog render-block consumption policy. |
| `apps/workbench/src/main.rs` | Diagnostic composition root, frame loop, and pending operator dispatch. |
| `apps/workbench/src/inspect/protocol.rs` | Compact workbench control vocabulary. |
| `apps/workbench/src/inspect/protocol/objects.rs` | Strict canonical object query payload decoding. |
| `apps/workbench/src/inspect/protocol/terrain.rs` | Strict terrain query/contact plus actor lifecycle/simulation payload decoding. |
| `apps/workbench/src/inspect/app.rs` | Main-thread control dispatch. |
| `apps/workbench/src/inspect/app/actor.rs` | Strict actor lifecycle/typed simulation dispatch and schema-2 prepared-work/commit evidence response. |
| `apps/workbench/src/inspect/app/objects.rs` | Exact committed object query dispatch and zero-query-work evidence response. |
| `crates/engine-runtime/src/streaming/address.rs` | Signed global window and bounded projection. |
| `crates/engine-runtime/src/streaming/objects/mod.rs` | Bounded schema-3 object I/O transactions. |
| `crates/engine-runtime/src/streaming/terrain/mod.rs` | Bounded signed terrain I/O transactions. |
| `crates/engine-runtime/src/rendering/async_resident/transfer.rs` | Source-addressed object CPU/GPU page residency, copy, and slot lifetime. |
| `crates/engine-runtime/src/rendering/async_resident/renderer/query.rs` | Strict committed active-page authored-object lookup and pure order/error tests. |
| `crates/engine-runtime/src/rendering/terrain/transfer.rs` | Terrain GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/composition/mod.rs` | Atomic pair publication and fixed composition. |
| `crates/engine-runtime/src/rendering/renderer/actor_projection.rs` | Private actor projection, active/pending typed admission, required failure conversion, and bounded scene-center derivation. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/mod.rs` | Fixed visible-record layout, capacity, descriptors, and skeletal GPU resource ownership. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/actor.rs` | Exact frame-resolved actor-local phase encoding and two-frame GPU upload-resource ownership. |
| `crates/engine-runtime/src/rendering/composition/traversal.rs` | Latest-wins traversal, prefetch, and rollover policy. |
| `crates/engine-runtime/src/rendering/composition/probe.rs` | Canonical attachment and oracle evidence. |
| `crates/engine-runtime/src/rendering/composition/probe/terrain_query.rs` | Dense query/contact oracle evidence and compact body-contact transition witness. |
| `crates/engine-runtime/src/rendering/frame_targets.rs` | Neutral reverse-Z depth and semantic render-target ownership. |
| `crates/engine-runtime/src/rendering/renderer/frame.rs` | Clear-only idle-shell/canonical frame dispatch. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/surface/shadow.rs` | Fixed directional-light projection and shadow probe oracle. |
| `.runseal/wrappers/init.ts` | Toolchain and repository initialization. |
| `.runseal/wrappers/guard.ts` | Repository/runtime ownership, dependency, and retired compatibility-symbol gates. |
| `.runseal/wrappers/gpu-lab.ts` | Experiment 0001 operator entry point. |
| `.runseal/wrappers/prototype.ts` | Self-contained finite-sandbox cook, conservative playable bounds, strict bootstrap, and manual prototype lifecycle entry point. |
| `.runseal/wrappers/workbench.ts` | Compact manual workbench control. |
| `.runseal/wrappers/canonical-prototype.ts` | Focused fresh-source prototype input-edge/boundary/gravity/camera-relative Walk/Run/Jump/presentation/traversal/backpressure, restart, failure, and lifecycle entry point. |
| `.runseal/wrappers/canonical-actor.ts` | Focused fresh-source actor lifecycle, schedule/actor partition and rollback, render admission, animation epoch, and GPU phase entry point. |
| `.runseal/wrappers/canonical-frame.ts` | Focused fresh-source committed object query, canonical GPU frame, and immediate replay entry point. |
| `.runseal/wrappers/canonical-resources.ts` | Focused active/quiescent same-process GPU resource plateau entry point. |
| `.runseal/wrappers/canonical-runtime.ts` | Direct canonical acceptance entry point over the converged runtime. |
| `.runseal/support/canonical-frame.ts` | Shared exact canonical frame, shadow, occlusion, and capture baseline. |
| `.runseal/support/canonical-runtime.ts` | Non-recursive canonical acceptance support. |
| `.runseal/support/canonical-setup.ts` | Typed deterministic test/build, source-cooking, identity, and corruption setup owner. |
| `.runseal/support/object-query.ts` | Independent schema-3 pack-byte oracle, strict query rejection, order, movement, rollback, and restart evidence. |
| `.runseal/support/compatibility-removal.ts` | Clear-only idle capture and retired inspect-verb rejection evidence. |
| `.runseal/support/terrain/contact.ts` | Exact contact rejection, direct classification, and bounded-witness acceptance support. |
| `.runseal/support/guard/contact-removal.ts` | Forbidden-symbol gate for the retired dense contact command and runtime coverage mode. |
| `.runseal/support/guard/terrain-transaction-removal.ts` | Forbidden-file/symbol gate for retired copied-value terrain mutation controls and support. |
| `.runseal/support/guard/simulation-control-removal.ts` | Forbidden-file/symbol gate for retired independent controls, duplicate schedule status, recurring history evidence, retained-body history, and pre-owner actor support paths. |
| `.runseal/support/guard/canonical-operator.ts` | Exact neutral canonical revision/collection and current evidence-path guard. |
| `.runseal/support/guard/live-operator-surface.ts` | Exact wrapper set, single current-boundary authority, and maintained prototype-operator documentation gate. |
| `.runseal/support/guard/input-journal-removal.ts` | Forbidden-file/symbol/verb/command gate for the retired diagnostic input journal surface. |
| `.runseal/support/actor/lifecycle.ts` | Actor presentation admission, lifecycle rollback, generation replay, restart reset, and independence support. |
| `.runseal/support/actor/admission.ts` | Retired standalone-status rejection, canonical-aggregate schedule evidence, schema-2 advance, typed pending block, zero-commit rollback, and retained-frame support. |
| `.runseal/support/actor/gpu.ts` | Exact actor candidate, frame-slot, workload, semantic, compaction, and rollback acceptance support. |
| `.runseal/support/actor/animation.ts` | Fixed-tick spawn/transition actor epoch, GPU local-phase, same-clip retention, and fractional rollback support. |
| `.runseal/support/actor/simulation.ts` | Canonical-aggregate schedule assertions plus schema-2 fractional, partition, rollback, and sole actor advance support. |
| `.runseal/support/runtime-bootstrap.ts` | Configured failure, canonical-ready, exact restart, and cleanup acceptance support. |
| `.runseal/support/prototype/host.ts` | Prototype startup/failure, exact Walk/Run/Jump simulation, camera-orbit/zero-block readiness, held-input boundary survival, Escape clean exit, restart, and no-inspect lifecycle orchestration. |
| `.runseal/support/prototype/boundary.ts` | Real activated held-input finite-edge process survival and cleanup evidence owner. |
| `.runseal/support/prototype/actor.ts` | Current actor, grounded spawn, and bounded animation-epoch readiness invariant owner. |
| `.runseal/support/prototype/camera.ts` | Exact default/orbit rig, actor anchor, and camera/frame readiness invariant owner. |
| `.runseal/support/prototype/input.ts` | Process-qualified ordered native prototype-window W/Shift/Escape/E/Space and E+W injection for locomotion and action acceptance. |
| `.runseal/support/prototype/presentation.ts` | Exact prototype Survey/Walk/Run, locomotion yaw, and committed actor presentation invariant owner. |
| `.runseal/support/prototype/process.ts` | Shared readiness framing, native Escape exit, and prototype Sidecar status/PID process owner. |
| `.runseal/support/prototype/simulation.ts` | Exact stationary, Walk, Run, camera-relative Walk, and Jump command expectation owner. |
| `.runseal/support/prototype/traversal.ts` | Exact default/orbit traversal targets, bounded async/latest-wins publication, and no-prefetch/block/failure invariant owner. |
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

### 6.2 Focused canonical validation

```powershell
runseal :canonical-actor
runseal :canonical-frame
runseal :canonical-prototype
runseal :canonical-resources
```

The actor workflow cooks fresh signed sources and proves schema-3 prepublication/advanced behavior,
typed pending-window backpressure with one prepared step/query and zero actor/schedule/presentation
commits, transactional Survey/Walk candidate admission, a retained successful frame, the
capacity-one actor's exact generation identity, alternating
frame-slot writes, existing-pipeline participation, despawn/respawn clearing, frustum rejection,
outside-window rollback, and semantic capture. Its ignored evidence belongs under
`out/captures/canonical-actor/`.

The prototype workflow runs focused runtime/host/application tests, cooks the four required signed
centers, and proves exact grounded gravity admission, stationary and explicitly activated
process-qualified native-W Walk, visible native-Shift+W Run, and same-sample native-E+W
camera-relative locomotion, one-region held-input
boundary survival, one committed
current actor authority, actor-relative camera/frame ordering,
typed Survey/Walk/Run selection with exact committed eight-way facing, render-block consumption with
zero normal-path blocks, one exact camera-derived traversal schedule with prefetch disabled,
no-readiness bootstrap failures, direct restart equality, and complete Sidecar cleanup. Its
ignored evidence belongs under
`out/captures/canonical-prototype/`.

The frame workflow cooks one fresh minimal signed pair, rejects unavailable/invalid committed object
queries, checks three authored triples against an independent pack-byte oracle, publishes it through
the sole runtime, and checks the exact accepted GPU frame plus an immediate deterministic replay.
Use it for focused renderer iteration; it is not an end-to-end acceptance substitute. Generated
evidence belongs under `out/captures/canonical-frame/` and remains ignored.

The resource workflow cooks only the three centers required by the established 32-warm/64-sampled
publication workload. It separately proves a bounded active plateau and recovery to the quiescent
process baseline. Its ignored evidence belongs under `out/captures/canonical-resources/`.

### 6.3 Canonical runtime acceptance

```powershell
runseal :canonical-runtime
```

This workflow cooks fresh signed sources and directly validates canonical correctness,
source reordering, movement, aliasing, failure rollback, all four fault gates, reactive
and prepared traversal, rollover, the runtime-owned frame transaction and deterministic
presentation time, deterministic host input and process-restart replay, configured canonical
readiness, shared reference-host ownership, prototype startup/restart/cleanup, fixed camera-visible
directional object shadows, exact committed CPU authored-object lookup, exact CPU terrain-height
query/body contact and oracle evidence, a
bounded contact transition witness, private simulation-schedule partition/rollback/one-hour proofs,
private fixed-step/translation/batch contracts, retained runtime-actor lifecycle, and the sole
explicit elapsed schedule/actor dual gate with partition equality, mid-batch rollback, retired-route
rejection, frame/presentation independence, private frame actor projection/preflight, frame-safe actor
presentation in prototype lifecycle, a same-process
clear-only idle attachment capture, retired-control rejection, 64-publication resource plateau,
and 16 complete lifecycle cycles. It must not invoke an older experiment wrapper.

Generated evidence belongs under
`out/captures/canonical-runtime/` and remains ignored.

### 6.4 Manual workbench

```powershell
runseal :workbench start
runseal :workbench terrain-open out/cooked/example/terrain.wlt
runseal :workbench objects-open out/cooked/example/objects.wlr
runseal :workbench schedule 0 0 0 0 2
runseal :workbench probe
runseal :workbench stop
```

The only frame outcomes are clear-only `idle-shell` before a pair is published and
`canonical-runtime` afterward. The idle shell has no scene or semantic object. Manual controls do
not select renderer modes, fixture variants, pass order, or local schedules.

### 6.5 Plain prototype

```powershell
runseal :prototype start
runseal :prototype status
runseal :prototype restart
runseal :prototype stop
```

The prototype has no inspect endpoint or idle-shell mode. It shows the same canonical runtime only
after configured content is ready, advances one grounded runtime actor with fixed gravity and fixed
W/A/S/D displacement on Ready samples, reduces unsafe maximum-batch axes against bootstrap-authored
inclusive playable bounds, applies one committed four-state Q/E camera orbit before every live frame, and enables
camera-driven composition traversal once after spawn with prefetch disabled. Window close, Escape,
and wrapper stop are its current controls. Start deterministically cooks the documented zero-origin
`[-8,8]²` finite sandbox, declares inclusive `[-6,6]²` playable bounds, and publishes strict schema
2 before Sidecar readiness; no prior acceptance output is required. Prototype v0 remains the sealed
base loop with explicit finite-edge, input-edge, cleanup, and discrete-camera dependencies. Pointer
camera control, infinite source service, sustained traversal policy, gameplay
interaction, and multiple actors are not part of this workflow.

### 6.6 Experiment lifecycle

1. State the hypothesis, workload, controlled variables, metrics, pass criteria, and
   evidence path before implementation.
2. Keep the proof isolated until its acceptance criteria pass.
3. Record failures as evidence; do not conceal them with fallback behavior.
4. Promote only proven reusable ownership into `crates/` or `benchmarks/`.
5. Update this file when core ownership or stable workflows change.

### 6.7 Core implementation change

1. Inspect the working tree and relevant owner files.
2. Change the narrowest responsible boundary without compatibility scaffolding.
3. Run focused checks while iterating; use `runseal :canonical-frame` when the accepted GPU frame
   boundary may have changed and `runseal :canonical-resources` when GPU resource lifetime may
   have changed.
4. Run `runseal :guard` before accepting the change.
5. Run the active GPU experiment workflow when GPU behavior or lifecycle changes.

### 6.8 Mod content workflow

- Add Wulin-specific content only after its engine dependency has passed its experiment.
- Keep Wulin code and data under `mods/wulin/`.
- Do not modify engine behavior solely to reproduce a game-specific quirk without an
  explicit engine-level requirement.
