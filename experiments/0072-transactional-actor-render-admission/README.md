# Experiment 0072: Transactional Actor Render Admission

Status: Accepted

## Hypothesis

When canonical composition is enabled, the runtime can validate a fully prepared simulation-actor
candidate through the sole existing frame preflight before committing either actor motion or the
simulation schedule. Every successful advance will then be render-admissible against both the
published pair and any non-prefetch pending pair, without changing prepublication simulation,
adding another projection route, or introducing input/backpressure policy.

## Scope

Insert one candidate admission point into the existing `Runtime::advance_simulation_actor`
transaction after copied schedule/motion preparation and before its actor/schedule dual commit.
The candidate preserves the exact handle and presentation and replaces only motion. Admission uses
`Renderer::preflight_actor`, the same private authority called by the frame.

Canonical-disabled advances remain unchanged because no actor projection/rendering exists in that
state. A rejected canonical candidate remains an explicit error and exposes no actor or schedule
mutation. Turning expected admission rejection into a typed, nonfatal application outcome is a
later decision.

Horizontal command mapping, traversal/prefetch enablement in the prototype, retry behavior,
elapsed-time handling during a block, command substitution, camera smoothing, multi-actor storage,
Wulin content, shaders, GPU resources, and synchronization are out of scope.

## Workload

1. Preserve an exact fractional prepublication simulation/actor commit with no canonical
   composition and no terrain query.
2. Publish the base pair, hold one non-prefetch pending pair centered at `base + (2,2)`, and spawn a
   grounded actor at the shared corner of both windows.
3. Commit one due step whose candidate remains in both windows and require the existing dual-commit
   response, schedule tick, actor generation/presentation, and terrain query count.
4. Prepare another due step whose destination remains in published terrain but lies outside the
   held pending window. Require exact rejection with byte-identical stored actor and simulation
   status plus unchanged pending token/stages.
5. Render the retained actor, release and publish the pending pair, stop that process, then start a
   fresh base publication for the existing canonical actor GPU admission/capture/rollback workload.

## Controlled Variables

- Simulation remains the existing rational 60 Hz schedule and fixed terrain-body batch.
- Terrain queries, contact ordering, actor capacity/generation/presentation, and commit order are
  unchanged.
- Published and non-prefetch pending projection use the existing private frame preflight.
  Speculative prefetch remains excluded until promotion, matching frame behavior.
- The rejected command is not retried, rewritten, clamped, or converted to zero motion.
- Camera, frame algorithm, shaders, resources, uploads, barriers, fences, waits, and source formats
  are unchanged.

## Metrics

- Successful candidate actor/schedule commit count and exact output.
- Rejected candidate terrain-query work, error identity, actor/schedule mutation count, and pending
  transaction stability.
- Prepublication behavior and existing actor GPU record/capture hashes.
- Engine API, projection route, pass, resource, copy, synchronization, and allocation deltas.

## Acceptance Criteria

- With canonical composition enabled, no prepared actor candidate commits before exact published
  and non-prefetch pending preflight succeeds.
- A pending-window rejection preserves the stored actor and complete simulation schedule, changes
  no pending token/stage, and the retained actor still renders successfully.
- An admissible candidate under the same pending pair commits exactly once with unchanged handle,
  presentation, schedule arithmetic, and query accounting.
- Canonical-disabled fractional advance retains its accepted behavior. Speculative prefetch is not
  promoted into an admission dependency before the frame does so.
- `runseal :canonical-actor`, focused checks, and `runseal :guard` pass. No public projection read,
  typed application backpressure, fallback command, frame/GPU resource, synchronization, or
  compatibility path is added.

## Reference Environment

The experiment uses the repository-pinned Rust/Deno toolchains, reference Windows workbench, fresh
signed terrain/object sources for the base and diagonal centers, the capacity-one runtime actor,
and the sole canonical renderer.

## Evidence

- `cargo test --locked -p engine-runtime` passed all 74 unit tests plus the semantic actor test.
- A prepublication one-nanosecond advance retained the accepted zero-step behavior, committed actor
  and schedule once, performed no terrain query, and preserved remainder numerator `60`.
- With the diagonal non-prefetch pair held at terrain `staged` and objects `in-flight`, the shared
  candidate committed tick `1`, remainder `20`, one terrain query, and local X `1` while preserving
  generation `2`, handle, and presentation.
- The next candidate was valid in the published base window but outside the pending diagonal window.
  It returned exactly `actor_simulation_advance_failed: runtime actor is outside the pending render
  window`; actor, full schedule, pending token `2`, config, stages, camera-driven flag, and prefetch
  flag remained identical. The retained actor then completed `canonical.probe` successfully.
- The optimized two-process `runseal :canonical-actor` gate passed in `31.577` seconds. Its initial
  three-process form also passed in `46.081` seconds; combining the independent prepublication and
  held-pending checks removed an unnecessary cold start without reducing assertions.
- Fresh source hashes were terrain
  `d477e79c49631565be3d43131a6a2c4f39a8efac0b9cb33143519c306290da28` and objects
  `e8d5565dab74f334c37096e3f8874c6fec95325fbe9a7a3840de1fb5c28119fb`. Existing generation-one
  and generation-two actor record hashes remained
  `2f14c5bbe4268821f0cdd3cbc7a8fbce6d92c08c944476a91f82f353f505ac3a` and
  `ab33d261aebcc263cf32a7849fb3637a2a82e37c130a021a1962327a8198dc78`.
- `runseal :init` and `runseal :guard` passed with zero Flavor denies. No frame algorithm, GPU
  resource, copy, synchronization, allocation, public projection, or compatibility surface changed.

## Conclusion

Accepted. Canonical simulation candidates now share the frame's sole render-admission authority
before the existing actor/schedule dual commit. The narrow preflight closes the state divergence
without changing prepublication simulation or promoting application backpressure policy.
