# ADR 0096: Exact Canonical Object Query

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0093 Exact Canonical Object Query

## Context

Schema-3 object streaming already verifies and decodes exact spatial records, authored local IDs,
and presentation records before copying all three planes into a source-addressed 50-slot GPU cache.
The decoded CPU values are then discarded. Canonical probes can reconstruct them only through an
explicit GPU readback, so the runtime has no read-only CPU object authority suitable for later
gameplay policy.

Reading the pack on demand would bypass committed-snapshot lifetime and make source I/O part of a
query. Reading back the GPU would add synchronization and make diagnostics a gameplay dependency.
A separate CPU scene would create another publication and identity authority.

## Decision

- Move each verified decoded schema-3 page into a CPU page owned by the existing asynchronous object
  cache after filling the GPU upload arenas. Keep the same fixed 50 physical slots and source/region
  reservation.
- Use shared immutable page references. A pending transfer owns a complete next cache candidate;
  copy completion admits it to residency, and the existing staged/commit/discard pair path controls
  which 25 active pages become query-visible.
- Preserve prior published page references across pending, failed, prefetched, and source-switch
  work. Cache residency alone never grants query authority.
- Expose `Runtime::query_canonical_object(RegionCoord, authored_local_id)`. Require a committed
  snapshot, an in-window signed region, and a local ID below 1,024; validate the page region, triple
  shape, unique match, and canonical stable seed before returning the exact authored triple.
- Return raw region-local float position/height and `PresentationRecord`. Do not infer fixed-point
  actor position, grounding, proximity, collision, semantics, or interaction eligibility.
- The successful query performs no allocation, source I/O, GPU copy/readback, fence wait, or
  synchronization. The workbench verb and independent pack parser are acceptance surfaces only;
  Prototype remains non-diagnostic.
- Authored local IDs remain scoped to one signed region and current object source. They are not
  persistent gameplay or network identifiers.

## Consequences

- The runtime gains one exact CPU object lookup over the same committed content generation rendered
  by the GPU, without a second scene or query-time data movement.
- CPU resident payload is bounded to 50 × 40,960 bytes = 2,048,000 bytes, plus fixed vector and
  shared-reference metadata. Published and staged snapshots share pages rather than copying them.
- Physical triple reordering is invisible to lookup because authored local ID, not physical ordinal,
  selects the result.
- Later interaction may explicitly choose spatial conversion and selection policy above this
  authority. This ADR does not choose those boundaries.

## Evidence

Experiment 0093 passes all 86 runtime tests, `canonical-frame-v2` in 16.131 seconds, and the complete
`canonical-runtime-v1` workflow in 807.8 seconds. Independent source-byte comparisons prove local
IDs 0/511/1023, physical order A/B equality, adjacent-window replacement, failed object/terrain pair
retention, and restart equality with zero query-side work.

The 64-publication plateau peaked at its 531-handle baseline and finished at 516 handles with private
bytes below baseline; six final 10-second samples were identical. Both 32-crossing traversal sweeps
and all 16 lifecycle cycles passed with zero remaining Sidecar processes.
