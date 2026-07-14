# Experiment 0039: Canonical Runtime Host Separation

Status: Accepted

## Hypothesis

The accepted canonical runtime can be promoted from workbench-private modules into one reusable
engine crate behind a narrow runtime facade without changing rendering, streaming, presentation,
control, failure, resource, or lifecycle behavior and without inventing a platform abstraction or
a second runtime path.

## Scope

This experiment separates proven runtime ownership from the existing workbench host. The promoted
runtime owns scene/world state, canonical source streaming and composition, rendering, GPU device
and resource lifetime, presentation time, shader compilation, and the D3D12 Agility binding. One
facade owns both the current renderer and scene state and exposes only the operations already used
by the workbench frame loop and compact control vocabulary.

The workbench remains the only executable and retains the Win32 window/message pump, Sidecar and
inspect transport, operator capture persistence, perception request/response shaping, process
readiness, and console lifecycle. It passes the native window handle and frame requests into the
runtime and does not assemble renderer-internal state itself.

Native input, automatic source loading, a prototype executable, simulation timing, player or
actor state, new visuals, resize/fullscreen work, portability, a general engine API, compatibility
wrappers, and Wulin content are out of scope.

## Workload

1. Record the current module dependency direction and exact controlled attachment/shadow evidence.
2. Create one engine-owned runtime crate and move the accepted `load`, `resident`, `world`,
   `scene`, `streaming`, `rendering`, shader, and GPU build ownership into it. Preserve the current
   Windows/D3D12 reference-platform contract.
3. Introduce one facade that owns renderer and scene state. Adapt the workbench main loop and
   inspect handlers to call that facade without changing verbs, payloads, responses, frame order,
   or capture persistence.
4. Prove the crate dependency points from `apps/workbench` to the runtime and never back to apps,
   host modules, experiments, or mods. Prove there is only one renderer, shader set, source
   streamer, composition owner, and presentation clock.
5. Run focused build/tests, the repository guard, and the complete direct canonical GPU workflow,
   including failure gates, traversal/prefetch/rollover, the 64-publication plateau, and 16
   lifecycle cycles.

## Controlled Variables

- Signed schema-3 presentation authority, exact `i64` identity, pack formats, 50-slot residency,
  terrain-first publication, grounding, LOD, occlusion, surface/shadow execution, and all hashes
  remain unchanged.
- The renderer frame order, root signatures, descriptors, shaders, fixed submissions, object-ID
  semantics, presentation clock, and source-duration mapping remain byte-for-byte or
  behavior-for-behavior identical.
- The compact inspect vocabulary and `runseal :canonical-runtime` stay direct and non-recursive.
- The reference platform remains native Windows with D3D12 Agility. No cross-platform host or
  graphics abstraction is introduced.
- Generated outputs, caches, and captured evidence remain ignored.

## Metrics

- Runtime-owned and host-owned file sets, dependency direction, facade operations, forbidden
  imports, renderer/shader/source-owner counts, and workbench source ownership after promotion.
- Controlled color, PNG, object-ID, diagnostic, light-matrix, and shadow-depth SHA-256 values.
- Visible/caster/LOD/pose counts, CPU/GPU oracle mismatches, fixed terrain/skeletal submissions,
  root cost, descriptor count, and content I/O/copy/publication counters.
- Failure/rollback results, traversal continuity, device state, handle/private-byte plateau,
  thread count, and descendant cleanup.

## Acceptance Criteria

- `apps/workbench` depends on one promoted runtime crate. The runtime crate has no dependency or
  source-path reference back to `apps/`, `mods/`, `experiments/`, inspect transport, capture
  persistence, perception protocol, or window-message ownership.
- One facade owns the renderer and scene state. The workbench does not retain private copies or
  directly compose streaming/rendering subsystems. Exactly one renderer implementation, shader
  set, canonical source pair, composition owner, and presentation clock remain live.
- The controlled baseline remains exact: color
  `8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135`, PNG
  `e96e44cc6c7cf05338433a05568e2a41e81f95f2f5ba8c52ce7baa26114450c6`, object-ID
  `01951615d1b4645bdfba68991c75b8ea333482d312f31f39ed3b907ca479da5b`, diagnostic
  `5f6f2f195d9deadfc4db905692d22e805b4e7000f102537ad36a2e01bd319855`, light matrix
  `480ef3365b258ea2a93b21942a800bfdc21d8d1f6241c45ef36fd2d5fa41fd65`, and shadow depth
  `2415cfdd82a769056d4e91e4a6575de1a5f8628a7fedc5b630af00569d1233d5`.
- The controlled frame still has 10,538 shadow casters, one shadow dispatch, 60 surface constant
  DWORDs, 98 descriptors, six fixed skeletal submissions, zero sample mismatch, and unchanged
  terrain/object attachments. All source-duration, reorder, revisit, alias, failure, hold, and
  rollover evidence remains exact.
- The 64-publication run has no transient handle growth after settling, all 32 reactive and 32
  prepared crossings pass, all 16 lifecycle cycles clean up, no device removal or validation
  error occurs, and the complete repository guard passes.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0039-canonical-runtime-host-separation/`.

## Results

The 579.4-second direct GPU workflow passed. The controlled color, PNG, object-ID, diagnostic,
light-matrix, and shadow-depth hashes exactly matched the six pre-migration values fixed in the
acceptance criteria. The frame still used 10,538 shadow casters, one shadow dispatch, 60 surface
constant DWORDs, 98 descriptors, six fixed skeletal submissions, and zero sampled shadow mismatch.

The accepted runtime, shaders, private tests, shader compilation, and Agility binding moved into
`crates/engine-runtime`. One `Runtime` facade now owns the sole `Renderer` and `SceneState` and
contains the existing frame, control, fault-gate, and observability operations. The workbench host
is reduced to 11 source files and 59,157 bytes covering its window/message loop, inspect transport,
capture persistence, perception shaping, and process readiness. A repository guard rejects legacy
runtime paths under the host, reverse host imports, a workbench dependency in the runtime tree, or
more than one renderer owner.

All 18 runtime-private tests passed from the promoted crate. An initial repository guard correctly
rejected missing public `unsafe` safety contracts and Flavor exceptions that still named the old
app-owned paths. The accepted change documents the native-thread/window invariants and moves the
existing exceptions to the new owner without disabling rules or increasing thresholds.

All 32 reactive and 32 prepared crossings, source/time/reorder/alias/hold/failure/rollback gates,
rollover, and 16 complete lifecycle cycles passed. The 64-publication run settled at 531 handles,
413,147,136 private bytes, and 22 threads; peak handles remained 531 with zero transient growth.
The final sample had 516 handles, 411,627,520 private bytes, and 18 threads.

## Conclusion

Accepted. The canonical renderer, scene/world state, source streaming, composition, presentation
time, shaders, and GPU lifecycle now have one reusable engine owner. The workbench is a host of
that runtime rather than its composition root. The promotion changed ownership and public safety
contracts but did not change renderer behavior, assets, content authority, controls, submissions,
attachments, resources, or lifecycle outcomes.

## Promotion

Promoted the accepted runtime into `crates/engine-runtime` and added a stable repository gate for
the `apps -> crates` direction and single renderer owner. Native input, automatic bootstrap,
simulation stepping, a prototype host, portability, and a generalized application API remain
unproven and are not part of this decision.
