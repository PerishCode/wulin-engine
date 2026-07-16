# Experiment 0097: Committed Prototype Object Observation

Status: Accepted

## Hypothesis

The prototype can consume the accepted bounded nearest-object query as one capacity-one read-only
intent after a successful actor commit, without turning the linear scan into recurring frame work or
introducing retained selection, interaction, or another object authority.

## Scope

- Add one prototype-owned F observation intent with a fixed inclusive 512-Q9 planar radius.
- Coalesce duplicate presses into one pending bit.
- Observe host time discontinuity before current-batch admission: Reset and Suspended cancel the
  intent, while Ready and Stalled preserve it.
- Preserve the intent across zero-step samples and typed render backpressure.
- After the next successful nonzero actor advance, query from that exact committed output position.
- Clear the intent only after the existing runtime query succeeds. Retain no object or selection
  beyond same-completion readiness evidence.
- Prove the product join with a visible-window native F+W process and an independent schema-3 pack
  oracle.

Automatic per-frame proximity, enumeration, a spatial index, retained selection, disappearance or
source-replacement policy, highlight, interaction eligibility/effects, facing or line of sight, 3D
distance, navigation/collision, persistent identity, networking, multiple actors, and Wulin
semantics are out of scope.

## Workload

1. Prove duplicate presses coalesce, zero steps retain, and a nonzero committed advance completes
   exactly once.
2. Prove Ready and Stalled retain pending intent while Reset and Suspended cancel it.
3. Preserve the ordinary and restarted prototype with no pending or completed observation.
4. Launch one visible native F+W process and verify normalized input, ordinary forward simulation,
   and exact post-commit query ordering.
5. Decode the cooked `.wlr` independently and compare all 25,600 candidates, the nearest identity,
   normalized position, deltas, and squared distance at the emitted origin.
6. Preserve focused startup failure, finite boundary, locomotion, presentation, camera, jump,
   traversal, Escape, restart, and process cleanup gates.

## Controlled Variables

- The runtime nearest query, committed snapshot, schema-3 source, cache, renderer, GPU resources,
  traversal, bootstrap, and actor transaction are unchanged.
- The policy owns only one boolean; it stores neither a result nor a completion count.
- Query origin is copied from `advance.actor.output`, never the pre-transaction actor or a prepared
  render-blocked candidate.
- A successful no-candidate query still completes the intent. Query failure is fatal and cannot
  consume state before success.
- Observation is event-driven and performs no scan without an admitted F press and nonzero commit.

## Metrics

- Pending state before and after zero/nonzero steps and every host elapsed outcome.
- Native input key identities, committed actor input/output, observation origin, candidate count,
  selected object, deltas, squared distance, and source-oracle equality.
- Focused test counts, workflow duration, startup/restart/boundary/exit outcomes, and process cleanup.

## Acceptance Criteria

- The policy retains at most one intent and consumes it only after a successful nonzero actor
  transaction and successful nearest query.
- Reset/Suspended cancel stale intent; Ready/Stalled, fractional work, and render backpressure do
  not consume it.
- Native F+W observation origin equals the committed forward actor output and its result equals the
  independent source-byte oracle within the exact inclusive 512-Q9 radius.
- Ordinary and restarted processes expose no observation and reproduce identical empty policy
  state.
- No recurring scan, retained target, interaction action, engine/host input policy, new resource,
  compatibility route, asset/format change, networking, or Wulin behavior is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, the maintained
signed canonical prototype source centered at `(2^40, -2^40)`, and the focused prototype workflow.

## Evidence

The focused engine-runtime, prototype, and reference-host test selection passes. Two policy tests
cover capacity-one admission, zero-step retention, nonzero completion, stall/Ready preservation,
and Reset/Suspended cancellation. Clippy and all selected Deno checks pass.

The final `canonical-prototype-v17` pass completed in 71.900 seconds. The visible process
received native F/VK 70 and W/VK 87 together. Its one-step actor transition committed from local Z
zero to `-32` Q9; the observation used that exact output as its origin, scanned 25,600 committed
candidates, and matched the independent source oracle at every returned field. The oracle selected
authored local ID 496 at delta `(160,0)` and squared Q18 distance 25,600, inside the 512-Q9 radius.
Ordinary and restarted processes retained no intent or result, and all existing focused product,
failure, boundary, exit, and cleanup gates passed with no residual process.

Full runtime and deep resource workflows are not selected: this experiment changes no runtime
query behavior, GPU/source resource, publication lifetime, or recovery owner. Repository guard is
the final cross-repository gate and passes with zero Flavor deny issues; the five retained warnings
are unchanged ownership pressure outside this experiment.

## Conclusion

Accepted. The prototype can request one bounded read-only nearby-object observation whose spatial
origin is the next committed actor output. This is not retained selection or interaction authority.

## Promotion

Promoted the prototype-owned capacity-one policy, fixed 512-Q9 radius, post-commit ordering, native
F+W focused gate, and independent source oracle reuse. Promoted no recurring scan, retained result,
selection/action state, engine input policy, new resource, compatibility surface, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype -p reference-host
runseal :canonical-prototype
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-prototype/`.
