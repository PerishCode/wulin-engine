# Experiment 0045: Active Compatibility Removal

Status: Accepted

## Hypothesis

The accepted runtime can delete its pre-canonical calibration scene, split-world controls, and
their rendering pipeline while retaining a clear-only idle shell and the exact canonical runtime,
host, query, failure, resource, and lifecycle behavior. Shared depth and semantic attachments can
move to a neutral frame-target owner without preserving the deleted calibration abstraction.

## Scope

This experiment removes the live `scene.list_objects` and `world.*` inspect vocabulary, the
calibration objects and spatial probes behind it, `SplitPosition`/`WorldSpace`, the calibration
shader and draw pipeline, and runtime methods whose only caller is that surface. `RegionCoord`
moves to a narrow current owner. Canonical semantic lookup moves into the existing ID-mapping
owner, and the canonical depth/semantic render targets move out of the calibration renderer.

The idle shell remains a valid clear-only frame outcome for the diagnostic workbench. Camera
status/set/reset, signed global addressing, canonical publication and traversal, semantic capture,
the exact terrain query, and prototype bootstrap remain current behavior.

The fixed eight-slot animation bank is explicitly retained. The imported source's three clips are
cooked into the current eight-slot region/GPU presentation contract; changing that bank is a
separate format, shader, cooker, and evidence problem rather than compatibility cleanup. Strict
bootstrap tests containing a rejected `fallback` field, current projection aliases, and settled
historical experiment/ADR text are also retained because none is an executable compatibility path.

## Workload

1. Inventory every live calibration/world symbol, caller, shader stage, public runtime method,
   inspect verb, test, Flavor override, and stable workflow reference. Record a closed deletion and
   retention list before implementation.
2. Replace the calibration renderer's shared depth and `R32_UINT` semantic attachments with one
   neutral frame-target owner. Clear both attachments during idle frames and feed the same handles
   and resource to the unchanged canonical passes and capture readback.
3. Reduce `SceneState` to current camera ownership, move canonical semantic lookup to the existing
   ID-mapping owner, move `RegionCoord` out of the split-world module, and delete the calibration scene,
   renderer, shader, world controls, and their tests.
4. Require all deleted inspect verbs to fail as unknown. Capture an idle frame through the semantic
   path and require a uniform clear color, all-zero semantic attachment, no visible/unknown semantic
   object, current camera-space metadata, and no calibration/world/object inventory fields.
5. Add a repository guard that rejects reintroduction of the deleted files, verbs, types, methods,
   shader, revision, and renderer owner in active source while excluding settled documentation.
6. Run focused tests, `runseal :guard`, then the complete direct canonical GPU, prototype, query,
   failure, traversal, resource-plateau, restart, and 16-cycle lifecycle workflow.

## Controlled Variables

- Signed schema-3 objects, signed terrain packs, 50-slot terrain residency, triple-plane object
  residency, terrain-first atomic publication, traversal/prefetch/rollover, and timeline ownership
  remain unchanged.
- Canonical color, depth, semantic ID, visibility, skeletal, surface, occlusion, shadow, and query
  work remain in the sole renderer. No second runtime mode, fallback, or replacement scene exists.
- The idle shell clears the swap-chain color, shared reverse-Z depth, and semantic attachment but
  submits no scene geometry and owns no semantic objects.
- `RegionCoord` retains its exact signed `i64` representation and public API. The move changes only
  source ownership; no conversion or legacy re-export layer is added.
- The current eight-slot animation bank and imported source-to-bank cook remain byte-for-byte
  unchanged. No format, cooker, shader, payload, or presentation change is part of this experiment.
- Historical ADRs and experiment reports remain immutable decision history and are not scanned as
  live compatibility behavior.

## Metrics

- Deleted active files, source lines, shader stages, inspect verbs, public runtime methods, static
  calibration objects, and compatibility-symbol scan hits.
- Idle color `differentPixelCount`, semantic raw SHA-256, visible semantic count, unknown semantic
  count, spatial metadata shape, renderer error, and device-removal outcome.
- Exact controlled canonical color/PNG/object-ID/diagnostic, shadow, query result, and query identity
  hashes before and after cleanup.
- Same-process baseline/peak/final handles, private bytes, threads, transient growth, and complete
  lifecycle cleanup.

## Acceptance Criteria

- Active source contains none of the calibration shader/renderer/object inventory,
  `SplitPosition`/`WorldSpace`, calibration revision/mode, deleted runtime methods, or deleted inspect
  verbs. The guard fails if any returns, and no alias or wrapper preserves them.
- The idle workbench reports only `idle-shell`, captures exactly its configured clear color, and
  produces an all-zero semantic attachment with zero visible and zero unknown semantic objects.
  It exposes current camera/coordinate/depth metadata without a calibration scene, render camera,
  world state, or object inventory.
- Canonical publication uses the neutral frame targets directly. Semantic capture still recognizes
  terrain-region and region-proxy IDs, and every accepted controlled attachment hash remains exact.
- `RegionCoord` API and signed behavior, fixed eight-slot animation bank, bootstrap/prototype
  readiness, terrain query hashes, hold/rollback behavior, 32 reactive plus 32 prepared crossings,
  resource plateau, restart, and 16 lifecycle cycles remain accepted.
- Focused tests and `runseal :guard` pass. The direct GPU workflow completes without validation or
  device-removal error and does not increase settled or peak resource counts.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain, reference Windows host, D3D12 Agility
SDK and DXC configured by `runseal :init`, and the same reference adapter selected by the canonical
workflow.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0045-active-compatibility-removal/`.

## Results

The 645.2-second direct workflow passed. The active source/configuration diff deleted eight files,
six inspect verbs, seven obsolete `Runtime` methods, two split-world types, eight static scene
objects, and two shader stages. It removed 1,655 tracked lines while adding 424 active lines for the
neutral frame targets, signed-region owner, acceptance gate, and narrow call-site updates, for a net
removal of 1,231 active lines. The repository guard found none of the retired paths or symbols.

The idle shell reported `canonical-camera-space-v1` metadata with only camera, coordinate, depth,
and revision fields. Its 1280x720 capture contained 921,600 background semantic values, zero
visible or unknown semantic IDs, and zero pixels different from RGBA `[9, 27, 36, 255]`. The color
hash was `cd26eaab8a2f1f4aa8ac5c7bcfe824d509f3d6da0f7a6fbae4798eec90e76db8`; the
all-zero semantic hash was
`0c660f2bd3eff3150dd0040789abe2291613b9af319df870203d4f77a4913a5f`. All six removed
verbs failed through `unknown_event` without an alias or dispatcher.

The controlled canonical color, PNG, object-ID, and diagnostic hashes remained
`8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135`,
`e96e44cc6c7cf05338433a05568e2a41e81f95f2f5ba8c52ce7baa26114450c6`,
`01951615d1b4645bdfba68991c75b8ea333482d312f31f39ed3b907ca479da5b`, and
`5f6f2f195d9deadfc4db905692d22e805b4e7000f102537ad36a2e01bd319855`.
The shadow matrix/depth and terrain-query result/identity hashes also remained exact, including
zero query-oracle mismatch through every fault, traversal, restart, and lifecycle gate.

The resource workload settled and peaked at 527 handles with zero transient growth. Publication
64 ended at 516 handles, 408,723,456 private bytes, and 18 threads. All 32 reactive crossings, 32
prepared crossings, and 16 complete lifecycle processes passed with no validation or
device-removal error.

## Conclusion

Accepted. The pre-canonical calibration scene and split-world operator surface were active
compatibility debt and are now absent. The diagnostic idle shell is intentionally clear-only, and
canonical depth/semantic attachments belong to neutral frame targets. Signed global identity,
camera state, semantic capture, and the fixed eight-slot animation bank remain current contracts;
none is a compatibility alias.

## Promotion

Promoted neutral depth/semantic attachment ownership into
`crates/engine-runtime/src/rendering/frame_targets.rs`, signed region identity into
`crates/engine-runtime/src/region.rs`, and canonical semantic lookup into the existing ID-mapping
owner. Added a stable guard against the retired surface and direct idle/unknown-verb evidence to
the sole canonical workflow.
