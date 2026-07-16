# Experiment 0083: Explicit Playable Region Boundary

Status: Accepted

## Hypothesis

The finite Prototype v0 sandbox can have exact, non-fatal product edge behavior without weakening
the runtime's strict source and published-window contracts: one strict bootstrap rectangle can own
the playable global-region extent, and prototype locomotion can reduce only an axis whose maximum
admitted fixed-step batch could leave that extent before invoking the existing runtime transaction.

## Scope

Replace live bootstrap schema 1 directly with schema 2 and one inclusive
`playableRegionBounds` rectangle whose minimum and maximum are signed global region coordinates.
Require a non-empty rectangle containing the configured initial global center. Expose that exact
validated value from `reference-host::bootstrap::Plan` and include it in pending/ready evidence.

Before constructing each live actor command, the prototype reads the current authoritative actor
and tests each requested planar axis independently at the existing maximum eight steps per
advance. An axis whose maximum candidate leaves the declared rectangle becomes zero; the other
axis, gravity, schedule, presentation policy, camera, and complete runtime transaction remain live.

The maintained product operator declares inclusive `[-6,6]²` playable bounds inside its cooked
`[-8,8]²` center horizon. Canonical support declares only the region used by each focused setup.
Do not infer bounds from pack indexes, add a schema-1 decoder, create an engine collision/boundary
system, swallow source failures, add product telemetry or inspect commands, expand the cooked
horizon, or change renderer/GPU/source-format/Wulin behavior.

## Workload

1. Retain the pre-change real release-process witness: one minimal cooked center, schema 1,
   explicit native activation, and held W input must exit through the strict terrain-query failure.
2. Sweep schema-2 decoding across signed rectangles, reversed/empty axes, initial-center exclusion,
   unknown fields, schema 1, projection overflow, and exact pending evidence.
3. Sweep product boundary reduction at both local edges, exact maximum-batch contact, one-Q9
   crossing, inward motion, independent diagonal axes, signed far regions, and translation
   overflow. Require no runtime query or mutation in this pure policy.
4. Through `runseal :canonical-prototype`, launch the real prototype with a one-region playable
   rectangle, explicitly activate its native window, hold W for at least 15 seconds, require the
   process to remain live beyond the 11.282-second pre-change failure, and then terminate it under
   evidence ownership with no stderr failure.
5. Run the existing startup, failure, stationary/native-W movement, presentation/epoch, camera,
   traversal, restart, and cleanup gates plus the merge-checkpoint guard.

## Controlled Variables

- `SIMULATION_MAX_STEPS_PER_ADVANCE` remains eight and is the sole maximum-batch authority.
- Cardinal/diagonal Q9 displacement, fixed gravity, step-up limit, rational schedule, actor storage,
  transactional admission, render backpressure, camera rig, and traversal behavior are unchanged.
- Runtime terrain query, pending/published-window, missing-source, and atomic rollback failures stay
  strict. The engine does not consume or infer the product rectangle.
- Bootstrap paths, active radius, signed origin/center projection, 64 KiB document limit, and
  deny-unknown-fields behavior remain exact apart from the direct schema replacement.
- The maintained manual source horizon remains zero-origin `[-8,8]²`, with prefetch disabled.
- No shader, GPU resource, copy, synchronization, capture, pack format, asset, or Wulin path changes.

## Metrics

- Exact accepted/rejected schema cases and serialized playable rectangle evidence.
- Per-axis requested/admitted Q9 deltas at maximum batch, exact edge positions, and overflow result.
- Native activation/key evidence, hold duration, premature-exit state, stderr tail, owned process
  cleanup, and the prior strict-failure duration/error.
- Focused test counts, canonical-prototype revision/outcome/duration, Flavor denies/warnings, and
  final diff ownership.

## Acceptance Criteria

- Schema 2 is the only accepted live bootstrap document and rejects unknown fields, invalid signed
  rectangles, and an initial center outside the inclusive extent before source/runtime work.
- Product locomotion preserves a safe requested axis exactly and sets only an unsafe axis to zero
  using the current actor plus maximum eight-step candidate; presentation follows the admitted
  command and gravity/schedule/runtime transaction still execute.
- The activated real prototype remains live for at least 15 seconds under held W at the one-region
  playable edge, then exits only through evidence cleanup with no application failure.
- Existing canonical-prototype startup, movement, animation, camera, traversal, restart, failure,
  backpressure, and zero-process gates pass with strict runtime errors unchanged.
- No schema-1 fallback, source-index inference, engine boundary mode, compatibility alias, product
  diagnostic surface, renderer/GPU/source-format/asset/Wulin change, or generated output is added.
- `runseal :guard` passes. A long canonical-runtime rerun is not required because no renderer, GPU,
  resource, synchronization, source format, or engine failure contract changes.

## Reference Environment

The experiment uses the repository-pinned Windows/Rust/Deno toolchains, Win32 native input,
Sidecar process ownership, deterministic cookers, and the existing canonical-prototype workflow.

## Evidence

The dependency-free pre-change witness cooked one minimal center, launched the release prototype,
explicitly activated its real window, held W, and reproduced exit code 1 after 11.282 seconds with
`terrain-body motion batch step 1 of 2 failed: terrain query region is outside the published active
window`. The earlier non-activated hidden-window attempt timed out and is excluded from evidence.

Schema-focused tests now total 23 reference-host tests, including exact signed bounds evidence,
ordered-axis and initial-center rejection, schema-1 rejection, unknown-field/path/projection
preservation, and signed extrema. Prototype-focused tests now total 16, including four new exact
maximum-batch boundary-policy cases. All passed with the existing 77 engine-runtime tests.

`canonical-prototype-v9` passed in 71.271 seconds. Its real boundary process received explicit
native activation plus W through `prototype-native-key-v2`, remained live for 15,002.745 ms against
the required 15,000 ms, emitted no stderr, and was terminated only by evidence cleanup. Existing
startup/failure, stationary and moving actor, Survey/Walk/facing/epoch, camera, traversal,
render-block, restart, and zero-process Sidecar gates passed in the same workflow.

The maintained `prototype-operator-v2` recooked the exact 289 centers / 441 source regions,
generated a 376-byte schema-2 document with inclusive `[-6,6]²` bounds and SHA-256
`b6c33bbd...b6fa1`, reached a live product process, and stopped with empty target/broker PID sets.
Terrain/object sizes and hashes remained 1,835,008 / 18,092,032 bytes and
`b93391dd...0b97` / `ca1f2163...5186`.

The final diff leaves engine-runtime, renderer/GPU, source formats, assets, manifests, and Wulin
ownership unchanged. The merge-checkpoint guard passed with zero Flavor denies and five
pre-existing warnings.

## Conclusion

Accepted. Strict bootstrap schema 2 now owns one explicit playable global-region rectangle, and
the prototype enforces it conservatively per axis before its existing runtime transaction. A real
activated held-W process remains live beyond the demonstrated finite-source failure while strict
runtime source/query semantics remain unchanged.

## Promotion

ADR 0086 accepts the schema/application ownership boundary. The product bootstrap writer and
existing canonical-prototype workflow become its maintained operator and regression owners; the
dependency-free investigation script remains task evidence rather than a new live wrapper.
