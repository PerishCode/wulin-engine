# Experiment 0040: Runtime Frame Transaction

Status: Accepted

## Hypothesis

The engine runtime can own one deterministic frame transaction and the canonical presentation
timeline while the renderer consumes an immutable presentation tick, preserving every accepted
frame, control, failure, resource, and lifecycle outcome without introducing elapsed wall time, a
simulation clock, a fixed-step policy, or a second render path.

## Scope

This experiment moves the accepted presentation timeline state and controls from the skeletal
renderer into the `Runtime` owner accepted by Experiment 0039. A runtime frame samples the current
tick, renders and collects any capture/probe evidence at that tick, and commits one automatic
advance only after a successful submitted canonical frame. The renderer receives the sampled tick
as frame input and cannot decide whether time advances.

The existing integer source-duration phase formula, 4,800 units per nominal second, 80 units per
frame, common period, pause/resume/set/step inspect vocabulary, and post-submit advance behavior
remain unchanged.

Elapsed wall time, display pacing, interpolation, fixed or variable simulation steps, gameplay or
network time, input, source bootstrap, player/actor state, root motion, new visuals, portability,
and Wulin content are out of scope.

## Workload

1. Record the current workbench/runtime/renderer call order, pause behavior, presentation state,
   and renderer time dependencies.
2. Add one runtime-owned timeline with the accepted tick/running/counter/wrap behavior and focused
   state-transition tests.
3. Pass the sampled tick through renderer, skeletal recording, capture/probe, and CPU/GPU oracle
   inputs without retaining a mutable clock in rendering ownership.
4. Preserve the existing workbench controls and JSON responses while making `Runtime::frame` the
   sole begin/render/commit coordinator.
5. Add a stable repository gate for runtime timeline ownership, run focused tests and the complete
   direct canonical GPU workflow, then revalidate all failure, traversal, resource, and lifecycle
   gates.

## Controlled Variables

- Signed sources, exact identity, residency, composition, traversal, terrain, grounding, LOD,
  animation selection, source-duration phase mapping, shadows, shaders, attachments, and all
  accepted hashes remain unchanged.
- A successful canonical frame still renders and probes the pre-advance tick, then advances by one
  frame. Idle-shell frames do not advance. Pause, set, step, invalid-request rollback, source
  changes, failure holds, traversal, and rollover retain their current semantics.
- The workbench frame loop, inspect vocabulary, capture persistence, and host pause behavior remain
  unchanged.
- No wall clock or host frame duration becomes an engine input in this experiment.

## Metrics

- Timeline owner count, renderer time-state/control symbol count, sampled/committed frame order,
  automatic/manual/wrap counters, and focused transition-test results.
- Controlled color, PNG, object-ID, diagnostic, light-matrix, and shadow-depth SHA-256 values.
- Source-duration tick/phase/palette evidence, held-pair and invalid-request rollback, visible and
  submitted work, CPU/GPU mismatches, and content transaction counters.
- Traversal continuity, device state, 64-publication handle/private-byte/thread plateau, and 16
  complete lifecycle cycles.

## Acceptance Criteria

- One runtime-owned timeline contains the active tick, running state, automatic advance count,
  manual step count, and wrap count. Rendering modules contain no competing mutable timeline or
  pause/resume/set/step authority.
- `Runtime::frame` samples one immutable tick, passes it through every presentation and oracle
  consumer, and commits an automatic advance only after the renderer returns a successful
  canonical frame. Evidence observes the sampled pre-commit tick; idle-shell and failed frames do
  not commit.
- `canonical.time.status`, `pause`, `resume`, `set`, and `step` preserve their revision, fields,
  bounds, paused-only mutation rules, counters, wrap behavior, and invalid-request rollback.
- Controlled source-duration frames and the six Experiment 0039 attachment/shadow hashes remain
  exact. There is no new GPU resource, descriptor, root constant, content copy, submission, or
  renderer path.
- All focused timeline/runtime tests, the repository guard, source/failure/hold/rollover gates,
  32 reactive and 32 prepared crossings, the 64-publication plateau, and all 16 lifecycle cycles
  pass without device removal or validation errors.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0040-runtime-frame-transaction/`.

## Results

The 598.8-second direct GPU workflow passed. The runtime sampled one immutable tick for each frame
and committed automatic advancement only after a successful canonical render. Tick 0/42/43/85
still selected imported Walk phases 0/63/0/0; frames 0, 43, and 85 reproduced the same color hash,
and the maximum CPU/GPU palette delta remained `2.3283064e-10`.

The controlled color, PNG, object-ID, diagnostic, light-matrix, and shadow-depth hashes exactly
matched Experiment 0039. The frame retained 10,538 shadow casters, one indirect shadow dispatch,
60 shadow root-constant DWORDs, 98 shadow descriptors, four fixed terrain submissions, six fixed
skeletal submissions, and zero grounding, palette, surface, or shadow sample mismatch.

Three focused timeline tests prove pre-commit sampling, paused automatic holds, manual common-period
wrap, and invalid-request rollback. The complete engine-runtime suite now has 21 tests. A first
guard run rejected an eight-argument renderer call; the accepted implementation instead names the
immutable renderer input `RenderFrame`. Flavor then required private timeline tests to live under
`tests/private`; the final source contains only the established test-only module hook and one
scoped private-owner rule rather than a public testing API.

All source, reorder, revisit, alias, held-pair, failure, rollback, and rollover gates passed. The
automatic clock reached tick 12 after 12 committed frames, while a held incomplete pair continued
to the expected tick 11 without changing content authority. All 32 reactive and 32 prepared
crossings and 16 lifecycle cycles passed.

The resource workload settled at 492 handles, 421,593,088 private bytes, and 18 threads. The
post-settle publication samples peaked at 490 handles with zero transient growth; publication 64
ended at 490 handles, 424,812,544 private bytes, and 17 threads.

## Conclusion

Accepted. The engine runtime now owns the only mutable presentation timeline and the successful
frame commit decision. Rendering receives a frame-scoped tick and status snapshot, uses them for
GPU constants and exact probe/oracle evidence, and cannot pause, set, step, or advance time. The
change preserves all externally accepted behavior while establishing the temporal ownership and
render/commit order needed before any simulation-step policy is considered.

## Promotion

Promoted `PresentationTimeline` and frame transaction coordination into
`crates/engine-runtime`. Added a stable repository gate that rejects presentation timeline control
or advancement under rendering ownership. Elapsed time, display pacing, interpolation, simulation
time, fixed-step scheduling, input sampling, and gameplay state remain unproven.
