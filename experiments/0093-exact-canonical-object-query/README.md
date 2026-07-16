# Experiment 0093: Exact Canonical Object Query

Status: Accepted

## Hypothesis

The existing verified schema-3 object triples can remain as a bounded CPU resident projection of
the source-addressed object cache and publish atomically with the existing GPU pages. A runtime
caller should then be able to look up one exact authored object in the committed snapshot without
source I/O, GPU work, synchronization, a second scene, or gameplay selection policy.

## Scope

- Retain each decoded spatial/identity/presentation page in the existing 50-slot asynchronous
  object cache after its bytes have filled the GPU upload arenas.
- Carry shared references to the 25 active CPU pages through the same copy completion, staged
  publication, pair commit, discard, reuse, and rollback path as the GPU mapping.
- Expose one read-only lookup by signed `RegionCoord` plus authored local ID in `0..1024`.
- Return the exact region-scoped identity, raw region-local position and authored height, and the
  schema-3 presentation record.
- Add one strict diagnostic `canonical.objects.query` verb and an independent `.wlr` byte oracle
  for acceptance. The non-diagnostic prototype remains unchanged.

Spatial selection, conversion to actor fixed-point position, interaction input/policy, collision,
navigation, sparse occupancy, persistent gameplay/network IDs, ECS ownership, multiple actors, and
Wulin semantics are out of scope.

## Workload

1. Prove pure lookup of local IDs 0, 511, and 1023 through two distinct physical triple orders.
2. Reject pre-publication lookup, local ID 1024, an outside-window region, a page with the wrong
   signed region, and malformed triple-plane lengths.
3. In a real workbench process, parse the signed schema-3 header, index, identity plane, spatial
   record, and presentation record independently and compare all returned fields exactly.
4. Switch from physical order A to B and back, requiring identical identity-keyed query results and
   zero-copy cache reuse on the order-A revisit.
5. Publish one adjacent window; require the retired edge region to fail and the newly admitted edge
   region to match its source bytes.
6. Fail one corrupt object pair and one corrupt terrain pair, requiring the prior committed object
   query to remain exact after both rollbacks; then restart and reproduce the base queries.
7. Preserve the accepted canonical GPU frame and immediate replay.
8. Run the full canonical workflow across holds, rollover, 32 reactive plus 32 prepared crossings,
   the warmed 64-publication resource plateau, and 16 complete lifecycle cycles.

## Controlled Variables

- Schema-3 bytes, signed addressing, source namespaces, physical cache capacity, GPU resources,
  descriptor layout, copy counts, fences, terrain-first pair publication, renderer submission, and
  probe readback authority remain unchanged.
- CPU pages use the same cache slot assignment as GPU pages. New uploads move their decoded vectors
  into one `Arc`; retained and published generations clone only bounded references, not payloads.
- The cache owns at most 50 decoded 40,960-byte triple pages, or 2,048,000 payload bytes. A
  publication references only its existing active pages and cannot mutate them.
- Authored local IDs remain a complete region-local permutation, not a persistent public object
  handle. Lookup scans the fixed 1,024-entry identity plane and allocates nothing on success.
- Source switches and valid prefetch work may retain cache pages without making them query-visible;
  only the committed snapshot is public.

## Metrics

- Exact query object fields and independent source-byte equality.
- Rejection codes, physical-order equality, old/new active-window behavior, rollback, and restart.
- Per-query allocation, source read, GPU copy/readback, fence-wait, and synchronization counts.
- Existing object triple copy counts, canonical capture/shadow/occlusion hashes, traversal counts,
  handles, private bytes, threads, and process cleanup.

## Acceptance Criteria

- All successful queries match independently decoded source bytes for the requested region/local ID
  and report zero query-side allocation, source, GPU, fence, or synchronization work.
- Invalid identity/window/publication states fail without fallback to source I/O or GPU readback.
- Physical order A/B, cache revisit, adjacent movement, failed pairs, and restart never expose a
  mismatched or premature CPU page.
- Existing GPU resources, copy counts, frame hashes, submission, and synchronization remain exact.
- The same-process resource sample does not grow beyond its accepted handle/private-memory bounds,
  and all 16 lifecycle cycles leave no descendants.
- No gameplay selection, action, interaction, position conversion, compatibility alias, alternate
  scene, format change, product telemetry, networking, or Wulin behavior is added.

## Reference Environment

The experiment uses the pinned Rust and Deno toolchains, Windows D3D12 reference runtime, signed
far-coordinate sources centered at `(2^40, -2^40)`, schema-3 physical orders A/B, and the maintained
canonical frame/runtime workflows.

## Evidence

All 86 `engine-runtime` tests pass. Three new pure tests prove physical-order-independent lookup,
strict identity/window rejection, and page-region/triple-shape validation. `workbench` compiles and
Rust/Deno formatting and type checks pass.

`canonical-frame-v2` passed in 16,130.7382 ms. Pre-publication, local-ID-1024, and outside-window
requests failed strictly. Local IDs 0, 511, and 1023 matched the independent schema-3 byte oracle,
each with zero query allocation/source/GPU/fence/synchronization work. The existing canonical frame,
capture, shadow, occlusion, and immediate replay evidence remained exact.

The full `canonical-runtime-v1` workflow passed in 807.8 seconds. Order A/B and the zero-copy A
revisit returned identical queries. An adjacent publication retired the old left edge and admitted
the exact new right edge. Corrupt object and terrain candidates preserved local ID 511 from the old
committed snapshot, and restart reproduced 0/511/1023. Both 32-crossing traversal sweeps passed.

After 32 warm publications, all six 10-second quiescent samples held 531 handles and 413,949,952
private bytes. Across 64 sampled publications the peak remained 531 handles; the final sample was
516 handles and 412,336,128 bytes. All six final quiescent samples were identical, and 16 complete
lifecycle cycles left the development, benchmark, bootstrap, and prototype namespaces stopped with
zero PIDs.

## Conclusion

Accepted. The canonical object cache now retains one bounded CPU copy of each verified resident
schema-3 page and publishes exact active-page references with the existing GPU snapshot. Runtime
callers can query one authored triple from the committed pair without source or GPU work.

## Promotion

Promoted the 50-slot CPU triple residency, shared page lifetime through the existing transfer and
pair publication, `CanonicalObject`, `Runtime::query_canonical_object`, strict workbench query, and
maintained independent oracle evidence. Promoted no selection or interaction policy, persistent
object handle, position conversion, second scene, compatibility layer, asset/format change,
networking, or Wulin behavior.

## Reproduction

```powershell
runseal :canonical-frame
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/canonical-frame/` and
`out/captures/canonical-runtime/`.
