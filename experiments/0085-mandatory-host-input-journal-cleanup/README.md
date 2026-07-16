# Experiment 0085: Mandatory Host Input Journal Cleanup

Status: Accepted

## Hypothesis

The process-local diagnostic input journal and its workbench record/replay/native-post chain can be
deleted completely now that Prototype v0 consumes normalized held and sample-edge state directly.
The remaining host input owner can become fixed, allocation-free state without changing real Win32
capture, pre-readiness locomotion, focus cleanup, Escape exit, or any engine/runtime behavior.

## Scope

Delete `HostInput` transaction vectors, counters, recording capacity/fault lifecycle, JSON status,
canonical hashes, and isolated replay. Reduce the owner to fixed held/pressed/released key sets and
direct ordered native-message reduction.

Delete `PostedMessage`, `window::post_input`, workbench `input.status`, `input.record.start`,
`input.record.stop`, `input.replay`, and `input.native.post` protocol/dispatch, their Runseal
workbench commands, `.runseal/support/host-input-replay.ts`, and the `hostInput` section of the long
canonical-runtime report. Retain no alias, redirect, old decoder, revision placeholder, or empty
compatibility result.

Remove persistent `HostInput` ownership from the diagnostic workbench after startup; it drains and
discards native input because it has no application consumer. Configured workbench bootstrap may
use one temporary input owner to preserve the shared driver order. Prototype keeps the same input
owner across hidden bootstrap and live execution because maintained native-W evidence posts before
readiness and held W must survive into the first live transaction.

Supersede ADR 0044 with a decision describing only the current normalized state boundary. Keep
Experiment 0041 as immutable historical evidence. Add one maintained guard that prevents every
retired file, verb, wrapper command, journal symbol, and workbench ownership path from returning.

Do not add input behavior, action mapping, an event queue, new diagnostics, a replacement journal,
persistent replay, a new operator, engine-runtime input, renderer/GPU/source/asset changes, or
Wulin content.

## Workload

1. Inventory every journal, inspect, native-post, wrapper, report, dependency, documentation, and
   bootstrap/workbench owner before deletion.
2. Replace journal-backed normalization with three fixed key sets. Sweep duplicate/unmatched/
   invalid messages, focus loss, same-sample dual edges, empty-ingest expiry, held continuity, and
   fixed allocation-free ownership.
3. Start the real workbench and require all five retired inspect verbs to return unsupported-event
   failures. Require the four retired wrapper commands to fail as unknown commands.
4. Deliberately reintroduce one retired verb in a live path and require the maintained removal guard
   to fail before build/test work; remove it and require the guard to pass.
5. Run `canonical-prototype-v10`. Require pre-readiness native W to retain exact moving readiness,
   native Escape to exit 0, held-W finite-edge survival, restart equality, and complete cleanup.
6. Run focused tests, Runseal init, and the merge-checkpoint repository guard.

## Controlled Variables

- `NativeMessage`, Win32 key/system-key/focus capture, queue-drain order, key range, repeated-down/
  unmatched-up suppression, focus cleanup, and empty-ingest edge expiry remain unchanged.
- One prototype `HostInput` remains live across bootstrap. W/A/S/D uses held state and Escape uses
  the press edge exactly as accepted by Experiment 0084.
- Bootstrap source publication, readiness order, runtime frame transaction, actor simulation,
  camera, traversal, playable bounds, and process lifecycle remain unchanged.
- Workbench rendering/inspect commands unrelated to input, Sidecar manifests, and wrapper set remain
  unchanged.
- Renderer, GPU resources, shaders, synchronization, source/pack formats, assets, and Wulin paths
  remain unchanged.

## Metrics

- Removed live files, methods/types, inspect verbs, wrapper commands, canonical report fields, and
  source lines.
- `HostInput` resident size/drop requirement and focused normalized-state test outcomes.
- Exact unsupported-event and unknown-command results for retired live surfaces.
- Native-W command/presentation/displacement, Escape exit, finite-edge hold, process restart, and
  zero-process evidence from the focused product workflow.
- Guard deliberate-negative/positive results and Flavor deny/warning counts.

## Acceptance Criteria

- Live code contains no active/completed recording, transition journal, counter/capacity/fault,
  status JSON, hash/revision, replay, `PostedMessage`, or native-post implementation.
- The five old inspect verbs and four old workbench commands fail immediately. The long canonical
  report and support graph contain no `hostInput` compatibility field or gate.
- `HostInput` owns exactly fixed held/pressed/released state with no heap-owning/drop type. Focused
  tests preserve all product-relevant normalization and edge semantics.
- Workbench owns no persistent `HostInput` after bootstrap. Prototype retains one owner across
  bootstrap and reproduces pre-ready W movement plus Escape press-edge exit in the real process.
- ADR 0044 is superseded, historical Experiment 0041 remains unchanged, and the maintained removal
  guard rejects deliberate reintroduction before expensive checks.
- No alias, fallback, replacement journal, new operator, feature, engine/runtime/GPU/source/asset/
  Wulin change, or generated output is added.
- Focused tests, retired-surface process evidence, `runseal :canonical-prototype`, `runseal :init`,
  and `runseal :guard` pass. The long canonical-runtime workflow is not required because only its
  obsolete diagnostic gate/report field is deleted and no renderer/resource/synchronization/
  lifecycle implementation changes.

## Reference Environment

The experiment uses the repository-pinned Windows/Rust/Deno toolchains, real Win32 workbench and
prototype windows, Sidecar process ownership, and existing focused product workflow.

## Evidence

Commands:

```powershell
cargo test -p reference-host -p prototype
runseal :canonical-prototype
runseal :init
runseal :guard
```

The audit found one closed diagnostic chain with no product reader: journal state/counters/hashes,
five workbench inspect verbs, reverse native-message posting, four wrapper commands, one long-gate
support owner, and one canonical report field. It also found a required preservation dependency:
the native-W gate posts before readiness, so prototype must retain its input owner across hidden
bootstrap. Workbench needs only a temporary input owner during configured bootstrap and now drains/
discards later native messages.

Tracked Rust/TypeScript implementation and tests deleted 814 lines and added 66; the maintained
93-line removal guard brings the live implementation/test net to minus 655 lines. `HostInput`
shrunk from 414 to 94 physical lines and owns exactly three 256-bit sets: 96 bytes on the reference
platform with `needs_drop == false`. Its focused suite is three tests; the complete reference-host
suite now has 20 passing tests, and all 16 prototype tests pass.

The dependency-free `0085-retired-input-surface-gate-v1` real-process witness passed in
11,984.418 ms. `input.status`, `input.record.start`, `input.record.stop`, `input.replay`, and
`input.native.post` each returned exit 1 with `unknown_event`; `runseal :workbench input`,
`input-record-start`, `input-record-stop`, and `input-replay` each returned exit 1 with
`unknown command`. No retired call returned an empty result, alias, or redirect.

A deliberate `input.status` insertion in the live workbench wrapper made the new removal guard fail
in 0.7 seconds at `removed diagnostic input journal surface`, before Rust build/test work. Removing
the injected line restored the clean source.

`canonical-prototype-v10` passed in 75,044.572 ms with 77 engine-runtime, 16 prototype, and 20
reference-host tests. The process receiving native W before readiness still published
`deltaZQ9=-32`, Walk clip 1, and exact moved actor evidence. The native Escape process exited 0 in
4,136.619 ms with empty stderr, and the held-W boundary process remained live for 15,007.519 ms.
Startup failures, stationary/restart equality, presentation, camera, traversal, backpressure,
Sidecar restart, and zero-process cleanup also passed.

`runseal :init` passed with the pinned toolchain and updated required-file inventory. The long
canonical-runtime workflow was not rerun: its obsolete diagnostic input gate/report field is gone,
while runtime, renderer/GPU, resources, synchronization, source formats, and lifecycle
implementation are unchanged. The merge-checkpoint `runseal :guard` passed with zero Flavor denies
and five pre-existing warnings.

## Conclusion

Accepted. The diagnostic journal and every route that existed only to operate or verify it are no
longer live. Host input is fixed product state, workbench cannot synthesize or replay it, and real
pre-ready locomotion plus Escape behavior remain exact.

## Promotion

Promoted only the fixed normalized state owner and removal guard. ADR 0088 supersedes ADRs 0044 and
0087, while Experiments 0041 and 0084 remain settled evidence. No replacement diagnostic or
operator was promoted.
