# Experiment 0046: Exact Terrain Body Contact

Status: Accepted

## Hypothesis

The canonical runtime can resolve one caller-owned, bounded vertical body against the exact height
of the currently committed terrain snapshot, producing strict separated/touching/penetrating
classification and the minimum upward non-penetration correction without persistent identity,
simulation time, source I/O, GPU access, allocation, or dependence on render LOD and camera state.

## Scope

This experiment adds one value-type body with an exact signed-region/local-Q9 horizontal position,
signed Q16 center height, and positive Q16 half-height. `Runtime` resolves that value against one
exact terrain query and returns the sampled terrain, initial foot separation, classification,
non-negative upward correction, and resolved body value.

The input body remains caller-owned and has no runtime ID, lifetime, storage slot, velocity,
acceleration, orientation, radius, or presentation. The resolver never moves a separated body
downward: touching means exact zero separation, penetration receives only the exact minimum upward
correction, and positive separation remains unchanged.

Terrain normal, slope, material, multi-point footprint, step handling, swept collision, dynamic or
object collision, gravity, jump, locomotion, simulation time, input mapping, camera behavior,
runtime actors, and gameplay are out of scope. The existing `lod-terrain-contact-v1` probe remains
visual LOD error evidence and does not become body-contact authority.

## Workload

1. Define the bounded body, strict contact classification, and resolved-contact result in Q16.
   Cover separated, touching, penetrating, invalid/non-positive half-height, resolved-height
   overflow, extreme arithmetic, both terrain triangles, the diagonal, half-open horizontal bounds,
   adjacent-region seams, and signed far regions.
2. Expose one read-only `Runtime` resolver over the committed terrain snapshot and one diagnostic
   `canonical.terrain.contact` command. Pre-publication, outside-window, inconsistent snapshot, and
   unrepresentable result failures must remain explicit and must not mutate the input body.
3. At one explicit acceptance checkpoint, use the independent exact grounding oracle at all 76,800
   deterministic points in the published 5x5 window to construct bodies whose feet are one Q16
   numerator below, exactly on, and one numerator above terrain. Resolve all 230,400 bodies and
   record classification/correction counts, exact result and identity hashes, mismatches, and
   elapsed CPU time.
4. Carry a compact 225-body witness covering all 25 regions, all three terrain triangles, and all
   three contact classifications through object reorder, revisit, compensated alias, movement
   return, process restart, all four held I/O/copy gates, both failed pair publications, prepared
   rollover, and 32 reactive plus 32 prepared traversal publications. Do not repeat the dense sweep
   from every generic composition probe.
5. Require held and failed pair operations to resolve against the old committed terrain exactly;
   only a successful atomic pair publication may change contact terrain and result hashes.
6. Run focused tests, `runseal :guard`, then the complete direct canonical GPU, prototype, query,
   failure, resource-plateau, and 16-cycle lifecycle workflow.

## Controlled Variables

- Horizontal identity is exactly the accepted `TerrainQueryPosition`: signed `i64` region plus
  half-open local X/Z Q9 in `[-4096, 4096)`.
- Vertical body center and half-height use the terrain height denominator 65,536. Public body
  components are bounded to signed `i32`, half-height must be positive, and all intermediate
  separation/correction arithmetic uses checked `i64` before the resolved center is converted back.
- Classification has no epsilon or skin: positive separation is separated, zero is touching, and
  negative is penetrating. Only penetration changes the returned body, by exactly `-separation`.
- One resolution performs one committed-snapshot height query and fixed integer arithmetic. It
  performs no allocation, file read, worker dispatch, GPU command/copy/readback, fence wait, lock,
  or synchronization operation.
- The resolver owns no body state and does not execute from `Runtime::frame`. Hosts and prototype
  loops do not yet sample input, advance simulation, or retain the returned body.
- Signed schema-3 presentation, terrain/object caches, atomic composition, traversal, timeline,
  neutral frame targets, rendering, prototype bootstrap, and fixed animation bank remain unchanged.

## Metrics

- Focused test count and exact rejection outcomes for invalid, unavailable, outside-window,
  inconsistent-snapshot, overflow, and horizontal-bound cases.
- Dense region/position/resolution counts, per-classification and per-correction counts, result and
  signed-identity SHA-256 values, oracle mismatch count, first mismatch, and elapsed CPU time.
- Contact generation and hashes before/during/after reorder, alias, hold, rollback, traversal,
  rollover, restart, and lifecycle operations, using the compact transition witness.
- Per-resolution allocation bytes, source reads, GPU copies/readbacks, fence waits, and
  synchronization count.
- Existing controlled GPU/query hashes, traversal outcomes, resource plateau, and lifecycle data.

## Acceptance Criteria

- Public inputs and outputs are typed exact values. Half-height is strictly positive; every
  intermediate and resolved value is checked, and invalid or unrepresentable bodies fail without
  clamp, wrap, float conversion, fallback, or input mutation.
- The three classifications and correction rule match exact signed separation for both triangles,
  the diagonal, seams, and far signed regions. Separated and touching bodies are byte-identical to
  input; penetrating bodies move upward by exactly the penetration magnitude and end touching.
- The dense sweep performs exactly 230,400 resolutions with 76,800 separated, 76,800 touching,
  and 76,800 penetrating inputs; only the penetrating third has non-zero correction. It reports
  zero oracle mismatches, allocations, source reads, GPU copies/readbacks, fence waits, and
  synchronization.
- The routine 225-body witness reports 75 results in each classification and 75 corrections. Its
  exact result and signed-identity hashes participate in every stable comparison without turning
  dense acceptance into recurring probe cost.
- Resolution before publication, outside the active window, against inconsistent identity, or with
  an unrepresentable result fails explicitly. Held and failed publication never expose staged
  terrain; successful pair publication changes contact evidence atomically.
- Reorder/revisit/restart reproduce exact content results. Alias, 32+32 traversal, and rollover
  preserve exact results while signed identity hashes follow committed ownership.
- `runseal :guard`, all prior canonical GPU/prototype/query gates, the resource plateau, and 16
  lifecycle cycles pass without controlled-hash change, resource growth, validation error, or
  device removal.

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
`out/captures/0046-exact-terrain-body-contact/`.

## Results

- Focused runtime/workbench tests passed 25 engine tests, including exact separated, touching, and
  penetrating behavior, positive half-height validation, denominator rejection, triangle
  preservation, extreme arithmetic, and unrepresentable resolved-center rejection. `runseal
  :guard` passed with zero deny issues.
- The short runtime gate rejected pre-publication, zero/negative half-height, invalid local bound,
  outside-window, and resolved-center overflow requests. Its three direct samples returned exact
  penetrating/touching/separated results without allocation, source read, GPU copy/readback, fence
  wait, or synchronization work.
- The explicit dense checkpoint resolved 230,400 bodies: 76,800 separated, 76,800 touching, 76,800
  penetrating, and 76,800 corrected. Oracle mismatch count was zero. The accepted run took
  617,976,600 ns inside the probe and produced result SHA-256
  `ea383efa16692b18f263e8be99e7ebc15e15ed23c4e3efb640520b8d33c9d091` and signed-identity
  SHA-256 `6ffebc40ce99c3b285b07179946cbe20323713830648eef9ea821454c60948ce`.
- The routine witness resolved 225 bodies with 75 in each classification and 75 corrections. Its
  result SHA-256 was `2cd0d7110b580d58d3835f38e44a77ed3339ba028225e6cf7e2a5da590464306` and its
  signed-identity SHA-256 was
  `16446f145eecf59e79dda0a30f8193bf9240b5d93103b95c7c0c6c4aa7e15c9a`. It remained exact across
  four held gates, both failed publications, reorder/revisit/alias/return/restart/rollover, 32
  reactive crossings, 32 prepared crossings, and 16 complete lifecycle cycles.
- The one final direct workflow passed in 701.5 seconds. The resource plateau held 531 baseline and
  peak handles with zero transient growth; the last sample had 516 handles, 409,477,120 private
  bytes, and 18 threads. Existing controlled color, PNG, object-ID, diagnostic, shadow, terrain,
  presentation, and lifecycle gates remained exact.

## Conclusion

The hypothesis is accepted. The runtime now has one exact caller-owned vertical body-contact
transaction over the committed CPU terrain snapshot. It adds no runtime actor, persistent body
state, simulation time, gravity, velocity, locomotion, input policy, camera coupling, rendering
dependency, fallback, or compatibility surface. Dense proof is an explicit acceptance action;
routine state transitions retain only the bounded deterministic witness needed to prove atomic
snapshot observation.
