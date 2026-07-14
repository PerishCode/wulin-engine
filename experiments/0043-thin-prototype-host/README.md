# Experiment 0043: Thin Prototype Host

Status: Accepted

## Hypothesis

The accepted reference-platform window, normalized input, and canonical bootstrap mechanisms can
be promoted into one concrete shared host boundary and consumed by both the diagnostic workbench
and a new non-diagnostic prototype executable, without duplicating runtime/rendering behavior or
changing any accepted frame, failure, resource, or lifecycle result.

## Scope

This experiment promotes only the already accepted Win32 single-window/message lifecycle,
keyboard/focus normalization and bounded process-local journal, strict schema-1 argument/config
parser, pack-path validation, and hidden canonical bootstrap driver. The existing workbench
consumes that boundary while retaining inspect, capture/perception persistence, pause, operator
mutation, fault injection, and evidence shaping.

One new `apps/prototype` composition root requires a bootstrap document, creates one reference
window, waits hidden for canonical readiness through the same driver, then continuously renders
the same `engine-runtime`. Its only normalized input action is held Escape requesting host exit.

Cross-platform traits, another graphics/runtime path, resize, multiple windows, camera actions,
mouse/gamepad, elapsed or fixed simulation time, terrain queries, player state, locomotion,
actors, interaction, UI, saves, content discovery, Wulin content, and compatibility behavior are
out of scope.

## Workload

1. Move the accepted workbench window, input, strict bootstrap parser/path validation, and hidden
   bootstrap loop into one reference-platform host crate. Require the workbench to consume it
   without retaining shadow implementations.
2. Run all focused argument, config, input normalization, bounded journal, and replay tests at the
   new owner; retain exact canonical input hashes in the process-restart gate.
3. Launch the prototype with invalid, missing-source, and requested corrupt-payload bootstrap
   documents. Require nonzero exit before the prototype readiness role and complete cleanup.
4. Launch and restart the prototype with freshly cooked valid sources and a far signed target.
   Require canonical-only readiness through the promoted driver, the same config hash and target,
   a live window/process, and clean Sidecar stop.
5. Launch the configured and ordinary workbench lifecycles. Require the accepted canonical-ready
   and idle-shell-ready behavior respectively, including the unchanged inspect surface.
6. Run the complete canonical GPU correctness, failure, traversal, resource, and lifecycle gates.

## Controlled Variables

- `crates/engine-runtime`, the runtime facade, renderer, scene/world, presentation timeline,
  source formats, async streaming, pair publication, traversal, shaders, GPU resources, and frame
  transaction remain unchanged.
- The shared host owner remains specific to the accepted Windows reference platform and one
  process-local window. It does not define backend, OS, window-system, input-device, or
  application-framework interfaces.
- Bootstrap schema, path restrictions, 64-KiB bound, exact source/target semantics, 120-second
  host timeout, hidden progress, and terminal no-ready failure remain unchanged.
- Input transaction ordering, virtual-key domain, repeat/unmatched suppression, focus cleanup,
  record bounds, canonical hashes, and isolated replay remain unchanged.
- Prototype startup is always configured and has no idle shell, inspect endpoint, diagnostic
  command path, capture writer, or fault gate. Escape affects only process lifecycle.

## Metrics

- Physical owner and dependency graph for reference-host, workbench, prototype, and engine-runtime
  modules; forbidden duplicate/reverse-dependency scan results.
- Focused input/bootstrap test counts and exact accepted input stream/held-state hashes.
- Invalid, missing, corrupt, first-valid, and restarted prototype exit/readiness outcomes,
  configuration hash, signed target, hidden ready-frame count, elapsed host duration, and residual
  process count.
- Existing controlled attachment/shadow hashes, CPU/GPU oracle mismatch counts, traversal,
  resource plateau, and lifecycle evidence.

## Acceptance Criteria

- Workbench and prototype depend on one promoted reference-host owner. No copied window procedure,
  input normalizer, config decoder, pack validator, or canonical bootstrap driver remains under an
  app, and engine-runtime has no host-policy dependency.
- Workbench ordinary/configured startup, inspect controls, deterministic input/replay, and all
  evidence remain behavior-exact after the move.
- Prototype requires a valid bootstrap document and emits its readiness role only after the shared
  driver completes a canonical frame. Immediate and async failure emit no readiness and never
  fall back to an idle shell.
- Valid prototype restart reports the exact config digest, target, and canonical-ready contract;
  stop/close leaves no prototype, cargo, broker, or inspect process. The prototype exposes no
  workbench command endpoint.
- Focused tests, repository guard, all prior canonical correctness/failure/traversal/resource
  gates, and 16 complete lifecycle cycles pass with unchanged GPU hashes and no device-removal or
  validation error.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0043-thin-prototype-host/`.

## Results

The 587.5-second direct GPU workflow passed. The prototype rejected an unknown-field document in
13.0 ms, a missing object source in 3,409.4 ms, and a requested corrupt object payload in 3,352.8
ms. All three exited nonzero without emitting the prototype readiness role; the corrupt pair
reported the exact signed global-region checksum failure and discarded its terrain half.

The first direct prototype process reached canonical-ready on hidden frame 8 after 79.1 ms, and a
fresh process did so on frame 9 after 90.8 ms. Both emitted the same schema/config invariant,
signed target, token-1 schedule, exact source namespaces, and configuration hash
`a0ffa4e534399b90f614825df4ba5e052ed265e4ce388600ec7f77e9d2848438`.
The separate no-inspect Sidecar lifecycle started, replaced, and stopped the complete prototype
process set; final runtime and target PID lists were empty.

The accepted window, input, journal, path/config, and bootstrap mechanisms now have one physical
owner under `crates/reference-host`. Ten focused tests passed there, including a read-only held-key
consumer. Workbench native replay retained stream hash
`ec86601874cb60a8c592b9caf500da94111b6a7360647d316ce1e858b55de435`, and ordinary/configured
workbench lifecycles retained their idle-shell/canonical readiness and inspect behavior.

All six controlled GPU hashes remained exact; the controlled color hash stayed
`8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135`.
Walk ticks 0/42/43/85 retained phases 0/63/0/0 and maximum imported palette delta
`2.3283064e-10`. All failure, hold, rollback, alias, rollover, 32 reactive, and 32 prepared gates
passed. The resource workload settled at 527 handles, peaked at 527 with zero transient growth,
and publication 64 ended at 516 handles, 413,556,736 private bytes, and 18 threads. All 16 complete
lifecycle cycles, the repository guard, and final four-namespace cleanup passed.

## Conclusion

Accepted. The repository now has a plain, non-diagnostic prototype composition root over the same
canonical runtime and one concrete reference-platform host boundary shared with the workbench.
The prototype proves configured application ownership, visible canonical startup, continuous
rendering, and host close/Escape lifecycle; it does not yet prove camera movement, simulation,
terrain contact, actors, or gameplay interaction.

## Promotion

Promoted the accepted reference window/message lifecycle, normalized input/journal, strict
bootstrap parser/path validation, and hidden canonical bootstrap driver into
`crates/reference-host`; migrated workbench to that owner; added `apps/prototype`, one no-inspect
Sidecar lifecycle, direct prototype gates, and stable ownership scans. No renderer, platform
traits, input-to-camera mapping, simulation clock, actor layer, or compatibility path was added.
