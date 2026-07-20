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

Experiments 0031-0163 and the current ADR set through 0166 define one live content runtime
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
  semantic region, frame-resolved actor-local animation phase, pose, and exact two-word identity;
  streamed records expose their authored local ID in the high word, actor records retain their full
  generation identity, and streamed instances/ground values are skeletal-cull inputs only;
- one fixed actor candidate after the 25,600 streamed candidates, backed by one two-frame upload
  resource and consumed by the existing skeletal, surface, shadow, and occlusion path without a GPU
  copy or additional synchronization;
- one exact 1,024-slot shared-pose GPU palette keyed only by the fixed two-rig/eight-clip/64-phase
  catalog domain, retaining the 128-bone/48-byte affine stride in one 6,291,456-byte resource;
  candidate/visible capacity remains independently 25,601, every live pose index is bounded by the
  shared domain, and the historical unique-pose live branch has no setting, shader/oracle/report
  path, alias, or fallback;
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
  230,400-body checkpoint and standalone copied-value contact path have no live inspect verb,
  Runtime method, branch, coverage mode, or recurring history witness;
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
  playable bounds inside cooked `[-8,8]²` centers, whose sole focused real process receives exact-PID
  atomic held Shift/W only after idle readiness, runs for at least 15 seconds, and exits through the
  existing two-value Escape completion on the same window; after reaching inclusive local-Z
  `[-4096,-3648]`, the second exact-window helper atomically reasserts Shift/W with A before its
  bounded phase, removing any implicit cross-helper held-state dependency while repeated downs
  retain current normalization; it commits 16..=48 exact negative-X 45-Q9 Run steps, of which at
  least seven must occur after the maximum-nine-step coupled prefix and therefore prove live
  tangential-only admission while Z is reduced; stable identity/region/shape, final
  Survey/tangential yaw, continuous clock/frame progress, and idle object state complement the pure
  product tests' exact maximum-batch/per-axis authority, without a second boundary process, engine
  boundary mode, source-index inference, compatibility decoder, product telemetry, new output
  type, or weakened runtime source/query failure;
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
  through the existing bounded latest-wins state machine; a Q/E down ordered before FocusLost in one
  ingest remains one sample-scoped press/release, clears held state, commits at most one camera step,
  and does not repeat on the next empty ingest; if Q/E was already held, duplicate down before
  FocusLost produces no new press or camera step and only releases the held key, while the next
  physical down is re-admitted as one fresh press and exact next orbit, without an engine
  input/camera controller or simulation-action cancellation rule;
- one accepted post-v0 actor transaction input that checked-adds a required caller-owned vertical
  velocity delta once at nonzero fixed-step batch entry, consumes nothing on zero steps, preserves
  schedule/actor/render-admission rollback, and introduces no jump verb, retained intent, default,
  alias, independent mutation route, or prototype behavior;
- one mandatory post-v0 cleanup that deletes the recurring missing-field and invented-alias actor
  velocity requests plus their report fields; current required-field commands, admitted nonzero
  ordering, invalid-presentation rollback, and pending-window rollback remain the process authority,
  while the existing simulation removal guard forbids probe restoration without an alias,
  compatibility decoder, or replacement rejection registry;
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
  actor advance, with no recurring retired request and a stable removal guard;
- one accepted post-v0 Run modifier whose prototype derives a private gait fact only from held Shift
  plus final admitted nonzero W/A/S/D, selects exact fixed 64/45-Q9 displacement and the existing
  imported Run clip in the sole actor transaction, and adds no retained gait state, host/engine
  action state, acceleration, horizontal velocity, alternate movement path, or new asset;
- one accepted post-readiness native Run-modifier release proof that targets the exact live PID,
  atomically queues Shift/W, admits the authored 500 ms hold by monotonic deadline, releases Shift,
  retains W, and completes as exact negative-Z Walk with a later epoch; the maintained focus
  discontinuity atomically queues Space/W before focus loss on one window thread and proves both
  action and held-locomotion suppression across suspension/reset without product input state, gait
  state, telemetry, or Runtime/GPU behavior;
- one accepted post-readiness native Run-modifier re-press proof that targets the exact live PID,
  atomically queues W, admits the authored 500 ms Walk hold by monotonic deadline, presses Shift,
  and completes as exact negative-Z Run with a later epoch without product input state, retained
  gait state, telemetry, or Runtime/GPU behavior;
- one accepted post-readiness native opposite-locomotion proof that atomically queues Shift/W/S on
  the exact live window thread, holds that opposed input for at least 250 ms, releases only S, and
  completes as exact retained Shift/W negative-Z Run with a later epoch; exact product tests remain
  the cancellation authority, with no retry, product delay, relaxed threshold, input history,
  telemetry, or Runtime/GPU behavior;
- one accepted post-readiness native diagonal-Walk proof that atomically queues W/A against the
  exact live PID and produces equal negative 23-Q9 X/Z movement with Walk clip 1, yaw 40,960, and
  one Survey-to-Walk epoch transition between readiness and completion, with no pre-child helper,
  retry, product delay, threshold relaxation, product output, input history, telemetry, or
  Runtime/GPU behavior;
- one accepted native diagonal-to-left Walk transition in that same process: the atomic W/A prefix
  is held before a bounded delayed W release, retained A continues exact 32-Q9 left movement, and
  the final displacement uniquely decomposes into positive 23-Q9 diagonal plus 32-Q9 cardinal
  step counts; after another bounded hold A release retains that exact position for a stationary
  interval and completes as Survey/yaw 32,768, with unchanged clock/object/process ownership and no
  intermediate output, query, retry, threshold relaxation, product behavior, or Runtime/GPU/source/
  resource change;
- one accepted post-readiness native diagonal-Run proof that keeps the exact live window thread
  suspended while queueing Shift/W/A and produces equal negative 45-Q9 X/Z movement with imported
  Run clip 2, yaw 40,960, and one Survey-to-Run epoch transition between readiness and completion;
  schema-4 exact-PID transport retains atomic prefixes without a startup request, schema-3 decoder,
  response-before-post probe, retry, product delay, threshold relaxation, product output, input
  history, telemetry, or Runtime/GPU behavior;
- one accepted native diagonal-to-left Run transition in that same process: the atomic Shift/W/A
  prefix is held before a bounded delayed W release, retained Shift+A continues exact 64-Q9 left
  movement, and final displacement uniquely decomposes into positive 45-Q9 diagonal plus 64-Q9
  cardinal step counts; after another bounded hold A release retains that exact position with
  Shift-only stationary work and completes as Survey/yaw 32,768, with unchanged clock/object/
  process ownership and no
  intermediate output, query, retry, threshold relaxation, product behavior, or Runtime/GPU/source/
  resource change;
- one mandatory acceptance-report compatibility cleanup that deletes all 16 static
  `actionAfterReadiness` fields and 11 positive source-shape expectations from boundary, camera,
  object, focus, forward-release, Run-transition, opposed-locomotion, and diagonal owners;
  readiness parsing before exact-PID action dispatch remains the sole live ordering authority, and
  one central static guard scans all 11 current owners with no replacement flag, alias, decoder,
  registry, fallback, retry, product delay, product/Runtime/GPU/source/resource behavior, or
  process-count change;
- one accepted post-v0 camera-relative locomotion policy that uses the current pure Q/E camera
  candidate to quarter-rotate exact local Walk/Run into world XZ before boundary admission, authors
  facing from that final world command, and still commits orbit state only after the existing checked
  runtime camera mutation without another controller or cross-subsystem transaction;
- one accepted post-v0 exact CPU object authority that moves each verified schema-3 triple page into
  the existing source-addressed 50-slot cache, shares immutable active-page references through the
  same GPU copy completion and atomic pair publication, and exposes strict committed-snapshot lookup
  by signed region plus authored local ID with no query allocation, source I/O, GPU work, second
  scene, spatial selection, interaction policy, or persistent gameplay identity;
- one accepted exact checked conversion from a committed canonical object's finite authored closed
  Q9 X/Z into the sole signed-region/half-open-local-Q9 `TerrainPosition`, independently normalizing
  each positive edge while preserving owner-region/local-ID identity and rejecting non-lattice,
  out-of-range, and signed-overflow input without selection or interaction policy;
- one accepted bounded nearest-object query over the committed 25-page CPU snapshot that validates
  and scans at most 25,600 triples once from an in-window exact `TerrainPosition`, returns one
  optional exact object/position/delta/squared-distance result under stable owner-region/local-ID
  ties, and performs no allocation, source/GPU work, visibility filtering, enumeration, retained
  index, interaction policy, or persistent identity;
- one accepted prototype-owned capacity-one F object-observation intent with a fixed inclusive
  512-Q9 radius that survives fractional/stalled/render-blocked work, cancels on Reset/Suspended,
  queries only after a successful nonzero actor commit from its exact output position, and consumes
  only after query plus identity-only target admission succeeds;
- one accepted source-qualified canonical object identity consisting of the exact committed object
  source namespace, owner region, and authored local ID; nearest emits it without changing ties, the
  former unqualified API/payload has no overload or fallback, and this remains snapshot/source
  qualification rather than a gameplay-persistent ID;
- one accepted explicit canonical object resolver that returns exact `resolved`, `source-replaced`,
  or `outside-published-window` outcomes for a complete identity while keeping pre-publication,
  invalid ID, malformed/missing/duplicate page data, and identity divergence fatal; it replaces the
  old Runtime query method and inspect verb without an alias, executes only on demand over the sole
  committed CPU snapshot with zero allocation/source/GPU/synchronization work and no internal
  recurring resolution;
- one live-Runtime-scoped typed canonical object snapshot stamp carrying the sole published pair's
  token plus source namespace, and one prototype-owned retained target carrying only qualified
  identity, last validated stamp, and resolved/outside-window availability; explicit empty scans and
  source replacement clear it, same-source departure retains unavailable intent, successful frames
  compare stamps only while a target exists, equal stamps eliminate resolver work, and changed
  stamps trigger exactly one typed resolution without copied object state, automatic nearest scans,
  engine target ownership, or gameplay-persistent identity;
- one optional immutable Selected/Activated/Rejected frame object feedback value that validates before work
  and projects source/owner region only after the frame's pending publication; exact streamed
  semantic-region/authored-ID matches receive the fixed amber/green/red mix in the sole surface resolve,
  `RenderOutcome` returns feedback only for an exact projected identity, and existing visible-record,
  root-constant, statistics, dispatch, resource, descriptor, copy, readback, and synchronization
  ownership remains structurally fixed;
- one pure exact `CanonicalObject::proximity_from` authority over signed-region/half-open-local-Q9
  positions consumed by nearest search and prototype action eligibility, plus one prototype-owned
  capacity-one Enter intent that resolves the retained target only after a nonzero actor commit,
  applies an inclusive 512-Q9 gate, and commits a 12-successful-projected-frame acknowledgement with
  reset/suspend cancellation, ineligible consumption, malformed rollback, and no canonical object
  mutation, engine-owned action state, persistent gameplay identity, networking, or Wulin semantics;
- one prototype-owned runtime-session consumption slot committed only with the exact projected
  Activated frame; its qualified identity immediately excludes only itself from nearest selection
  while preserving full 25,600-candidate validation/counting, becomes immutable frame suppression
  after the existing 12-frame acknowledgement, projects only for the matching source/window, and
  rejects the exact active-index/local-ID after grounding but before frustum/visibility in the sole
  skeletal cull; the maintained post-readiness gate starts an acceptance-owned exact-window pixel
  observer before recovery input, requires an interactive desktop, captures the exact visible
  client through synchronized full-content window composition rather than an RDP desktop DC,
  detects the fixed Activated-green class rising from its captured baseline and returning for two
  samples after acknowledgement, restores its temporary
  no-activation topmost capture state, and only then posts Escape, with a 10-second failure deadline
  but no success dwell or product telemetry; timeout still restores z-order, posts product Escape,
  and retains typed failure/capture-owner evidence through exact final completion before rejection,
  while the final product report still requires exact
  12-frame acknowledgement plus following suppression, so same-source departure/revisit, source
  replacement, restart, and every downstream render path retain one deterministic lifetime without
  canonical mutation, registry, inventory, persistence, networking, or Wulin semantics;
- one pure Prototype object-action facing gate that maps only the committed actor output's eight
  exact yaws to integer planar directions, admits a non-coincident in-radius target only for a
  positive exact Q9 dot product, retains zero-distance eligibility, consumes side/rear attempts as
  typed `OutsideFacing`, and rejects malformed yaw before mutation without new retained state,
  engine/GPU work, visibility policy, registry, or Wulin semantics;
- one exact Prototype `OutsideFacing` frame rejection that submits immutable red `Rejected`
  feedback only for a resolved in-radius target, reuses the existing 12-successful-projected-frame
  acknowledgement slot, returns `applied=false`, and never commits consumption, exclusion, or
  suppression; other ineligible outcomes remain feedback-free and no second timer, queue, action
  state, renderer lifetime, pass, resource, descriptor, copy, readback, or synchronization exists;
- one accepted bounded non-diagnostic Prototype session contract that publishes exactly one
  sequence-1 readiness value only after the first successful nonzero commit/frame and exactly one
  sequence-2 completion value only after a later graceful Escape or window-close exit in the same
  process, after GPU idle and before teardown, with exact final actor/clock/frame/object-action
  state, no event stream/history, and forced termination remaining completion-free;
- one accepted native window-close session gate that posts exactly one `WM_CLOSE` to the visible
  class/title/PID-qualified Prototype window after readiness and proves the existing two-value
  completion with reason `window-close`, stable process/actor identity, idle object policies, exit
  zero, and no activation, key transition, direct destroy, process termination, product behavior,
  schema, telemetry, engine/GPU, or resource change;
- one accepted native focus-discontinuity session gate that atomically posts Space/F/Enter/W
  immediately before `WM_KILLFOCUS` on the exact visible process window thread, observes suspended
  sampling, posts `WM_SETFOCUS`, and then atomically posts fresh A-down, bounded A-up, and delayed
  Escape on the same PID/window; it proves one suspend/resume pair, one post-resume reset, later
  Ready progress, zero elapsed backlog/stalls/blocks, exact negative-X 32-Q9 Walk admission with
  zero Z displacement and final Survey/left yaw, and idle observation/interaction with zero action
  counts; the queued Jump, observation, activation, and held W cannot reach resumed nonzero
  simulation while fresh locomotion does, without claiming immediate edge deletion or adding
  product input/clock/schema/telemetry or Runtime/GPU/resource changes;
- one accepted native Jump-readmission session gate that posts one grounded Space, waits beyond
  the exact 48-step flight while Space remains held, atomically posts duplicate Space-down plus
  focus loss on the exact visible process window thread, resumes, and then posts Space up/down
  before one same-helper monotonic delayed Escape; it proves focus cleanup cannot repeat held Jump,
  the later normalized press starts the exact second `4369/-179` flight, and the clock records one
  suspend/resume/reset while identity, X/Z, shape, Survey presentation/epoch, object policy,
  session schema, Runtime, and engine/GPU/resource ownership remain unchanged;
- one accepted native midair-Jump rejection session gate that posts Space down/up, waits a bounded
  interval, posts a second Space with W, and exits after another bounded interval; the final exact
  single-impulse trajectory proves no second impulse while same-batch Walk displacement proves
  product admission, with no Jump queue/state/report, Runtime query, or engine/GPU/resource change;
- one accepted native held-camera repeat session gate that begins at orbit-zero readiness, posts
  E-down to the exact PID, holds it for at least 250 ms, then posts repeated E-down plus W to the
  same window and proves duplicate-down suppression through retained orbit-one negative-X/zero-Z
  Walk output, with no input history, action queue, controller state, product schema, Runtime, or
  engine/GPU/resource change;
- one accepted native camera re-press session gate that begins at orbit-zero readiness, posts and
  holds E-down for at least 250 ms, atomically queues E-up/E-down/W-down against the same exact
  window thread, and proves the fresh press edge commits orbit 2 through positive-Z-only
  Walk/yaw-16,384 output, with no input history, controller state, product schema, traversal change,
  Runtime, or engine/GPU/resource change;
- one mandatory invalid-key alias and workspace-resource cleanup that deletes the recurring
  `invalid-camera-alias` process, native sequence, dispatcher, raw report, paired invariant, and
  validator; exact `0x145` rejection and absence of `0x45` held/pressed/released state remain beside
  the checked `u8::try_from` normalizer and under one static removal guard, complete Prototype
  acceptance retains 18 graceful zero-copy pairs, and guarded workspace-local `target/` plus `out/`
  are removed without a replacement process, report, alias, compatibility decoder, product
  behavior, Runtime, renderer/GPU/source/synchronization change, or shared-cache cleanup;
- one accepted native opposite-camera-edge session gate that suspends only the exact visible
  process window thread while atomically queuing Q-down/E-down/W-down, restores it before
  execution, and proves both press edges survive one ingest and cancel through orbit-zero
  negative-Z-only Walk output, with no input history, telemetry, controller state, product schema,
  Runtime, or engine/GPU/resource change;
- one accepted native counter-clockwise camera session gate that atomically queues Q-down/W-down
  against the exact visible window thread after orbit-zero readiness and proves the Q edge wraps
  the pure camera candidate to orbit 3 through positive-X-only Walk/yaw-zero output, with no input
  history, controller state, product schema, traversal change, Runtime, or engine/GPU/resource
  change;
- one accepted post-readiness native object-action gate that first establishes the exact product
  PID plus idle observation/interaction state; the Activated process first atomically queues
  F/Enter before focus loss, resumes through one reset boundary, and then queues fresh F/Enter to
  prove stale intent cancellation plus one exact stationary action; before that fresh action, one
  Enter-only attempt with no target commits exactly one feedback-free ineligible outcome, then an
  exact-window atomic Enter-up/F-down/Enter-down prefix proves immediate recovery into the existing
  12-frame Activated, source-qualified consumption/exclusion, and suppression lifetime with no
  poisoned input/action state; the unchanged Rejected process and independent source oracle retain
  stationary rejection and motion-then-capacity rejection evidence; the former `"object-action"`
  startup request and action-specific readiness oracles are deleted because queue-before-resume does not prove
  message-pump-before-current-frame ordering, with no retry, event stream, product delay, relaxed
  threshold, product change, or Runtime/GPU/resource change;
- one accepted object-focus evidence ownership boundary that emits the exact native recovery input
  once at the admitted session top level, keeps only cancellation/resume, missing-target/hold, and
  focus-clock evidence under `focusRecovery`, and deletes the duplicate nested recovery action plus
  three constant interpretation flags; exact raw launch/completion and final behavior gates remain
  authoritative, with a central guard rejecting every retired token and no alias, decoder,
  compatibility layer, product behavior, process, Runtime, or GPU/resource change;
- one mandatory compatibility cleanup that deletes three `delayedExit` object-action report aliases
  from sustained consumption, Activated recovery, and Rejected action while retaining their exact
  ordered messages, requested delay checks, measured exit intervals, native ownership, and process
  completion; the private validator still enforces both current terminal and non-terminal modes,
  one current-owner guard rejects the retired spelling, and no replacement field, decoder,
  fallback, product behavior, process, Runtime, or GPU/resource change exists;
- one accepted native retained-target OutsideRadius witness in the existing Rejected process that
  completes the current 12-frame OutsideFacing acknowledgement, releases F/Enter, holds and
  releases D inside one measured same-PID/window/thread native helper, then submits Enter without F
  or another nearest scan; the same resolved source-qualified identity remains retained, an
  independent signed-region/Q9 oracle proves exact Q18 distance beyond the inclusive radius, the
  second ineligible action adds no Activated/Rejected/suppression frame, and no process, product
  behavior, Runtime, renderer/GPU/source/synchronization owner, compatibility path, or resource
  cleanup exists;
- one mandatory acceptance ownership cleanup that replaces every graceful launch's split
  `postReadinessInput` / `exitInput` report pair with one required `nativeInput`, deletes 17 exact
  terminal-value duplicates and their equality-only checks, retains the sustained capacity
  session's independently posted Escape under one `terminal` child and direct WM_CLOSE as the
  window-close input, and makes the existing 500-line session guard reject both retired spellings
  across current owners with no alias, decoder, dual write, version branch, product behavior,
  process, Runtime, renderer/GPU/source/synchronization change, or resource cleanup;
- one accepted Prototype invariant-evidence ownership boundary that keeps complete readiness,
  completion, native-input, clock, camera, simulation, and object values only under each of 19 raw
  graceful launches; paired invariants retain only checked derived facts, and one fixed pairwise
  runtime gate rejects every exact copied object/array subtree of at least 16 minified UTF-8 bytes,
  with a zero-copy aggregate fact and no alias, decoder, dual write, relaxed behavior gate, process,
  product, Runtime, renderer/GPU/source/synchronization change, or resource cleanup;
- one accepted acceptance-process cleanup that deletes the separate readiness-only
  `objectActionBaseline` launch, its report field, two parameters, and pairwise startup/actor
  comparisons; Activated and Rejected now validate the exact bytes, SHA-256, paths, signed
  coordinates, active radius, bounds, and actor center of their host-authored schema-2 documents
  directly, while the two first/restart captures and all 19 graceful processes remain, with no
  replacement process, alias, fallback, relaxed gate, product behavior, Runtime,
  renderer/GPU/source/synchronization change, or resource cleanup;
- one mandatory acceptance compatibility cleanup that deletes five readiness-only action process
  launches, every `StartupInput` request/dispatcher, implicit PID-zero next-window selection,
  `startupNativeInput`, four action-only command expectations, and the remaining pre-child action
  helper route; all maintained native actions now begin after idle readiness and target one exact
  PID, while every positive delayed key and delayed exit uses a monotonic lower-bound deadline,
  and the eight residual per-session startup-report shape branches are deleted from camera
  repeat/re-press, Run release/re-press, opposite locomotion, diagonal Walk/Run, and forward
  release; one static owner guards every current source, with no alias, fallback, decoder, retry,
  product sleep, relaxed threshold, product behavior, Runtime, or renderer/GPU/resource change;
- one accepted normal native forward-release session that replaces the redundant idle Escape
  process/report field, begins only after idle readiness against the exact PID, atomically posts
  W-down, holds Walk for at least 250 ms, posts W-up, retains at least 250 ms of stationary work,
  then exits through Escape; the sole completion proves exact 32-Q9 movement followed by Survey
  clip 0 with retained committed yaw 49,152, stable actor/clock/object state, and zero render blocks,
  with no extra process, intermediate product output, input history, event stream, compatibility
  alias, product behavior, Runtime, or renderer/GPU/resource change;
- one mandatory bootstrap compatibility/resource cleanup that deletes three recurring
  fallback-driven invalid-document process launches, their report fields, and fallback/schema-1
  unit assertions while retaining current path/projection/bounds and missing/corrupt source
  failures; the existing compatibility-removal guard is the sole return authority, and the
  scheduled operator removes only resolved workspace-local `target/` and `out/` after validation
  without a cleanup wrapper, global-cache mutation, or product behavior;
- one accepted identity-only Prototype capacity-exhaustion rejection that submits existing red
  `Rejected` feedback only for a different currently resolved target without canonical resolution,
  proximity, or facing work, returns `applied=false`, reuses the sole acknowledgement owner, and
  preserves the immutable first consumed identity, count, exclusion, and simultaneous suppression
  without another timer, result history, mutation, registry, inventory, or product effect;
- one mandatory compatibility cleanup that deletes the duplicate transient Prototype action
  `attempt`/`completion` readiness fields, `FrameCompletion` return echo, report mapper,
  composition plumbing, test assertions, and acceptance consumers without an alias or fallback;
  exact renderer-returned projected feedback plus acknowledgement/counter/consumption/exclusion/
  suppression state remain the sole authority, with stable absence checks in the existing session
  guard and no behavior or engine/GPU/resource change;
- one mandatory compatibility cleanup that deletes ten recurring process requests for settled
  calibration/world, standalone-contact, and caller-owned terrain-body routes plus their
  `removedVerbs` report and mixed-purpose support module; owner-specific static guards remain the
  sole removal authority, while current clear-only status/image/semantic evidence lives only as
  `idleShell`, with no replacement rejection registry, alias, or runtime/product behavior;
- one mandatory post-v0 cleanup that deletes the duplicate standalone presentation-timeline Runtime
  forwarder and inspect verb; exact presentation state remains readable only through
  `canonical.status.presentationClock`, pause/resume/set/step keep their direct exact responses, and
  no recurring retired request remains while a stable guard prevents the old route from returning;
- one mandatory Prototype acceptance cleanup that deletes raw `exitCode`, `stderr`, `exitReason`,
  `outputValueCount`, and `trailingOutput` success aliases plus paired `exactlyTwoValues` from all 18
  graceful report pairs; the shared runner still reads exact readiness/completion, bounds exit,
  rejects unsuccessful status, drains and rejects trailing stdout, and rejects stderr before
  returning, product completion remains the sole semantic outcome/reason owner, and a nested guard
  forbids every retired graceful result shape without product or process changes;
- one mandatory compatibility cleanup that deletes the final recurring `simulation.status`,
  `canonical.time.status`, and `canonical.objects.query` unknown-event requests, both private
  retired-status helpers, and all three copied report fields; existing simulation, presentation,
  and object owner guards become the sole absence authority, current malformed-payload and
  transaction-rollback tests remain live, and no alias, decoder, rejection registry, product
  behavior, or engine/GPU/resource ownership changes;
- one mandatory negative-session compatibility/resource cleanup that deletes six static
  false-valued event-stream/history/copied-object report fields plus every downstream assertion
  and summary copy, advances the sole Prototype session schema directly to v2, and makes the
  existing session guard the sole return authority across six current owners; current positive
  session/object/process evidence remains exact, and the scheduled operator removes only resolved
  workspace-local `target/` and `out/` without a decoder, replacement flag, cleanup wrapper,
  `.task/` deletion, shared-cache mutation, or product/engine/GPU/resource behavior;
- one mandatory forced-process report cleanup that deletes two static startup
  `readinessEmitted` fields, four readiness-only capture fields, two downstream reads, and one
  summary copy; startup failures retain direct successful-output rejection and exact diagnostics,
  readiness-only capture rejects extra stdout and stderr at its execution boundary, normal
  graceful-session evidence remains live, and the existing session guard prevents restoration
  without a replacement flag, alias, decoder, retry, telemetry, or product/engine/GPU/resource
  behavior;
- one accepted plain Prototype v0 stage boundary over that exact self-contained finite single-actor
  loop; it does not claim sustained product traversal, a source service, finite-edge behavior,
  gameplay interaction, multiple actors, networking, or Wulin content;
- one exact read-only CPU terrain-height query over the committed snapshot, addressed by signed
  region plus half-open local Q9 and independent from camera, render LOD, source I/O, and GPU work;
- one private pure exact vertical terrain-body contact contract with strict
  separated/touching/penetrating classification and minimum upward correction, consumed only by
  motion/translation owners and the bounded canonical probe with no standalone Runtime/inspect path;
- one clear-only diagnostic idle shell with neutral reverse-Z depth and semantic frame targets,
  no calibration scene, and no split-world control surface;
- one readback-only `perception.observe` path for acceptance hashes and semantic evidence that
  performs no PNG encoding or artifact writes; persistent captures remain explicit representative
  assets rather than a side effect of every frame assertion;
- one compact `actor.*` / `simulation.actor.advance` / `camera.*` / `source.*` / `canonical.*`
  inspect vocabulary with no standalone simulation-schedule or presentation-timeline status alias;
- one non-recursive `runseal :canonical-prototype` host/application workflow, one non-recursive
  `runseal :canonical-actor` actor GPU workflow, one `runseal :canonical-frame`
  focused GPU regression workflow, one `runseal :canonical-resources` deep same-process plateau and
  16-cycle lifecycle workflow, and one non-recursive `runseal :canonical-runtime` end-to-end
  acceptance workflow with bounded resource/lifecycle checkpoints;
- one exact optional `runseal :canonical-prototype --case=activated-frame` acceptance-only lane
  that builds one Prototype, cooks only base/startup-traversal centers, launches one existing
  Activated recovery process, reuses the sole object-feedback and copied-subtree invariant owners,
  writes an independent generated report, fails fast without an interactive desktop, and never
  replaces or feeds the no-argument 18-process matrix;
- one accepted acceptance-only in-process Win32 FFI native-action transport that lazily loads and
  explicitly closes `user32.dll`/`kernel32.dll`, preserves all 35 schema-4 exact-PID actions across
  the unchanged 18-product-process matrix including 14 atomic window-thread batches, and retires
  every per-action PowerShell process, dynamic `Add-Type`, helper-ready handshake, child transport,
  and JSON round-trip; one named/reportable 500-ms sustained rejection hold replaces the exposed
  implicit helper delay and retains exact 12 Activated/12 Rejected completion, while the separate
  frame observer remains unchanged and a failure-first sustained invariant avoids launching 14
  irrelevant sessions after divergence;
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
| `docs/architecture/canonical-acceptance.md` | Risk ownership, full/checkpoint/deep-soak boundaries, and acceptance cost policy. |
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
| `docs/adr/0097-exact-canonical-object-position.md` | Accepted exact authored-object conversion into the sole terrain-position domain. |
| `docs/adr/0098-retired-standalone-presentation-status.md` | Accepted retirement of the duplicate presentation status inspect chain. |
| `docs/adr/0099-bounded-canonical-object-nearest.md` | Accepted bounded committed-snapshot nearest-object scan and stable exact tie contract. |
| `docs/adr/0100-committed-prototype-object-observation.md` | Accepted capacity-one post-commit prototype object-observation intent. |
| `docs/adr/0101-source-qualified-object-identity.md` | Accepted exact source-qualified committed object identity and stale-address rejection. |
| `docs/adr/0102-typed-canonical-object-resolution.md` | Accepted typed on-demand object lifetime resolution and strict invalid-state separation. |
| `docs/adr/0103-retired-standalone-terrain-contact.md` | Accepted retirement of standalone contact Runtime/inspect ownership and retention of one private pure contract. |
| `docs/adr/0104-version-gated-prototype-object-target.md` | Accepted live snapshot stamp and version-gated identity-only prototype target policy. |
| `docs/adr/0105-frame-bound-object-target-feedback.md` | Accepted immutable frame target projection and exact existing-surface visual feedback. |
| `docs/adr/0106-committed-prototype-object-action.md` | Accepted exact committed target proximity, projected action commit, and bounded acknowledgement. |
| `docs/adr/0107-capacity-one-prototype-object-consumption.md` | Accepted capacity-one consumed identity, nearest exclusion, deferred exact GPU suppression, and source/window lifetime. |
| `docs/adr/0108-retired-recurring-compatibility-witness.md` | Accepted removal of recurring retired-verb process evidence and retention of one current idle-shell authority. |
| `docs/adr/0109-committed-prototype-object-facing.md` | Accepted exact committed eight-way front-half-plane Prototype action gate. |
| `docs/adr/0110-frame-bound-object-rejection-feedback.md` | Accepted exact red facing-rejection projection over the existing frame transaction and bounded acknowledgement. |
| `docs/adr/0111-bounded-prototype-session-completion.md` | Accepted one-readiness/one-graceful-completion session contract and sustained post-readiness acceptance boundary. |
| `docs/adr/0112-frame-bound-capacity-exhaustion-feedback.md` | Accepted identity-only red capacity rejection with continuous first-identity suppression. |
| `docs/adr/0113-retired-transient-object-action-report.md` | Accepted deletion of duplicate transient action readiness/return/report surfaces. |
| `docs/adr/0114-native-window-close-session-completion.md` | Accepted exact real-process native window-close proof for the existing bounded completion contract. |
| `docs/adr/0115-native-prototype-focus-discontinuity.md` | Accepted exact real-process native focus-loss cleanup and no-backlog recovery proof. |
| `docs/adr/0116-native-prototype-jump-readmission.md` | Accepted complete live Jump landing and exact second native press readmission proof. |
| `docs/adr/0117-native-midair-jump-rejection.md` | Accepted exact native midair Space rejection with same-batch Walk admission proof. |
| `docs/adr/0118-retired-final-unknown-event-witnesses.md` | Accepted final recurring retired-verb IPC/report deletion and static owner authority. |
| `docs/adr/0119-native-held-camera-repeat.md` | Accepted native held E repeat suppression with exact retained-orbit Walk proof. |
| `docs/adr/0120-native-out-of-range-key-rejection.md` | Accepted full-value native invalid-key rejection with low-byte camera-alias exclusion. |
| `docs/adr/0121-batch-invariant-native-object-feedback.md` | Accepted atomic native F/Enter transport and stationary batch-invariant Activated/Rejected source fixtures. |
| `docs/adr/0122-native-opposite-camera-edge-cancellation.md` | Accepted atomic same-ingest opposite Q/E edge cancellation with exact orbit-zero Walk proof. |
| `docs/adr/0123-retired-bootstrap-probes-resource-cleanup.md` | Accepted fallback/schema-1 bootstrap probe deletion and scheduled workspace compiler/generated-resource cleanup. |
| `docs/adr/0124-native-counter-clockwise-camera-wrap.md` | Accepted atomic native Q/W transport and exact orbit-three positive-X Walk proof. |
| `docs/adr/0125-native-camera-repress-readmission.md` | Accepted native held-E release/re-press readmission and exact orbit-two positive-Z Walk proof. |
| `docs/adr/0126-native-run-modifier-release.md` | Accepted native held-Shift release, retained-W Walk transition, and exact focus-discontinuity batch hardening. |
| `docs/adr/0127-native-run-modifier-repress-readmission.md` | Accepted native held-W Shift re-press and exact Walk-to-Run readmission proof. |
| `docs/adr/0128-retired-actor-velocity-compatibility-probes.md` | Accepted recurring actor velocity predecessor-shape probe deletion and current admission preservation. |
| `docs/adr/0129-native-opposite-locomotion-release.md` | Accepted atomic native Shift/W/S cancellation, S-release Run readmission, and startup transport ordering. |
| `docs/adr/0130-native-diagonal-walk.md` | Accepted atomic native W/A diagonal Walk and explicit startup-helper preparation handshake. |
| `docs/adr/0131-native-diagonal-run.md` | Accepted atomic native Shift/W/A diagonal Run and zero-delay startup-prefix boundary. |
| `docs/adr/0132-post-readiness-native-object-actions.md` | Accepted exact-PID post-readiness Activated/Rejected/capacity action evidence and retired startup object-action request. |
| `docs/adr/0133-retired-startup-action-acceptance.md` | Accepted deletion of readiness-only/startup-action acceptance, exact-PID post-readiness replacement, and monotonic delayed-key admission. |
| `docs/adr/0134-post-readiness-finite-boundary-run.md` | Accepted exact-PID post-readiness atomic Shift/W finite-boundary liveness with unchanged single-process/product-output budgets. |
| `docs/adr/0135-native-forward-release.md` | Accepted normal exact-PID W release, Walk-to-Survey retained-facing completion, and direct idle-Escape report replacement. |
| `docs/adr/0136-native-focus-jump-suppression.md` | Accepted same-batch native Space/W focus-loss suppression across resumed Ready simulation. |
| `docs/adr/0137-bounded-object-suppression-dwell.md` | Accepted 250 ms post-action dwell guaranteeing evidence beyond the 12-frame acknowledgement boundary. |
| `docs/adr/0138-retired-startup-report-branches.md` | Accepted deletion of eight residual startup-report compatibility branches with one current-source removal guard. |
| `docs/adr/0139-native-focus-object-intent-suppression.md` | Accepted same-batch native F/Enter focus-loss suppression beside retained Jump/locomotion evidence. |
| `docs/adr/0140-focus-cleanup-camera-edge.md` | Accepted sample-scoped camera press/release retention across same-ingest focus cleanup. |
| `docs/adr/0141-held-camera-focus-cleanup.md` | Accepted duplicate-down suppression and release-only focus cleanup for an already-held camera key. |
| `docs/adr/0142-camera-focus-readmission.md` | Accepted fresh Q/E press readmission after release-only focus cleanup. |
| `docs/adr/0143-retired-negative-session-fields-resource-cleanup.md` | Accepted static negative Prototype session-field retirement, schema-v2 replacement, and scheduled workspace resource cleanup. |
| `docs/adr/0144-native-focus-locomotion-readmission.md` | Accepted stale-W suppression and fresh-A locomotion readmission across one native focus discontinuity. |
| `docs/adr/0145-native-jump-focus-readmission.md` | Accepted held-Space focus cleanup and fresh grounded Jump readmission in the existing native process. |
| `docs/adr/0146-native-object-focus-readmission.md` | Accepted stale F/Enter cancellation and fresh exact-object action readmission across one native focus discontinuity. |
| `docs/adr/0147-exact-finite-boundary-run-completion.md` | Accepted exact bounded Run endpoint and standard graceful completion for the sole finite-boundary process. |
| `docs/adr/0148-retired-forced-process-reports.md` | Accepted forced/readiness-only negative report retirement with direct stdout/stderr enforcement. |
| `docs/adr/0149-native-diagonal-walk-release.md` | Accepted exact native diagonal-to-left Walk release transition in the existing process. |
| `docs/adr/0150-native-diagonal-run-release.md` | Accepted exact native diagonal-to-left Run release transition in the existing process. |
| `docs/adr/0151-native-diagonal-walk-stop.md` | Accepted final-direction release to retained-facing Survey after exact diagonal-to-left Walk in the existing process. |
| `docs/adr/0152-native-diagonal-run-stop.md` | Accepted final-direction release to Shift-only retained-facing Survey after exact diagonal-to-left Run in the existing process. |
| `docs/adr/0153-retired-post-readiness-report-flags.md` | Accepted deletion of redundant post-readiness report flags with structural execution ordering and one current-owner removal guard. |
| `docs/adr/0154-native-finite-boundary-axis-reduction.md` | Accepted real-process finite-edge tangential Run continuation after independent forward-axis reduction. |
| `docs/adr/0155-native-boundary-held-state-readmission.md` | Accepted exact-window Shift/W/A reassertion removing the finite-boundary phase's implicit cross-helper held-state dependency. |
| `docs/adr/0156-native-missing-target-action-recovery.md` | Accepted one-shot missing-target ineligibility followed by exact-window source-qualified Activated recovery in the existing process. |
| `docs/adr/0157-object-focus-evidence-single-ownership.md` | Accepted single-owner object recovery input evidence and deletion of duplicate focus report interpretations. |
| `docs/adr/0158-retired-delayed-exit-report-alias.md` | Accepted deletion of redundant object-action delayed-exit report aliases with exact messages and intervals retained. |
| `docs/adr/0159-native-retained-target-outside-radius.md` | Accepted real-process retained-target OutsideRadius proof with one measured native motion interval and independent exact proximity. |
| `docs/adr/0160-single-owner-prototype-native-input.md` | Accepted one native-input report owner per graceful Prototype launch and deletion of split-field duplication. |
| `docs/adr/0161-single-owner-prototype-invariant-evidence.md` | Accepted raw-launch evidence ownership with derived-only invariants and a fixed pairwise non-duplication gate. |
| `docs/adr/0162-retired-object-action-readiness-baseline.md` | Accepted deletion of the redundant object-action readiness process in favor of exact host-document startup validation. |
| `docs/adr/0163-retired-invalid-key-alias-resource-cleanup.md` | Accepted recurring invalid-key alias witness deletion with exact checked-normalizer coverage and workspace-local resource cleanup. |
| `docs/adr/0164-frame-observable-activated-completion.md` | Accepted bounded acceptance-owned visible-frame completion for the exact Activated acknowledgement and following suppression boundary. |
| `docs/adr/0165-retired-graceful-transport-report-aliases.md` | Accepted deletion of constant graceful transport result aliases with shared runner control-flow enforcement retained. |
| `docs/adr/0166-focused-activated-frame-acceptance.md` | Accepted one-process exact Activated-frame acceptance mode with interactive-desktop preflight, window-composition evidence, and unchanged full matrix. |
| `docs/adr/0167-shared-pose-palette-capacity.md` | Accepted fixed 1,024-slot shared-pose GPU palette capacity and retired unique-pose live mode. |
| `docs/adr/0168-in-process-native-input-transport.md` | Accepted in-process Win32 FFI Prototype native-action transport and retired per-action PowerShell/C# helper lifecycle. |
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
| `experiments/0094-exact-canonical-object-position/README.md` | Accepted checked Q9 object-position conversion, seam normalization, and integration evidence. |
| `experiments/0095-mandatory-presentation-status-cleanup/README.md` | Accepted standalone presentation-status removal and canonical aggregate preservation evidence. |
| `experiments/0096-exact-canonical-object-nearest/README.md` | Accepted bounded exact nearest-object scan, independent source oracle, and integration evidence. |
| `experiments/0097-committed-prototype-object-observation/README.md` | Accepted native F+W post-commit observation and independent source-oracle evidence. |
| `experiments/0098-source-qualified-object-identity/README.md` | Accepted A/B source-qualified identity, stale-address rejection, and lifecycle evidence. |
| `experiments/0099-typed-canonical-object-resolution/README.md` | Accepted typed resolved/source/window outcomes, strict invalid-state failures, and lifecycle evidence. |
| `experiments/0100-retired-standalone-terrain-contact/README.md` | Accepted standalone contact removal, bounded private-witness preservation, and compiler/generated resource cleanup. |
| `experiments/0101-version-gated-prototype-object-target/README.md` | Accepted typed snapshot, native retained-target revalidation, A/B/window/rollback, resource, and lifecycle evidence. |
| `experiments/0102-frame-bound-object-target-feedback/README.md` | Accepted frame-bound target projection, exact surface feedback, product forwarding, and source/window lifecycle evidence. |
| `experiments/0103-committed-prototype-object-action/README.md` | Accepted native committed Enter action, exact proximity, activated frame commit, and bounded acknowledgement evidence. |
| `experiments/0104-capacity-one-prototype-object-consumption/README.md` | Accepted exact nearest exclusion, native capacity-one consumption, GPU suppression, and source/window lifecycle evidence. |
| `experiments/0105-retired-recurring-compatibility-witness/README.md` | Accepted retired-verb IPC/report deletion, static removal authority, and current idle-shell preservation evidence. |
| `experiments/0106-committed-prototype-object-facing/README.md` | Accepted native front-facing admission and side-facing rejection evidence. |
| `experiments/0107-rejected-object-action-feedback/README.md` | Accepted exact Rejected projection, bounded acknowledgement reuse, and zero-effect native rejection evidence. |
| `experiments/0108-bounded-prototype-session-completion/README.md` | Accepted bounded readiness/completion framing, sustained native post-readiness action, and forced-termination silence evidence. |
| `experiments/0109-capacity-exhausted-object-action-feedback/README.md` | Accepted exclusion-aware second-target capacity rejection and concurrent first-target suppression evidence. |
| `experiments/0110-retired-transient-object-action-report/README.md` | Accepted transient action field/type/mapper/consumer removal and projected-state authority evidence. |
| `experiments/0111-native-window-close-session-completion/README.md` | Accepted visible exact-window WM_CLOSE, two-value completion, and unchanged Escape/session evidence. |
| `experiments/0112-native-focus-discontinuity/README.md` | Accepted native W/focus-loss cleanup, suspended sampling, exact recovery, and unchanged-actor evidence. |
| `experiments/0113-native-jump-readmission/README.md` | Accepted full first Jump landing, timed Space re-press, and exact second-flight completion evidence. |
| `experiments/0114-native-midair-jump-rejection/README.md` | Accepted timed midair Space rejection, single-flight arithmetic, and Walk admission evidence. |
| `experiments/0115-retired-final-unknown-event-witnesses/README.md` | Accepted final three unknown-event request/helper/report deletions and preserved current strictness evidence. |
| `experiments/0116-native-held-camera-repeat/README.md` | Accepted native held E repeat suppression and retained orbit-one locomotion evidence. |
| `experiments/0117-native-out-of-range-key-rejection/README.md` | Accepted native key 325 rejection and low-byte E truncation exclusion evidence. |
| `experiments/0118-batch-invariant-native-object-feedback/README.md` | Accepted root-cause diagnosis, atomic F/Enter transport, and stationary batch-invariant Activated/Rejected evidence. |
| `experiments/0119-native-opposite-camera-edge-cancellation/README.md` | Accepted atomic native Q/E/W batch and exact opposite-camera-edge cancellation evidence. |
| `experiments/0120-retired-bootstrap-probes-resource-cleanup/README.md` | Accepted recurring bootstrap compatibility-probe/report deletion and measured target/out cleanup evidence. |
| `experiments/0121-native-counter-clockwise-camera-wrap/README.md` | Accepted native Q-only counter-clockwise wrap and orbit-three positive-X Walk evidence. |
| `experiments/0122-native-camera-repress-readmission/README.md` | Accepted native held-E release/re-press and orbit-two positive-Z Walk evidence. |
| `experiments/0123-native-run-modifier-release/README.md` | Accepted native Run-modifier release, retained-W Walk transition, and helper-race diagnosis evidence. |
| `experiments/0124-native-run-modifier-repress-readmission/README.md` | Accepted native held-W Shift re-press and Walk-to-Run readmission evidence. |
| `experiments/0125-retired-actor-velocity-compatibility-probes/README.md` | Accepted recurring actor velocity compatibility-probe/report deletion and current transaction evidence. |
| `experiments/0126-native-opposite-locomotion-release/README.md` | Accepted atomic native W/S cancellation, release readmission, and startup transport race-removal evidence. |
| `experiments/0127-native-diagonal-walk/README.md` | Accepted native A-path, exact 23-Q9 diagonal Walk, and explicit helper-ready handshake evidence. |
| `experiments/0128-native-diagonal-run/README.md` | Accepted native Shift/W/A path, exact 45-Q9 diagonal Run, and schema-4 atomic startup-prefix evidence. |
| `experiments/0129-post-readiness-native-object-actions/README.md` | Accepted post-readiness native object-action ordering, exact completion/source-oracle evidence, and startup-edge race removal. |
| `experiments/0130-retired-startup-action-acceptance/README.md` | Mandatory deletion of readiness-only/startup-action acceptance, next-window selection, and sleep-based delayed-key admission. |
| `experiments/0131-post-readiness-finite-boundary-run/README.md` | Accepted atomic post-readiness Shift/W finite-boundary process survival and explicit liveness/state evidence boundary. |
| `experiments/0132-native-forward-release/README.md` | Accepted normal W-up Walk-to-Survey completion evidence and replacement of the redundant idle Escape process. |
| `experiments/0133-native-focus-jump-suppression/README.md` | Accepted same-batch Space/W focus-loss action and locomotion suppression evidence. |
| `experiments/0134-bounded-object-suppression-dwell/README.md` | Accepted bounded post-action exit timing with a deterministic suppression-frame witness. |
| `experiments/0135-retired-startup-report-branches/README.md` | Mandatory deletion of eight residual startup-report shape branches from current native-session oracles. |
| `experiments/0136-native-focus-object-intent-suppression/README.md` | Accepted same-batch F/Enter focus-loss suppression with exact idle object-policy completion evidence. |
| `experiments/0137-focus-cleanup-camera-edge/README.md` | Accepted one-step Q/E camera composition across same-ingest focus cleanup with no held repetition. |
| `experiments/0138-held-camera-focus-cleanup/README.md` | Accepted no-repeat Q/E camera composition when focus cleanup releases an already-held key. |
| `experiments/0139-camera-focus-readmission/README.md` | Accepted fresh Q/E camera press readmission after focus cleanup ends the prior held-key lifetime. |
| `experiments/0140-retired-negative-session-fields-resource-cleanup/README.md` | Mandatory static negative session-report compatibility deletion and measured target/out cleanup. |
| `experiments/0141-native-focus-locomotion-readmission/README.md` | Accepted exact stale-W suppression and fresh-A Walk/release evidence in the existing focus process. |
| `experiments/0142-native-jump-focus-readmission/README.md` | Accepted held-Space focus cleanup and exact second-flight readmission evidence without a new process. |
| `experiments/0143-native-object-focus-readmission/README.md` | Accepted stale F/Enter cancellation and fresh Activated object-action readmission evidence without a new process. |
| `experiments/0144-exact-finite-boundary-run-completion/README.md` | Accepted exact 64-Q9 endpoint band, final Survey state, and two-value completion evidence without a new process. |
| `experiments/0145-retired-forced-process-reports/README.md` | Mandatory forced/readiness-only report-chain deletion and direct current execution-boundary enforcement. |
| `experiments/0146-native-diagonal-walk-release/README.md` | Accepted exact two-phase diagonal-to-left Walk displacement and bounded native release evidence without a new process. |
| `experiments/0147-native-diagonal-run-release/README.md` | Accepted exact two-phase diagonal-to-left Run displacement and bounded native release evidence without a new process. |
| `experiments/0148-native-diagonal-walk-stop/README.md` | Accepted complete diagonal-Walk direction-key lifetime, retained-facing Survey, and exact movement evidence without a new process. |
| `experiments/0149-native-diagonal-run-stop/README.md` | Accepted complete diagonal-Run direction-key lifetime, Shift-only retained-facing Survey, and exact movement evidence without a new process. |
| `experiments/0150-retired-post-readiness-report-flags/README.md` | Mandatory redundant post-readiness report/check deletion with unchanged full native-session evidence. |
| `experiments/0151-native-finite-boundary-axis-reduction/README.md` | Accepted exact native tangential Run continuation after finite-edge forward-axis reduction without a new process. |
| `experiments/0152-native-boundary-held-state-readmission/README.md` | Accepted atomic held-state reassertion and clean-commit finite-boundary evidence without product or process expansion. |
| `experiments/0153-native-missing-target-action-recovery/README.md` | Accepted native missing-target consumption and immediate exact-object recovery without product or process expansion. |
| `experiments/0154-object-focus-evidence-single-ownership/README.md` | Accepted single-owner native recovery evidence with a measured report reduction and unchanged behavior gates. |
| `experiments/0155-retired-delayed-exit-report-alias/README.md` | Mandatory delayed-exit report-alias deletion with exact native message and interval evidence retained. |
| `experiments/0156-native-retained-target-outside-radius/README.md` | Accepted same-process retained-target OutsideRadius evidence without another scan or process. |
| `experiments/0157-single-owner-prototype-native-input/README.md` | Mandatory single-owner graceful native-input report convergence with measured duplicate-byte removal. |
| `experiments/0158-single-owner-prototype-invariant-evidence/README.md` | Accepted derived-only Prototype invariants, fixed pairwise copy rejection, and measured report reduction. |
| `experiments/0159-retired-object-action-readiness-baseline/README.md` | Accepted readiness-only process/report deletion with exact schema-2 bytes/hash startup evidence. |
| `experiments/0160-retired-invalid-key-alias-resource-cleanup/README.md` | Mandatory invalid-key alias process/report deletion, checked-normalizer evidence, and workspace-local resource cleanup. |
| `experiments/0161-frame-observable-activated-completion/README.md` | Accepted replacement of the cold-fragile Activated wall-time proxy with bounded exact-window frame-semantic completion. |
| `experiments/0162-retired-graceful-transport-report-aliases/README.md` | Mandatory graceful transport report-alias deletion with exact runner and product completion authority retained. |
| `experiments/0163-focused-activated-frame-acceptance/README.md` | Accepted one-process focused Activated-frame loop with exact window-composition/shared-invariant evidence and full-matrix neutrality. |
| `experiments/0164-shared-pose-palette-capacity/README.md` | Accepted exact shared-pose palette-capacity reduction with focused GPU, release timing, resource, and integration evidence. |
| `experiments/0165-in-process-native-input-transport/README.md` | Accepted exact 35-action in-process FFI transport with unchanged 18-process behavior and measured full/focused iteration reduction. |
| `assets/third-party/khronos-fox/README.md` | Pinned Khronos Fox source provenance, hashes, attribution, and redistributable license record. |
| `crates/engine-runtime/Cargo.toml` | Canonical runtime package and dependency boundary. |
| `crates/engine-runtime/build.rs` | Runtime shader compilation, Agility export linkage, and native SDK staging. |
| `crates/engine-runtime/src/lib.rs` | Public runtime, typed canonical object snapshot/resolution/nearest/proximity and immutable frame-feedback surface, typed actor-simulation outcome, capture, semantic, and signed-address surface. |
| `crates/engine-runtime/src/runtime/mod.rs` | Sole renderer/scene facade, frame coordinator and projected-feedback outcome, typed object snapshot/resolution/nearest and terrain queries, schedule/actor owner, typed canonical render-admitted advance, and actor-relative camera mutation. |
| `crates/engine-runtime/src/runtime/object_query.rs` | Live snapshot stamp, source-qualified object identity/resolution, checked terrain-position/proximity conversion, bounded nearest-query result, and immutable Selected/Activated/Rejected feedback contracts. |
| `crates/engine-runtime/tests/object_position.rs` | Public-API exact lattice, closed-edge normalization, rejection, and signed-overflow evidence. |
| `crates/engine-runtime/src/scene/mod.rs` | Canonical camera state plus validated atomic absolute and actor-anchored candidate publication. |
| `crates/engine-runtime/src/runtime/actor.rs` | Capacity-one actor slot, nonzero generation, exact motion/presentation/animation-epoch lifetime, transition identity, and checked complete-state replacement. |
| `crates/engine-runtime/src/runtime/motion_batch.rs` | Private bounded local multi-tick motion, checked batch-entry velocity delta, final grounded witness, query accumulation, and failure context. |
| `crates/engine-runtime/src/runtime/simulation_actor.rs` | Typed motion/presentation/initial-velocity command, prepared schedule/motion composition, complete actor transition with optional final grounded witness, blocked evidence, and rollback tests. |
| `crates/engine-runtime/src/region.rs` | Signed global region value and checked offset owner. |
| `crates/engine-runtime/src/timeline/mod.rs` | Presentation and simulation timeline ownership boundary. |
| `crates/engine-runtime/src/timeline/presentation.rs` | Deterministic presentation state, controls, counters, and successful-frame commit. |
| `crates/engine-runtime/src/timeline/simulation.rs` | Exact rational simulation accumulator, checked transaction, typed batch, and private one-hour proof. |
| `crates/engine-runtime/src/terrain_query/mod.rs` | Exact height query, caller-owned body values, and private minimum-correction contact contract. |
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
| `apps/prototype/src/main.rs` | Non-diagnostic composition root, camera-relative boundary/traversal, Ready-only simulation/frame ordering, committed Run/Jump/object-target/action/consumption composition, block accounting, current-actor readiness, and Escape host exit. |
| `apps/prototype/src/actor.rs` | Prototype-owned grounded spawn motion and fixed gravity policy. |
| `apps/prototype/src/boundary.rs` | Prototype-owned independent maximum-batch per-axis playable-region admission policy. |
| `apps/prototype/src/camera.rs` | Prototype-owned committed four-state Q/E actor-relative camera-orbit policy. |
| `apps/prototype/src/jump.rs` | Prototype-owned capacity-one grounded Space intent, discontinuity, and committed-consumption policy. |
| `apps/prototype/src/locomotion.rs` | Prototype-owned fixed W/A/S/D Walk/Run plus exact current-camera quarter rotation and bounded step-up policy. |
| `apps/prototype/src/object/mod.rs` | Prototype-owned object observation and interaction policy boundary. |
| `apps/prototype/src/object/observation.rs` | Prototype-owned F intent plus identity-only target admission, consumed-target clearing, snapshot-gated resolution, window/source lifetime, and rollback policy. |
| `apps/prototype/tests/object_observation_policy.rs` | Admission/consumed-clear/empty-clear/rollback, stamp work-elimination, window revisit, source replacement, and discontinuity evidence. |
| `apps/prototype/src/object/interaction.rs` | Prototype-owned capacity-one Enter intent, exact admission/rejection, state-only frame commit, projected acknowledgement, consumption/exclusion, and identity-aware suppression. |
| `apps/prototype/tests/object_interaction_policy.rs` | Durable intent/consumption/source/capacity/suppression/acknowledgement state plus malformed/projection rollback evidence without return echoes. |
| `apps/prototype/src/session.rs` | Bounded one-readiness/one-graceful-completion report ownership without copied transient action results. |
| `apps/prototype/tests/session_report.rs` | Exact completion schema, reason, final-state, and checked frame-total evidence. |
| `apps/prototype/src/presentation.rs` | Prototype-owned imported Survey/Walk/Run and committed eight-way locomotion-facing policy. |
| `apps/prototype/src/time.rs` | Prototype-only HostClock admission plus no-retry/no-backlog render-block consumption policy. |
| `apps/workbench/src/main.rs` | Diagnostic composition root, frame loop, and pending operator dispatch. |
| `apps/workbench/src/inspect/protocol.rs` | Compact workbench control vocabulary. |
| `apps/workbench/src/inspect/protocol/objects.rs` | Strict source-qualified canonical object resolution and nearest-query payload decoding. |
| `apps/workbench/src/inspect/protocol/terrain.rs` | Strict terrain-height query plus actor lifecycle/simulation payload decoding. |
| `apps/workbench/src/inspect/app.rs` | Main-thread control dispatch. |
| `apps/workbench/src/inspect/app/actor.rs` | Strict actor lifecycle/typed simulation dispatch and schema-2 prepared-work/commit evidence response. |
| `apps/workbench/src/inspect/app/objects.rs` | Typed source-qualified committed object resolution/nearest dispatch and zero-work evidence responses. |
| `apps/workbench/src/capture.rs` | Persistent capture encoding plus readback-only observation response. |
| `apps/workbench/src/perception.rs` | Shared semantic analysis with explicit diagnostic-image materialization. |
| `crates/engine-runtime/src/streaming/address.rs` | Signed global window and bounded projection. |
| `crates/engine-runtime/src/streaming/objects/mod.rs` | Bounded schema-3 object I/O transactions. |
| `crates/engine-runtime/src/streaming/terrain/mod.rs` | Bounded signed terrain I/O transactions. |
| `crates/engine-runtime/src/rendering/async_resident/transfer.rs` | Source-addressed object CPU/GPU page residency, copy, and slot lifetime. |
| `crates/engine-runtime/src/rendering/async_resident/renderer/query.rs` | Typed source-qualified committed active-page object resolution/nearest scan with exact optional exclusion after full validation/counting, consuming the pure proximity authority, with lifetime/order/radius/error tests. |
| `crates/engine-runtime/src/rendering/async_resident/renderer/query_fixture.rs` | Deterministic committed CPU object snapshot fixture shared only by resolver/nearest unit tests. |
| `crates/engine-runtime/src/rendering/terrain/transfer.rs` | Terrain GPU copy and slot lifecycle. |
| `crates/engine-runtime/src/rendering/composition/mod.rs` | Atomic pair publication and fixed composition. |
| `crates/engine-runtime/src/rendering/renderer/actor_projection.rs` | Private actor projection, active/pending typed admission, required failure conversion, and bounded scene-center derivation. |
| `crates/engine-runtime/src/rendering/renderer/object_target.rs` | Immutable source/window-qualified Selected/Activated/Rejected feedback and exact suppression projection plus bounded skeletal packing. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/surface/target_probe.rs` | Independent visibility/authored-ID target pixel oracle. |
| `crates/engine-runtime/tests/private/object_target.rs` | Private invalid/source/window/traversal target and exact suppression projection/packing evidence. |
| `crates/engine-runtime/shaders/skeletal_scene.hlsl` | Sole streamed/actor skeletal cull, grounding, exact object suppression, compaction, animation admission, and visible-record emission. |
| `crates/engine-runtime/tests/private/surface_target.rs` | Private authored-ID permutation and neighbor-exclusion target evidence. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/mod.rs` | Fixed visible-record layout, capacity, descriptors, and skeletal GPU resource ownership. |
| `crates/engine-runtime/tests/private/palette_capacity.rs` | Exact 1,024-slot / 6 MiB shared-pose palette capacity and candidate-capacity independence evidence. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/actor.rs` | Exact frame-resolved actor-local phase encoding and two-frame GPU upload-resource ownership. |
| `crates/engine-runtime/src/rendering/composition/traversal.rs` | Latest-wins traversal, prefetch, and rollover policy. |
| `crates/engine-runtime/src/rendering/composition/probe.rs` | Canonical attachment and oracle evidence. |
| `crates/engine-runtime/src/rendering/composition/probe/terrain_query.rs` | Bounded query oracle evidence and the sole process-level body-contact transition witness. |
| `crates/engine-runtime/src/rendering/frame_targets.rs` | Neutral reverse-Z depth and semantic render-target ownership. |
| `crates/engine-runtime/src/rendering/renderer/frame.rs` | Clear-only idle-shell/canonical frame dispatch plus post-publication frame-target projection. |
| `crates/engine-runtime/src/rendering/meshlet_scene/skeletal/surface/shadow.rs` | Fixed directional-light projection and shadow probe oracle. |
| `.runseal/wrappers/init.ts` | Toolchain and repository initialization. |
| `.runseal/wrappers/guard.ts` | Repository/runtime ownership, dependency, and retired compatibility-symbol gates. |
| `.runseal/wrappers/gpu-lab.ts` | Experiment 0001 operator entry point. |
| `.runseal/wrappers/prototype.ts` | Self-contained finite-sandbox cook, conservative playable bounds, strict bootstrap, and manual prototype lifecycle entry point. |
| `.runseal/wrappers/workbench.ts` | Compact manual workbench control. |
| `.runseal/wrappers/canonical-prototype.ts` | Complete fresh-source Prototype matrix plus the exact optional one-process Activated-frame focused dispatch, report separation, interactive-desktop preflight, and explicit native-action FFI lifetime entry point. |
| `.runseal/wrappers/canonical-actor.ts` | Focused fresh-source actor lifecycle, schedule/actor partition and rollback, render admission, animation epoch, and GPU phase entry point. |
| `.runseal/wrappers/canonical-frame.ts` | Focused fresh-source typed object snapshot/resolution/position/nearest/exclusion, exact GPU feedback/suppression, clear, and replay entry point. |
| `.runseal/wrappers/canonical-resources.ts` | Focused deep active/recovery GPU resource plateau and 16-cycle lifecycle entry point. |
| `.runseal/wrappers/canonical-runtime.ts` | Timed direct canonical acceptance with bounded resource/lifecycle checkpoints. |
| `.runseal/support/canonical-frame.ts` | Shared exact canonical frame, shadow, occlusion, and capture baseline. |
| `.runseal/support/canonical-runtime.ts` | Non-recursive acceptance, operation metrics, observation, and checkpoint/soak support. |
| `.runseal/support/canonical-rollover.ts` | Prepared traversal rollover acceptance stage. |
| `.runseal/support/canonical-setup.ts` | Typed deterministic test/build, source-cooking including batch-invariant object-action fixtures, identity, and corruption setup owner. |
| `.runseal/support/resource-acceptance.ts` | Pure warm-convergence, active, and recovery resource threshold policy used by checkpoints and deep soak. |
| `.runseal/support/resource-acceptance_test.ts` | Injected warm, early/delayed handle, and private-byte growth rejection evidence. |
| `.runseal/support/object/query.ts` | Independent schema-3 namespace/raw/Q9 oracle plus snapshot/published-pair, typed resolution, strict payload, rollback, and restart evidence. |
| `.runseal/support/object/nearest.ts` | Independent versioned source-qualified bounded nearest/exclusion oracle, strict origin/radius/tie, order, movement, rollback, and restart evidence. |
| `.runseal/support/object/feedback.ts` | Exact Selected/Activated/Rejected feedback and suppression, invalid input, replay/clear, and source/window projection lifecycle gates. |
| `.runseal/support/prototype/object/observation.ts` | Idle readiness observation/target/copy-state invariant owner. |
| `.runseal/support/prototype/object/observation_order.ts` | Zero-dependency valid asynchronous traversal/observation token-order contract. |
| `.runseal/support/prototype/object/observation_test.ts` | Equivalent pre/post asynchronous traversal observation order and impossible-token rejection evidence. |
| `.runseal/support/prototype/object/interaction.ts` | Idle readiness action/facing/acknowledgement/consumption/suppression invariant owner. |
| `.runseal/support/prototype/object/capacity.ts` | Exact sustained capacity-rejection motion/action input evidence owner. |
| `.runseal/support/prototype/object/gates.ts` | Single-owner post-readiness exact-PID Activated focus/missing-target input evidence, Rejected retained-target range/capacity completion, direct expected-startup/actor admission, bounded suppression dwell, independent source identity, restart, and unchanged-subsystem gate composition. |
| `.runseal/support/prototype/object/input-gates.ts` | Exact same-window missing-target, atomic Enter-release/F/Enter recovery, and retained-target motion/action input evidence owner. |
| `.runseal/support/prototype/object/outside-radius.ts` | Exact retained-target signed-region/Q9 proximity and bounded actor-transition acceptance owner. |
| `.runseal/support/object/integration.ts` | Object resolution/nearest source, window, movement, and corrupt-pair preservation integration gates. |
| `.runseal/support/idle-shell.ts` | Current clear-only status, renderer-health, image, and uniformly background semantic evidence. |
| `.runseal/support/guard/contact-removal.ts` | Forbidden-symbol gate for retired dense/standalone contact surfaces and required private witness authority. |
| `.runseal/support/guard/compatibility-witness-removal.ts` | Forbidden old support/report names and retired bootstrap fallback/schema probes plus required current idle-shell authority gate. |
| `.runseal/support/guard/terrain-transaction-removal.ts` | Forbidden-file/symbol gate for retired copied-value terrain mutation controls and support. |
| `.runseal/support/guard/simulation-control-removal.ts` | Forbidden-file/symbol gate for retired independent controls, duplicate schedule status, recurring history evidence, retained-body history, and pre-owner actor support paths. |
| `.runseal/support/guard/presentation-status-removal.ts` | Forbidden-symbol/verb gate for the retired standalone presentation status chain. |
| `.runseal/support/guard/canonical-operator.ts` | Exact neutral canonical revision/collection and current evidence-path guard. |
| `.runseal/support/guard/live-operator-surface.ts` | Exact wrapper set, single current-boundary authority, and maintained prototype-operator documentation gate. |
| `.runseal/support/guard/input-journal-removal.ts` | Forbidden-file/symbol/verb/command gate for the retired diagnostic input journal surface. |
| `.runseal/support/guard/object-identity.ts` | Required typed source-qualified resolver, nearest exclusion, frame suppression, prototype consumption, and forbidden old-surface gate. |
| `.runseal/support/guard/prototype-session.ts` | Required bounded Escape/window-close/focus/Jump/camera/Run/boundary/object post-readiness sessions, in-process FFI exact-PID schema-4 timing/cleanup, forbidden external action helpers, 18-pair single-owner native-input/invariant reporting, deterministic object startup validation, direct forced-process enforcement, and retired-surface guards. |
| `.runseal/support/guard/prototype/frame-completion.ts` | Required exact-window Activated-green rise/clear observer, bounded failure, delayed Escape, and forbidden fixed-dwell restoration gate. |
| `.runseal/support/guard/prototype/native-action-transport.ts` | Required in-process Win32 FFI load/search/timing/atomic/cleanup ownership and forbidden external native-action helper gate. |
| `.runseal/support/guard/prototype/transport-aliases.ts` | Required graceful runner timeout/status/stdout/stderr control flow and forbidden constant transport result alias gate. |
| `.runseal/support/guard/skeletal/palette.ts` | Exact shared-pose palette allocation/descriptor/probe authority and forbidden retired unique-pose branch gate. |
| `.runseal/support/actor/lifecycle.ts` | Actor presentation admission, lifecycle rollback, generation replay, restart reset, and independence support. |
| `.runseal/support/actor/admission.ts` | Canonical-aggregate schedule evidence, strict schema-2 advance, typed pending block, zero-commit rollback, and retained-frame support. |
| `.runseal/support/actor/gpu.ts` | Exact actor candidate, frame-slot, workload, semantic, compaction, and rollback acceptance support. |
| `.runseal/support/actor/animation.ts` | Fixed-tick spawn/transition actor epoch, GPU local-phase, same-clip retention, and fractional rollback support. |
| `.runseal/support/actor/simulation.ts` | Canonical-aggregate schedule assertions plus schema-2 fractional, partition, rollback, and sole actor advance support. |
| `.runseal/support/runtime-bootstrap.ts` | Configured canonical-ready/restart plus direct-output current missing/corrupt failure and bounded Prototype invariant/lifecycle checkpoints. |
| `.runseal/support/prototype/object/focused-frame.ts` | One-process exact Activated recovery launch, existing object-feedback invariant reuse, and raw/invariant single-owner gate composition. |
| `.runseal/support/prototype/host.ts` | Prototype direct-output current missing/corrupt startup failures, exact host-document bytes/hash expectations, plain readiness/restart baselines, post-readiness action sessions, exact boundary completion, 18-pair single-owner evidence aggregation, Sidecar status/PID ownership, and no-inspect lifecycle orchestration. |
| `.runseal/support/prototype/boundary.ts` | Post-readiness exact-PID held-Run duration, atomic tangential-phase held-state reassertion, conservative endpoint band, final Survey, clock/frame, and graceful finite-edge completion oracle. |
| `.runseal/support/prototype/actor.ts` | Current actor, grounded spawn, and bounded animation-epoch readiness invariant owner. |
| `.runseal/support/prototype/camera.ts` | Exact default/orbit rig, actor anchor, camera/frame readiness, held-repeat, and atomic opposite-edge locomotion oracle owner. |
| `.runseal/support/prototype/camera_counter_clockwise.ts` | Exact native counter-clockwise wrap, orbit-three Walk, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/camera_repress.ts` | Exact native held-E release/re-press, orbit-two Walk, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/input/mod.ts` | Explicit-PID native request validation, action/delay/atomic-prefix normalization, and fixed schema-4 message expectation owner. |
| `.runseal/support/prototype/input/prepared.ts` | Lazy in-process `user32`/`kernel32` FFI lifetime, bounded exact-window discovery, monotonic delayed keys/exits, atomic thread suspend/post/resume, and schema-4 evidence validation owner. |
| `.runseal/support/prototype/input/mod_test.ts` | UTF-16 window-name encoding and pre-FFI invalid PID/action/atomic-delay rejection evidence. |
| `.runseal/support/prototype/input/frame_completion.ts` | Synchronized exact visible-window full-content capture owner, typed timeout/capture-owner diagnostics, temporary no-activation z-order control, and prepared observer lifecycle. |
| `.runseal/support/prototype/input/frame_completion_contract.ts` | Fixed Activated frame color-rise, clear-tolerance, consecutive-sample, and failure-deadline constants. |
| `.runseal/support/prototype/input/frame_completion_desktop.ts` | Focused acceptance interactive-input-desktop prerequisite probe and actionable Win32 failure owner. |
| `.runseal/support/prototype/input/frame_completion_script.ts` | Bounded baseline/rise/clear observation state machine, z-order rollback, and post-completion Escape script. |
| `.runseal/support/prototype/input/frame_completion_validation.ts` | Exact observer process, pixel-semantic, timing, and Escape evidence validator. |
| `.runseal/support/prototype/input/actions.ts` | Named Prototype boundary Run start/reassertion, frame-complete post-readiness object recovery, explicit sustained rejection completion hold, missing-target/capacity action, atomic focus-loss batches, Escape, resume, and window-close native actions. |
| `.runseal/support/prototype/input/sequences.ts` | Exact-PID post-readiness forward release, diagonal Walk/Run component and final releases, Run transitions, Jump, opposed-locomotion, camera, and delayed-exit native input sequences. |
| `.runseal/support/prototype/jump.ts` | Exact native Jump policy, first/second/single-flight arithmetic, landing/readmission, and midair-rejection oracles. |
| `.runseal/support/prototype/presentation.ts` | Exact prototype Survey/Walk/Run, locomotion yaw, and committed actor presentation invariant owner. |
| `.runseal/support/prototype/sessions/focus.ts` | Exact atomic Space/W/focus-loss batch, clock recovery, and resumed-simulation suppression oracle. |
| `.runseal/support/prototype/sessions/gates.ts` | Bounded 18-process forward-release/window-close/focus/Jump/Run release/re-press/opposite-locomotion/diagonal-Walk/diagonal-Run/camera/input/sustained session matrix, failure-first sustained validation, positive baseline comparisons, derived-only invariant composition, and fixed pairwise copied-subtree rejection. |
| `.runseal/support/prototype/sessions/mod.ts` | Shared Prototype readiness/completion framing, single-owner graceful native-input/readiness/completion report, direct timeout/status/stdout/stderr enforcement, plus exact-PID finite-boundary, object/action, missing-target, and focus-recovery execution. |
| `.runseal/support/prototype/sessions/forward_release.ts` | Post-readiness W hold/release, exact Walk-to-Survey retained-facing transition, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/sessions/diagonal_walk.ts` | Post-readiness atomic W/A, delayed W/A releases, exact 23-Q9 diagonal plus 32-Q9 retained-left Walk decomposition, retained-facing Survey, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/sessions/diagonal_run.ts` | Post-readiness atomic Shift/W/A, delayed W/A releases, exact 45-Q9 diagonal plus 64-Q9 retained-left Run decomposition, Shift-only retained-facing Survey, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/sessions/locomotion_opposition.ts` | Post-readiness atomic Shift/W/S hold, S-release Run readmission, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/sessions/run_release.ts` | Post-readiness atomic Shift/W prefix, delayed Shift release, retained-W Walk transition, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/sessions/run_repress.ts` | Post-readiness atomic W prefix, delayed Shift re-press, Run transition, clock, and bounded session oracle owner. |
| `.runseal/support/prototype/simulation.ts` | Exact stationary readiness command expectation owner. |
| `.runseal/support/prototype/traversal.ts` | Exact default/orbit traversal targets, bounded async/latest-wins publication, and no-prefetch/block/failure invariant owner. |
| `.runseal/support/terrain/query.ts` | Exact single-query rejection, seam, triangle, and dense snapshot acceptance support. |
| `.runseal/support/cooked-gltf-presentation.ts` | Imported geometry/material/rig metadata, exact GPU palette, and controlled articulation acceptance support. |
| `.runseal/support/temporal-presentation.ts` | Aggregate-clock ownership, fixed-quantum duration time, current-operation rollback, common-period, and held-pair acceptance support. |

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
camera-relative locomotion, one-region held-Run exact endpoint and graceful completion, one committed
current actor authority, actor-relative camera/frame ordering,
typed Survey/Walk/Run selection with exact committed eight-way facing, render-block consumption with
zero normal-path blocks, one exact camera-derived traversal schedule with prefetch disabled,
same-batch Space/W focus-loss suppression across resumed Ready work, no-readiness bootstrap
failures, direct restart equality, and complete Sidecar cleanup. Its
ignored evidence belongs under
`out/captures/canonical-prototype/`.

The frame workflow cooks one fresh minimal signed pair, rejects unavailable/invalid committed object
queries, checks three authored triples against an independent pack-byte oracle, publishes it through
the sole runtime, and checks the exact accepted GPU frame plus an immediate deterministic replay.
Use it for focused renderer iteration; it is not an end-to-end acceptance substitute. Generated
evidence belongs under `out/captures/canonical-frame/` and remains ignored.

The resource workflow is the deep resource/lifecycle owner. It cooks only the three centers required
by the established 32-warm/64-sampled publication workload, samples the active baseline before the
first measured publication, proves bounded active growth and at least 60 seconds of post-workload
handle stability, then runs 16 complete start/publish/probe/stop cycles. Its ignored evidence belongs
under `out/captures/canonical-resources/`.

### 6.3 Canonical runtime acceptance

```powershell
runseal :canonical-runtime
```

This workflow cooks fresh signed sources and directly validates canonical correctness,
source reordering, movement, aliasing, failure rollback, all four fault gates, reactive
and prepared traversal, rollover, the runtime-owned frame transaction and deterministic
presentation time, deterministic host-input CPU proofs, configured canonical readiness, shared
reference-host ownership, a bounded invalid/corrupt/stationary prototype startup/restart/cleanup
checkpoint, fixed camera-visible
directional object shadows, exact committed CPU authored-object lookup, exact CPU terrain-height
query/body contact and oracle evidence, a
bounded contact transition witness, private simulation-schedule partition/rollback/one-hour proofs,
private fixed-step/translation/batch contracts, retained runtime-actor lifecycle, and the sole
explicit elapsed schedule/actor dual gate with partition equality, mid-batch rollback, retired-route
rejection, frame/presentation independence, private frame actor projection/preflight, frame-safe actor
presentation in a bounded prototype startup/restart checkpoint, a same-process clear-only idle
attachment capture, retired-control rejection, an 8-publication active-resource checkpoint, and two
complete lifecycle checkpoint cycles. The maintained `canonical-resources` workflow owns the longer
64-publication, 60-second recovery, and 16-cycle soak. Runtime acceptance records per-stage time,
operation counts, and generated artifact bytes and must not invoke an older experiment wrapper.

Generated evidence belongs under
`out/captures/canonical-runtime/` and remains ignored.

### 6.4 Manual workbench

```powershell
runseal :workbench start
runseal :workbench terrain-open out/cooked/example/terrain.wlt
runseal :workbench objects-open out/cooked/example/objects.wlr
runseal :workbench schedule 0 0 0 0 2
runseal :workbench probe
runseal :workbench observe
runseal :workbench stop
```

The only frame outcomes are clear-only `idle-shell` before a pair is published and
`canonical-runtime` afterward. The idle shell has no scene or semantic object. Manual controls do
not select renderer modes, fixture variants, pass order, or local schedules. `observe` returns exact
color/object-ID hashes and semantic evidence without creating capture files; `perception <id>` is the
explicit persistent artifact path.

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
