# Experiment 0066: Bounded Actor Render Projection

Status: Accepted

## Hypothesis

The exact live runtime actor can be projected against the sole published canonical composition
into one bounded, window-relative, integer render record that preserves handle, motion height, and
presentation exactly across signed global coordinates, region seams, and origin rollover, without
adding a GPU buffer, float world position, second scene, actor collection, camera policy, or frame
mutation.

## Scope

Read one actor by its exact generation, then read the currently enabled published composition as
the only spatial projection authority. Map the actor's signed global region to one active-window
ordinal, reuse the canonical terrain projection for its semantic region and exact Q9 X/Z window
position, and retain the actor's Q16 center/half-height plus complete schema-3 presentation.

The result is a copied immutable projection transaction. It does not upload, cull, draw, ground,
animate, identify, or otherwise mutate the actor. Dynamic GPU storage, shader descriptors, visible
candidate injection, surface/shadow/occlusion participation, semantic actor IDs, camera following,
input mapping, locomotion, gravity, multi-actor capacity, and Wulin content remain out of scope.

## Workload

1. Add one allocation-free active-window ordinal lookup to signed global addressing and one exact
   Q9 position operation to the existing terrain projection owner.
2. Add one renderer-owned projection transaction over the exact published pair and one actor
   copied from the runtime slot. Require an enabled canonical composition and reject actors outside
   its active window.
3. Expose the transaction through `Runtime` and one strict `actor.project` inspect verb. Preserve
   the exact actor, published global config, active ordinal, semantic region, Q9 window position,
   Q16 center/half-height, and denominators in its response.
4. Prove center, all four window corners, signed seams, `2^40` coordinates, origin alias changes,
   rollover invariance, outside-window rejection, empty/stale handle ordering, and byte-identical
   repeated projection.
5. Run focused Rust/protocol checks, one short real-process gate over a freshly published signed
   pair, and `runseal :guard`. Do not run the long canonical workflow because no frame, GPU
   resource, descriptor, shader, synchronization, or lifecycle implementation changes.

## Controlled Variables

- Actor position remains the accepted signed global region plus half-open local Q9 coordinates;
  no global coordinate is converted to float.
- Window X/Z are exact signed Q9 values: canonical active-region offset times 8,192 plus actor local
  Q9. Q16 center and half-height are copied without conversion.
- Active ordinal and semantic region reuse the same row-major active window and centered semantic
  projection used by canonical terrain/object rendering.
- Actor generation is validated before composition lookup. A valid actor requires one enabled
  published pair; an outside-window actor returns no projection.
- Projection performs no allocation, actor/schedule/presentation mutation, terrain query, source
  read, GPU command, frame, fence wait, or synchronization.

## Metrics

- Exact active ordinal, semantic region, Q9 window coordinates, Q16 heights, denominators, actor,
  and published-config equality.
- Focused branch coverage for center/corners, seams, far signed coordinates, origin aliases,
  rollover, outside, empty, stale, and replay cases.
- Real-process projection operation counters, setup publication count, result/replay SHA-256, and
  elapsed time.
- `runseal :guard` result and Flavor denies.

## Acceptance Criteria

- Exactly one renderer-owned projection composes existing signed addressing and canonical terrain
  projection; there is no parallel float-world transform or alternate scene path.
- Center/corner/seam/far-coordinate projections are exact and origin alias/rollover invariant.
  Outside-window input fails explicitly without clipping or fallback.
- Returned actor, presentation, generation, Q16 heights, and published config are unchanged copies.
  Empty/stale generation fails before composition lookup and all operations are mutation-free.
- The inspect surface is exactly one strict verb with no compatibility alias or hidden mode.
- Focused checks, the short process gate, and `runseal :guard` pass. Existing setup may publish the
  canonical pair, but each projection performs zero source, GPU, frame, fence, or synchronization
  work and the long canonical workflow is not repeated.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain, reference Windows workbench, one freshly
published signed canonical pair, and the current capacity-one runtime actor.

## Evidence

The implementation adds one allocation-free signed active-index lookup and one exact integer Q9
operation to the existing canonical terrain projection. A renderer-owned transaction requires the
enabled published pair, checks that its local/global configs agree, projects the copied live actor,
and returns no mutation capability. `Runtime::project_actor` validates the generation first; the
strict `actor.project` verb is its only inspect consumer.

All 69 `engine-runtime` tests passed, including new center/four-corner, Q9 range, signed seam,
`2^40`, origin alias/rollover, outside-window, and divergent-config cases. Workbench compiled and
the affected engine-runtime/workbench/prototype clippy gate passed with warnings denied. The new
TypeScript support and canonical wrapper type-check.

The first fresh-source evidence run exposed one harness error after every projection assertion had
passed: it compared presentation status across two canonical publications, which legitimately
change that status. The assertion window was corrected to compare immediately before/after only
projection operations; no runtime implementation changed. The final warm gate passed in 6,608.4
ms and stopped every process.

The live gate published the same signed center under canonical and `(96,32)` origin aliases. At
`(2^40,-2^40)` the controlled actor projected to active index 8, semantic region 8129, and exact Q9
`[8265,-8283]` in both aliases. It also proved edge index 4 / semantic 8002 / Q9
`[20479,-20480]`, pre-publication, empty, malformed, stale, and outside-window rejection, exact
actor rollback, and unchanged simulation/presentation state around projection operations. Result
and immediate replay SHA-256 were both
`72f7939b9b7cd945606b3fd537ec98ea9f8ecf9b3ed73779c1c6755d646ad688`.

The setup publications use the already accepted canonical source/GPU path, but each projection
reports zero allocation, terrain query, source read, GPU copy/readback, fence wait,
synchronization, mutation, frame, or renderer work. No resource, descriptor, shader, frame,
synchronization, or lifecycle implementation changed, so the long canonical workflow was not run.

The first merge guard then rejected five Flavor issues introduced by the new files: actor support
and composition/rendering shelves exceeded their ownership limits, and two test names exceeded the
configured vocabulary. The actor gates were moved directly under one `support/actor` owner, the
projection moved directly under the renderer owner, test names were shortened, and no old path or
threshold exception was retained. A following guard exposed two stale explicit Deno-check paths;
they were replaced directly. `runseal :init` passed in 0.25 seconds and the final guard passed in
3.72 seconds with zero Flavor denies.
