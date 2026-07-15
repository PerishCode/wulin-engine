# Experiment 0054: Transactional Retained-Body Advance

Status: Accepted

## Hypothesis

One exact live terrain-body generation can execute the accepted planar-first copied-value advance
and commit only its successful output back into the runtime-owned slot, while stale handles fail
before terrain lookup and every query/arithmetic/contact failure preserves the retained motion
exactly, without elapsed-time driving, input mapping, multi-body storage, presentation, or rendering.

## Scope

The runtime reads one `RetainedTerrainBody` by exact generation, passes its copied motion to the
Experiment 0052 transaction, and commits the resulting output under the same generation only after
the full transaction succeeds. The result exposes retained input, complete planar/vertical
intermediate evidence, and retained output.

No generic body setter is added. Spawn/read/despawn remain Experiment 0053's lifecycle authority;
the only new mutation is a successful canonical terrain advance. The operation does not consume or
inspect the simulation schedule, and callers still supply one tick's signed Q9 displacement,
nonnegative Q16 step-up limit, and Q16 vertical acceleration explicitly.

Live elapsed time, schedule-batch iteration, stall/focus policy, host input, horizontal velocity,
jump clearance, swept collision, multi-actor capacity, presentation identity, camera behavior,
and gameplay tuning remain out of scope.

## Workload

1. Add a private checked slot replacement that requires the exact live generation and preserves
   the handle. Prove wrong/empty replacement rollback directly.
2. Add one `Runtime` retained-advance transaction that reads before query, calls the accepted
   planar-first advance once, and commits only its final output after success.
3. Prove accepted, blocked, downhill, zero, and seam results mutate the retained motion exactly as
   the copied-value oracle while preserving generation and one/two-query behavior.
4. Prove stale/empty handle rejection performs no query. Prove negative limit, unavailable
   snapshot/destination/origin, coordinate, velocity, and contact failures retain exact state.
5. Expose one strict diagnostic verb and run a fresh-process committed-snapshot sequence plus
   immediate replay. Require exact state/hash, unchanged schedule/presentation, and zero source,
   GPU, fence, synchronization, frame, or renderer work per operation.
6. Run focused tests and `runseal :guard`; add the gate to the live canonical wrapper without
   executing the long GPU/lifecycle workflow for this frame-independent CPU mutation.

## Controlled Variables

- Capacity and lifecycle remain exactly Experiment 0053: one optional slot and one checked nonzero
  generation.
- Spatial computation remains exactly Experiment 0052: planar translation first, destination
  sample reuse for accepted/zero movement, and ordered destination/origin lookup for a distinct
  blocked candidate before one vertical step.
- Stored input is copied before spatial computation. No slot mutation occurs until a complete
  `TerrainBodyAdvance` exists.
- Commit changes only `motion`; the handle generation remains exact. Failed commit validation does
  not expose or apply the computed output.
- The simulation schedule and presentation timeline are neither read nor mutated.

## Metrics

- Focused test count and exact retained input/output, generation, query order, and rollback checks.
- Short-process accepted/blocked/downhill trajectory evidence, failure-state readback, result/replay
  SHA-256, and elapsed time.
- Allocation, terrain query, source-I/O, GPU, fence, synchronization, schedule, presentation,
  frame, and renderer counters plus `runseal :guard` results.

## Acceptance Criteria

- The exact live generation is validated before terrain lookup; empty/stale handles perform zero
  queries and leave the slot unchanged.
- Successful computation commits exactly `advance.output` under the same handle. Returned retained
  input/output and the complete copied-value evidence agree byte-for-byte.
- Every validation/query/arithmetic/contact failure returns no retained advance and preserves the
  previously readable retained value exactly.
- No generic setter, hidden schedule consumption, alternate terrain/contact path, host-owned body,
  multi-slot store, or rendering dependency is introduced.
- Focused tests, one short real-process gate, and `runseal :guard` pass with unchanged simulation
  and presentation authorities and zero non-CPU work. The long canonical workflow is not repeated
  unless a later merge candidate changes a consumer of frame/GPU/resource evidence.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain, committed CPU terrain snapshot, and
reference Windows workbench. The renderer remains only the existing snapshot owner and does not
consume the retained body.

## Evidence

The implementation adds a private checked `TerrainBodySlot::replace` and one
`Runtime::advance_retained_terrain_body` transaction. Runtime first copies the exact live retained
value, invokes the existing `advance_terrain_body` owner, and calls checked replacement only after
the complete result exists. The public result links retained input/output to the full copied-value
advance; no setter, alternate query/contact path, schedule read, or renderer dependency exists.

All 56 focused `engine-runtime` tests passed. Two new slot tests prove that successful replacement
changes only motion and preserves generation, while empty/wrong handle replacement leaves the slot
exact. The existing six planar-first tests continue to cover accepted, downhill, blocked, seam,
one/two-query order, validation, snapshot, arithmetic, and contact failures. Workbench and strict
protocol compilation passed.

The final fresh-process gate passed in 23,363.0 ms across two independent full process runs. Before
source publication, empty and stale generations rejected before snapshot lookup, negative limit
validation preserved the stored value, and malformed generation failed strictly. On the committed
snapshot, accepted 128-Q16 uphill and same-tick downhill used one query, blocked uphill used two,
and each committed output matched the copied-value oracle under the unchanged generation.
Out-of-window lookup and vertical-velocity overflow returned no retained advance and exact readback
proved rollback. Result and replay SHA-256 were both
`54dacac84b69c1ef1e98d127de23e646b0d18e6c9934e50d3e832abefa56f529`.

Simulation and presentation status remained unchanged; all declared allocation, source-I/O, GPU,
fence, synchronization, frame, and renderer counters were zero. `runseal :guard` passed with zero
Flavor denies and all repository suites green. The live wrapper retains the new gate. The long
canonical workflow was not run because no frame, renderer, GPU-resource, synchronization, or
existing lifecycle consumer changed.
