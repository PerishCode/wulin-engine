# Experiment 0073: Typed Actor Render Backpressure

Status: Accepted

## Hypothesis

The engine can distinguish a candidate that passes published admission but misses an authoritative
non-prefetch pending window from genuine simulation/projection failure before mutation, return that
case as one typed nonfatal actor-simulation outcome, and let the prototype continue anchoring and
rendering the retained actor. This can be done without matching error strings, retrying elapsed
time, weakening frame preflight, or attaching horizontal input and traversal policy.

## Scope

Refine the renderer-private actor preflight into admitted/active-blocked/pending-blocked results.
Frame callers convert both blocks back into their existing exact failures.
`Runtime::advance_simulation_actor` keeps a published active block fatal and maps only a pending
block to a public typed outcome after copied schedule/motion preparation and before actor/schedule
commit.

Workbench publishes an explicit schema-2 advanced/render-blocked response. Prototype consumes a
blocked Ready sample as no simulation commit, increments a checked evidence counter, performs no
retry or accumulation, and continues its existing camera/frame order with the retained actor.

Horizontal input mapping, traversal/prefetch enablement, command rewriting, retry queues, elapsed
accumulation, camera smoothing, multi-actor storage, Wulin content, frame algorithms, shaders, GPU
resources, and synchronization are out of scope.

## Workload

1. Prove private projection classification distinguishes a valid actor outside a window from
   config/projection failure and required admission preserves exact active/pending error identity.
2. Preserve advanced schema evidence for fractional, partitioned, terrain-failure, arithmetic, and
   stale-identity simulation workloads.
3. Hold the existing diagonal non-prefetch pending pair, prepare one candidate outside it but inside
   the published pair, and require a successful typed render-blocked response with one prepared
   step/query and zero actor/schedule commits.
4. Require byte-identical actor, complete schedule, pending token/stages, and a successful retained
   frame after the typed block; then release/publicize the pending pair and retain exact actor GPU
   evidence.
5. Unit-test prototype block consumption and run the direct prototype restart/failure/lifecycle
   workload with schema-2 driver evidence and zero normal-path block count.

## Controlled Variables

- The 60 Hz schedule, copied preparation, motion/contact order, candidate identity/presentation, and
  pre-commit location are unchanged.
- Published and non-prefetch pending windows remain authoritative. Speculative prefetch remains
  excluded exactly as in frame preflight.
- Only pending-window absence after published admission is nonfatal. Published-window, identity,
  elapsed, terrain, arithmetic, configuration, projection, renderer, and frame failures remain
  terminal errors.
- A blocked prototype sample is consumed once by HostClock, not retried or accumulated; runtime
  schedule and actor state do not advance.
- Camera/frame order, shaders, passes, resources, uploads, barriers, fences, waits, and source
  formats are unchanged.

## Metrics

- Advanced versus render-blocked outcome count and schema identity.
- Prepared step/query count and actor/schedule commit count for a block.
- Fatal-error identity, actor/schedule/pending mutation count, and retained-frame success.
- Prototype block-count accounting, readiness/frame ordering, restart equality, and final process
  cleanup.
- Engine API, projection route, pass, resource, copy, synchronization, and allocation deltas.

## Acceptance Criteria

- Pending-window absence after successful published admission produces exactly one typed
  render-blocked outcome; published-window absence and every other error category remain errors.
- A blocked response reports one prepared step/query, zero schedule/actor commits, unchanged stored
  state and pending transaction, and the retained actor still renders.
- Frame preflight continues to reject outside active/pending actors with its established errors.
- Prototype consumes the typed block without retry/backlog or process termination and continues the
  existing camera/frame order; normal direct runs report zero blocks.
- Focused tests, `runseal :canonical-actor`, `runseal :canonical-prototype`, and `runseal :guard`
  pass. No input/traversal, fallback command, GPU resource, synchronization, or compatibility route
  is added.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, reference Windows workbench and
prototype, fresh signed base/diagonal sources, the capacity-one runtime actor, and the sole canonical
renderer.

## Evidence

- Engine/runtime tests passed all 74 unit tests plus semantic actor identity. Prototype passed four
  actor/camera tests and two time-policy tests; the new policy proves one render block yields no
  advance/backlog, increments its checked counter once, and preserves the counter on overflow.
- Renderer tests prove window absence remains distinguishable from config divergence and required
  admission preserves the exact active/pending error strings. Runtime maps only pending blocks; no
  error text is used to classify backpressure.
- `runseal :canonical-actor` emitted schema `canonical-actor-v2` and passed in `36.740` seconds. The
  held diagonal candidate returned `runtime-actor-simulation-v2 / render-blocked` with prepared
  step/query `1/1`, schedule/actor commit `0/0`, and zero frame/renderer/GPU/synchronization work.
- The blocked actor remained generation `2`; schedule stayed at tick `1`, remainder `20`, and two
  successful advances. Pending token/config/stages remained identical, the retained probe frame
  succeeded, the pending pair completed after release, and the existing actor GPU record/capture
  hashes passed unchanged.
- Fresh actor-gate source hashes remained terrain
  `d477e79c49631565be3d43131a6a2c4f39a8efac0b9cb33143519c306290da28` and objects
  `e8d5565dab74f334c37096e3f8874c6fec95325fbe9a7a3840de1fb5c28119fb`.
- `runseal :canonical-prototype` emitted schema `canonical-prototype-v2` and passed in `33.457`
  seconds with 74 runtime tests, six prototype tests, and 21 reference-host tests. Direct processes
  `22040` and `20492` both reported `live-prototype-gravity-driver-v2`, zero render blocks, exact
  grounded actor stability, and camera anchor/frame counts `3/3`.
- Invalid document, missing source, and corrupt payload still emitted no readiness. Direct restart,
  stop, and complete Sidecar cleanup passed. No input, traversal, frame algorithm, GPU resource,
  copy, synchronization, allocation, public projection, or compatibility route changed.
- `runseal :init` and `runseal :guard` passed. Flavor reported zero denies and the same five
  pre-existing warnings.

## Conclusion

Accepted. Pending-window pressure after published admission is now one typed nonfatal simulation
outcome while published-window and every genuine failure retain the error channel. Prototype has an
explicit no-retry/no-backlog consumption policy and can continue rendering retained state before
horizontal input or traversal is introduced.
