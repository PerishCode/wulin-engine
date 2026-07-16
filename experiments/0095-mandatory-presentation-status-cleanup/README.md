# Experiment 0095: Mandatory Presentation Status Cleanup

Status: Accepted

## Hypothesis

The standalone `canonical.time.status` inspect chain can be deleted because the canonical aggregate
already publishes the exact same `presentationClock` value. Maintained temporal and actor gates can
consume that sole aggregate authority without weakening mutation, rollback, lifecycle, or GPU
evidence and without retaining a compatibility endpoint.

## Scope

- Delete `Runtime::presentation_time_status`, `ControlKind::CanonicalTimeStatus`, the parser arm,
  and the workbench dispatch arm.
- Retain `canonical.status.presentationClock` and the existing pause/resume/set/step controls.
  Mutation controls return the private timeline status directly after their existing operation.
- Replace all eleven maintained standalone reads in temporal presentation, actor lifecycle,
  simulation, and render-admission support with the existing aggregate field.
- Keep one real-process rejection for the newly retired verb and add a static guard covering the
  method, variant, route, and maintained support consumers.
- Add no alias, redirect, replacement verb, response field, cache, or product telemetry.

Presentation arithmetic, automatic/manual advancement, frame commit, actor-local animation epoch,
simulation scheduling, renderer resources, sources, formats, assets, product behavior, and Wulin
content are out of scope.

## Workload

1. Inventory the duplicate Runtime/protocol/dispatch chain, every maintained consumer, the canonical
   aggregate authority, and all product/manual operator consumers.
2. Delete the full standalone chain and migrate every assertion to
   `canonical.status.presentationClock`.
3. Preserve pause/resume/set/step response values while removing the public read-only forwarder.
4. Require the retired verb to fail through the generic `unknown_event` contract in the full
   temporal workload.
5. Add a stable removal guard and deliberately restore the forbidden method name once to prove the
   guard fails before compilation; then remove the mutant and require a clean pass.
6. Run focused Rust/Deno checks, `runseal :canonical-actor`, `runseal :canonical-runtime`, and the
   repository guard.

## Controlled Variables

- `PresentationTimeline`, its JSON encoding, counters, exact 31,002,560-frame period, validation,
  and successful-frame advancement remain unchanged.
- `Runtime::composition_status` remains the sole aggregate timeline read; canonical frame/probe
  semantics remain unchanged.
- Pause/resume/set/step validation and returned JSON shape remain unchanged.
- Actor lifecycle, simulation transaction, presentation epoch, render admission/backpressure, GPU
  candidate, and process restart behavior remain unchanged.
- Unsupported inspect events continue through the existing generic rejection path; no retired
  parser or special compatibility branch remains.

## Metrics

- Removed Runtime methods, protocol variants/arms, dispatch branches, and standalone support reads.
- Exact clock equality through invalid set/step rollback, manual wrap, automatic advancement,
  held publication, actor lifecycle/restart, simulation partitioning, and render admission.
- Retired-event rejection, deliberate guard failure, Rust/Deno checks, focused/full workflow time,
  traversal, resource, lifecycle, and Flavor outcomes.

## Acceptance Criteria

- No live standalone presentation-status method, enum variant, parser, dispatch, wrapper command,
  alias, redirect, decoder, or maintained read remains outside the explicit guard definition and
  settled history.
- `canonical.time.status` returns generic `unknown_event` in a current real workbench process.
- Pause/resume/set/step keep their exact status responses and all temporal rollback/wrap/automatic/
  held-publication assertions pass through the aggregate authority.
- Actor lifecycle, restart, fractional/coarse/nominal simulation, pending render block, GPU
  admission, and animation epoch remain exact.
- A deliberate forbidden-symbol reintroduction fails the guard before expensive checks; the
  restored repository passes with zero Flavor denies.
- Focused actor and optimized full-runtime workflows pass, including bounded traversal, resource,
  and lifecycle checkpoints.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, strict inspect
protocol, and maintained actor/full-runtime acceptance workflows.

## Evidence

The duplicate chain is gone: one public Runtime method, one protocol variant, one parser arm, and
one workbench dispatch arm. Eleven maintained reads now use the existing aggregate. The four
presentation mutations return the unchanged private timeline JSON directly. Repository search finds
the retired symbols only inside the guard definition and settled history.

Focused checks pass 90 engine-runtime tests, workbench compilation, Rust/Deno formatting, and Deno
type checking. Temporarily restoring the forbidden method name made the new guard reject the mutant
in under one second; removing it restored a clean guard pass.

`canonical-actor-v8` passed in 80,179.580 ms. Lifecycle/restart replay retained SHA-256
`6e5e8e3550b0eaf7d3f8cb740d3e1b99e69b5bbb7dd41277771eb40ec995ceb1`; fractional work,
coarse/nominal 60-step partitions, validation/query/arithmetic rollback, pending render admission,
GPU candidate, presentation epoch, and process cleanup all remained exact through the aggregate
clock.

`canonical-runtime-v4` passed in 268.804 seconds. The report contains exactly one rejected
`canonical.time.status` event with
`unknown_event: unsupported event "canonical.time.status"`; 326 `canonical.status` reads continued
to cover temporal, simulation, publication, and checkpoint state. Manual tick/wrap, invalid set and
running-step rollback, automatic advancement, and held old-publication time all passed.

Both 32-sample traversal sweeps passed. The eight-publication resource checkpoint held 503 handles
at baseline/peak/final and finished 950,272 private bytes above baseline, within the 16 MiB bound.
Both lifecycle cycles cleaned up. The optimized full report retained 113 probes, 32 observations,
six captures, 24 files, and 25,346,228 bytes. Final repository guard passes with zero Flavor denies.

## Conclusion

Accepted. Presentation state now has one read authority in the canonical aggregate. The four
mutation controls retain their exact responses, while the duplicate compatibility route is absent
and statically prevented from returning.

## Promotion

Promoted no runtime capability. Removed the duplicate Runtime/workbench status chain, migrated
maintained evidence to the current aggregate, added one current rejection witness, and promoted a
stable removal guard. No compatibility alias, product behavior, renderer/GPU/resource ownership,
source/format/asset, networking, or Wulin surface was added.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p workbench --release
runseal :canonical-actor
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-actor/` and
`out/captures/canonical-runtime/`.
