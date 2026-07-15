# Experiment 0079: Self-Contained Prototype Operator

Status: Accepted

## Hypothesis

One maintained `runseal :prototype` operator can cold-start the plain prototype by deterministically
cooking a documented finite sandbox, writing its strict bootstrap, and owning the existing Sidecar
lifecycle, so manual use no longer depends on canonical-acceptance residue or copied cooker
commands.

## Scope

Add `.runseal/wrappers/prototype.ts` as the sole manual prototype entry point with
`start|restart|stop|status`. `start` requires the owned Sidecar target to be stopped, cooks terrain
and base-presentation object packs for every global center in the fixed square `[-8,8]²`, writes
schema-1 active-radius-2 bootstrap at zero origin/center, publishes concise setup evidence, then
starts the existing `sidecar.prototype.toml`. Restart, stop, and status perform lifecycle work only.

The 289 centers expand and deduplicate to 441 source regions. The fixed range is an explicit
sandbox boundary, not a streaming-service or infinite-world claim. Source/config output remains
ignored under `out/cooked/prototype/` and `out/cooked/bootstrap/runtime.json`.

Replace direct Sidecar/manual-preparation instructions in the live README and AGENTS workflow with
the wrapper. Keep Sidecar as the underlying process owner; do not add an inspect endpoint, alternate
prototype mode, test flag, or compatibility alias.

Runtime/prototype behavior, actor/input/time/presentation/camera/traversal policy, renderer/shader/
GPU resources, synchronization, cooker/pack formats, canonical acceptance, and Wulin content are
out of scope.

## Workload

1. From stopped state, run `runseal :prototype start` with no prepared prototype source/config and
   require deterministic terrain/object cook, exact bootstrap, canonical readiness, and live owned
   process status.
2. Require a second `start` while running to fail before source/config mutation.
3. Run `restart`; require a new prototype PID set with unchanged config and source hashes.
4. Stop and require no owned prototype/broker processes. Cold-start again and require byte-identical
   config and source hashes, then stop cleanly.
5. Require wrapper help/argument rejection, exact maintained wrapper registration, Deno checks,
   init, and guard.
6. Remove the live direct-Sidecar/manual-bootstrap operator instructions without retaining an old
   command alias or fallback workflow.

## Controlled Variables

- The wrapper calls the existing release cookers and existing Sidecar manifest. It does not
  implement content generation, bootstrap parsing, readiness, or process ownership itself.
- Center enumeration is deterministic Z-major then X-major over inclusive `[-8,8]`; both cookers
  receive the exact same centers and active radius remains 2.
- Global origin and center are `(0,0)`. Terrain/object paths, physical order `a`, presentation
  profile `base`, config path, and sandbox half-extent are fixed.
- Restart/stop/status never recook or rewrite bootstrap. Start refuses to prepare while a target or
  broker is running.
- Generated sources/config remain disposable and unversioned.

## Metrics

- Requested center count, cooked region count, pack bytes, file SHA-256, bootstrap bytes/hash, and
  setup elapsed time.
- Start/restart/stop status, prototype PID sets, broker state, and readiness result.
- First/second cold-start config and source hash equality.
- Wrapper/help/invalid/running-start rejection behavior and pre-mutation equality.
- Runtime/application/GPU/synchronization/format/canonical acceptance deltas.

## Acceptance Criteria

- A fresh `runseal :prototype start` alone produces valid deterministic sources/bootstrap and a
  canonical-ready visible prototype; no prior acceptance run or manual cooker command is required.
- The setup reports 289 centers and 441 regions with exact stable paths/hashes. A second cold start
  reproduces byte-identical config and source hashes.
- Running-start rejection preserves all source/config hashes. Restart replaces live process IDs;
  stop leaves zero owned process IDs.
- README and AGENTS expose only the wrapper for manual prototype lifecycle. The existing Sidecar
  manifest remains an implementation dependency, not a second documented operator route.
- `runseal :init` and `runseal :guard` pass. No compatibility alias, inspect/test mode, runtime or
  prototype-loop change, renderer/shader/GPU resource, synchronization, cooker/format, traversal,
  or canonical acceptance behavior change remains.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference host, existing deterministic
cookers, strict bootstrap parser, Sidecar prototype manifest, and sole runtime/renderer.

## Evidence

- With the prior manual source directory and bootstrap removed, the first
  `runseal :prototype start` cooked and reached Sidecar readiness in 11.8 seconds. Status reported
  broker PID 32080 and prototype process set `23920/20988/32808`.
- The deterministic sandbox contained 289 requested centers and 441 cooked regions. Terrain was
  1,835,008 bytes with SHA-256 `b93391dd...0b97`; objects were 18,092,032 bytes with SHA-256
  `ca1f2163...5186`; the 240-byte strict bootstrap SHA-256 was `49353733...3fe` and selected zero
  origin/center, active radius 2, and only the operator source paths.
- A second start while running failed before cook/config work with
  `stop the running prototype before preparing a fresh sandbox`; all three hashes remained exact.
- `restart` replaced the prototype process set with `1092/23480/11616` and broker PID 32996 without
  changing sources/config. Stop left empty broker and target PID sets.
- A second stopped-state start reached readiness in 11.1 seconds with broker PID 10828 and
  prototype process set `6812/30792/8780`; all three hashes reproduced exactly. Final stop again
  left zero owned processes.
- Help succeeded and extra arguments failed without mutation. Deno formatting/type checking,
  `runseal :init`, and `runseal :guard` passed. Guard reported zero Flavor denies and the five
  existing warnings.
- The live manual documentation now uses only `runseal :prototype`; direct Sidecar/manual bootstrap
  preparation is no longer an operator workflow. Runtime/application/GPU/canonical behavior did
  not change.

## Conclusion

The hypothesis is accepted. The plain prototype now has one self-contained maintained operator:
`start` deterministically prepares its finite sandbox and strict bootstrap before existing Sidecar
readiness, while restart/stop/status retain lifecycle-only semantics. Running-start rejection and
two cold starts prove pre-mutation safety and exact reproducibility. No acceptance residue, copied
cooker sequence, direct Sidecar operator route, compatibility alias, inspect/test mode, runtime or
prototype-loop change, renderer/shader/GPU resource, synchronization, cooker/format, traversal, or
canonical acceptance change remains.
