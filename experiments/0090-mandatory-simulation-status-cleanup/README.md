# Experiment 0090: Mandatory Simulation Status Cleanup

Status: Accepted

## Hypothesis

The standalone `simulation.status` inspect chain can be deleted now that the canonical aggregate
already publishes the exact same `simulationSchedule` value and every successful actor transaction
publishes its own exact schedule transition. Maintained actor gates can consume those current
authorities without weakening rollback evidence, retaining a compatibility verb, or adding a
replacement status route.

## Scope

Delete `Runtime::simulation_status`, `ControlKind::SimulationStatus`, the `simulation.status` parser
arm, and its workbench dispatch. Retain private `SimulationSchedule::status_json`,
`canonical.status.simulationSchedule`, canonical frame/probe schedule evidence, and the serialized
`SimulationAdvance` carried by actor outcomes.

Move the 16 maintained actor-support reads in lifecycle, simulation, and render-admission gates to
the existing canonical aggregate field. Do not add a new helper API, verb, response field, cache,
or product telemetry.

Delete the recurring eight-request `retiredControlGate` accumulated from prior cleanup stages and
its `retiredControls` report field. Their implementation absence remains owned by the stable
simulation-control guard and settled experiment history. Keep one direct prepublication rejection
for the newly retired `simulation.status` verb, then extend the stable guard to reject its method,
variant, parser, dispatch, and any reintroduced standalone support dependency.

## Workload

1. Inventory the standalone chain, all consumers, the canonical aggregate authority, actor
   transition evidence, internal probe uses, and accumulated retired-control requests.
2. Delete the public Runtime method and complete workbench protocol/dispatch route.
3. Replace every maintained actor-support read with the existing
   `canonical.status.simulationSchedule` value while preserving exact schedule, actor, presentation,
   render-block, process, and restart assertions.
4. Reduce recurring historical rejection work from eight old requests to one newly retired status
   request. Require `unknown_event` without an alias, redirect, or empty compatibility response.
5. Extend the stable removal guard and deliberately reintroduce one forbidden symbol to prove it
   fails before build/test work; restore the clean tree and require it to pass.
6. Run focused Rust/Deno checks, `runseal :canonical-actor`, `runseal :init`, and `runseal :guard`.

## Controlled Variables

- Simulation schedule arithmetic, bounds, counters, internal status encoding, prepare/commit order,
  and focused rollback tests remain unchanged.
- `Runtime::composition_status`, canonical frame/probe schedule evidence, and actor
  `SimulationAdvance` serialization remain unchanged.
- Actor lifecycle, command schema, initial velocity delta, grounded witness, presentation epoch,
  render admission/backpressure, and GPU instance path remain unchanged.
- Prototype time/action policy, input, camera, traversal, renderer resources, sources, formats,
  assets, and Wulin behavior remain unchanged.
- Unsupported inspect verbs continue through the existing generic `unknown_event` contract; no
  deprecated parser or special compatibility branch is retained.

## Metrics

- Removed Runtime methods, protocol variants/arms, dispatch branches, standalone support reads,
  recurring retired requests/report fields, and net live lines.
- Exact status equality through the canonical aggregate before/after fractional, partitioned,
  failed, blocked, lifecycle, and restart workloads.
- Actor transition ticks/remainders, actor state, grounded witness, presentation, retained frame,
  GPU admission, process count, and resource/work counters.
- Retired status rejection, deliberate guard failure, focused workflow duration, and Flavor result.

## Acceptance Criteria

- No live standalone `simulation.status` method, variant, parser, dispatch, wrapper command, alias,
  redirect, decoder, response revision, or support read remains outside the explicit rejection and
  stable guard.
- The old eight-request recurring retired-control list and `retiredControls` report field are gone;
  stable guards and historical experiment/ADR records remain the only authority for those paths.
- `simulation.status` returns generic `unknown_event` in the current actor prepublication process.
- Lifecycle/restart independence, validation/query/arithmetic rollback, fractional/coarse/nominal
  schedule results, pending-window rollback, retained-frame state, actor/GPU behavior, and hashes
  remain exact through the canonical aggregate and actor transitions.
- A deliberate standalone-symbol reintroduction fails the guard before expensive checks, and the
  restored repository passes with zero Flavor denies.
- Focused checks, canonical-actor, init, and guard pass. Prototype and the long canonical runtime
  workflows are not required because no product, renderer, GPU resource, synchronization, source,
  format, asset, or lifecycle implementation changes.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference workbench, maintained actor
workflow, strict inspect protocol, and existing canonical status/probe owners.

## Evidence

The complete duplicate chain is gone: one public Runtime method, one protocol variant, one parser
arm, and one workbench dispatch arm. Sixteen maintained acceptance reads now use the existing
`canonical.status.simulationSchedule` object. Private schedule encoding, canonical aggregate,
frame/probe evidence, and actor `SimulationAdvance` are unchanged.

The recurring `retiredControlGate` deleted eight old process requests and the `retiredControls`
report field. The current prepublication actor-admission process retains only one rejection for
`simulation.status`, which returns exactly
`unknown_event: unsupported event "simulation.status"`. The stable guard now forbids the method,
variant, verb, recurring gate/report name, and all older retired request names in live actor
support. Temporarily restoring `Runtime::simulation_status` made guard fail in 1.5 seconds at the
removal scan before compilation; removing it restored the clean gate.

Focused checks pass 83 engine-runtime tests plus workbench compilation, Rust formatting, Deno
formatting, and type checks. Final `canonical-actor-v7` passed in 79,793.280 ms after promoting the
existing lifecycle and simulation-actor gates into that focused workflow. Lifecycle restart replay
matched exact SHA-256
`6e5e8e3550b0eaf7d3f8cb740d3e1b99e69b5bbb7dd41277771eb40ec995ceb1`.
Fractional work ended at tick/remainder `0/60`; coarse eight-batch and nominal sixty-partition runs
both ended at `60/0` with 60 terrain queries and exact actor equality. Query failure at batch step
3 and arithmetic failure at step 1 preserved canonical aggregate schedule and actor state. The
combined simulation result SHA-256 is
`27f414bc5909bc21a6ee3c5dcce0e332fe8000a5b730fa9c6ed5d85df1bf8825`.

Render admission retained exact typed behavior: the admitted transaction left schedule
tick/remainder `1/20`; the pending-window block reported one prepared step/query and zero commits,
and the retained canonical frame still reported `1/20`. Actor identity, initial vertical delta,
grounded witness, presentation/epoch, GPU candidate, semantic capture, despawn/respawn, and process
cleanup remained exact. `runseal :init` and `runseal :guard` passed with zero Flavor denies and five
pre-existing warnings. Prototype and the long canonical runtime workflow were not run because no
product, renderer/GPU/resource/synchronization/source/format/asset/lifecycle implementation
changed; all modified actor support now runs in the focused workflow.

## Conclusion

Accepted. Schedule state now has one inspect authority in the canonical aggregate, while actor
transactions retain their exact local transition evidence. The standalone compatibility route and
recurring settled-history requests are absent without weakening maintained rollback or lifecycle
proof.

## Promotion

Promoted no runtime capability. Removed the duplicate Runtime/workbench status chain, reduced
recurring rejection evidence to the current retired verb, extended the stable guard, and expanded
the existing canonical-actor workflow to own all maintained actor lifecycle/simulation support.
No alias, replacement verb/field, product telemetry, renderer/GPU change, or Wulin surface was
added.
