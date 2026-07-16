# Experiment 0096: Exact Canonical Object Nearest

Status: Accepted

## Hypothesis

The bounded CPU pages already owned by the committed canonical object snapshot can answer one exact
planar nearest-object query by scanning their fixed capacity once, without introducing a global
registry, source access, GPU work, allocation, or gameplay interaction policy.

## Scope

- Add one read-only query from an exact `TerrainPosition` origin and inclusive `u32` Q9 radius.
- Require the origin region to belong to the current committed active window.
- Scan all 25 active CPU pages and all 1,024 triples per page, for a fixed maximum of 25,600
  candidates.
- Convert every candidate through the accepted checked object-to-`TerrainPosition` contract.
- Return the candidate count and one optional nearest object, exact terrain position, signed planar
  Q9 deltas, and squared Q18 distance.
- Break equal-distance ties by owner region X, owner region Z, then authored local ID.
- Add the strict diagnostic `canonical.objects.nearest` route and an independent `.wlr` byte oracle.

Enumeration results, a persistent spatial index, facing or line-of-sight tests, renderer visibility,
selection state, interaction eligibility or input, collision/navigation, persistent identity,
networking, multiple actors, and Wulin semantics are out of scope.

## Workload

1. Prove physical-order-independent selection and a zero-radius four-page seam tie.
2. Prove exact inclusive radius behavior and the valid no-result case.
3. Prove the 25,600-candidate bound, signed far coordinates, and `u32::MAX` radius without overflow.
4. Reject pre-publication, malformed/outside origins, incomplete pages, duplicate local IDs, and
   non-lattice candidate positions.
5. Decode all active schema-3 pages independently and compare exact candidate count, nearest raw
   object, terrain position, deltas, distance, and tie ordering.
6. Preserve results through physical order A/B, source revisit, adjacent-window replacement,
   corrupt object/terrain rollback, and process restart.
7. Preserve the exact canonical GPU frame and immediate replay.
8. Run the optimized complete acceptance through both 32-crossing traversal sweeps, the bounded
   same-process resource checkpoint, and two lifecycle cycles.

## Controlled Variables

- Schema-3 format, source namespace, 50-slot cache, CPU page ownership, GPU mapping, pair
  publication, renderer submission, and source/terrain failure semantics remain unchanged.
- The scan consumes only immutable pages referenced by the published snapshot and allocates no
  query-side collection or index.
- Distance uses checked signed-region deltas in `i128`, applies the radius bound before squaring,
  and returns only values proven to fit the public `i64`/`u64` fields.
- Identity remains `(owner region, authored local ID)` even when positive-edge normalization gives
  the candidate a different spatial region.
- GPU visibility, occlusion, semantic IDs, presentation state, and actor state do not filter or
  rank candidates.
- The workbench response has one current revision; no compatibility alias or fallback is retained.

## Metrics

- Candidate count, exact selected identity/position/deltas/squared distance, and maximum capacity.
- Independent source-byte equality, physical-order equality, radius inclusion, tie result, strict
  failures, revisit, rollback, adjacent publication, and restart.
- Per-query allocation, source-read, GPU-copy/readback, fence-wait, and synchronization counters.
- Existing frame hashes, traversal counts, handles/private bytes/threads, process cleanup, workflow
  time, operation counts, and artifact bytes.

## Acceptance Criteria

- Every successful query validates and scans exactly the committed active pages once and never
  exceeds 25,600 candidates.
- The nearest result is the independent oracle minimum under
  `(distance squared, owner X, owner Z, authored local ID)` and the radius is exact and inclusive.
- Zero radius, no result, far signed coordinates, seam normalization, and maximum radius behave
  without float global coordinates, rounding, clamping, or overflow.
- Invalid publication, origin, page, identity, or lattice state fails without partial output or
  fallback work.
- A/B order, revisit, adjacent movement, both failed pair types, and restart preserve exact results.
- Every successful process query reports zero added work and the accepted GPU frame remains exact.
- No enumeration surface, retained index, interaction/gameplay policy, compatibility route,
  resource owner, asset/format change, networking, or Wulin boundary is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3
sources centered at `(2^40, -2^40)`, physical orders A/B, and the maintained canonical frame/runtime
workflows.

## Evidence

The engine-runtime test set passes all 95 tests. Four focused renderer tests prove order-independent
zero-radius seam selection, inclusive/no-result radii, the full 25,600 capacity at ordinary and
signed-edge coordinates, outside-origin rejection, and malformed snapshot/page/lattice failures.

`canonical-frame-v4` passed in 13.639 seconds. The independent source parser scanned all 25,600
triples per query. At the exact negative seam, four spatially coincident candidates resolved to
authored ID 1023 owned by `(baseX-1, baseZ-1)` under the declared tie. A one-Q9 displaced origin
returned the same object only at inclusive radius one. A center query selected ID 496 at delta
`(160,-32)` and squared distance 26,624; `u32::MAX` returned the same minimum. Pre-publication,
malformed-origin, and outside-window requests failed strictly. Every query reported zero allocation,
source read, GPU copy/readback, fence wait, and synchronization. First/replay color hash remained
`8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135`; object-ID hash remained
`01951615d1b4645bdfba68991c75b8ea333482d312f31f39ed3b907ca479da5b`.

`canonical-runtime-v5` passed in 251.987 seconds. Twenty-eight accepted and three rejected live
nearest events covered A/B order, revisit, adjacent publication, both failed-pair retentions, and
restart while both traversal sweeps completed 32 samples. The resource checkpoint first reached a
state-driven workload baseline after five publications, then held 492 handles and 21 threads across
all measured samples; final private bytes were 503,808 below baseline under the unchanged 16 MiB
allowance. Both lifecycle cycles cleaned up. The report retained 117 probes, 32 observations, six
captures, 24 files, and 25,346,305 bytes.

The full-workflow resource correction is independently covered by nine injected policy tests. The
routine workflow has no second-scale fixed wait; the maintained deep resource owner retains the
32-warm/64-measured/60-second-recovery/16-cycle soak. Repository guard passes with zero Flavor deny
issues.

## Conclusion

Accepted. The committed canonical object snapshot now provides one bounded exact nearest result in
the sole terrain-position domain. It does not establish enumeration, interaction, visibility, or
persistent object authority.

## Promotion

Promoted `CanonicalObjectNearest`, `Runtime::query_nearest_canonical_object`, the fixed 25,600 scan,
strict diagnostic route, independent source-byte oracle, and maintained acceptance evidence.
Promoted no spatial index, selection state, action policy, compatibility route, new resource,
format/asset change, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime --release
deno test .runseal/support/resource-acceptance_test.ts
runseal :canonical-frame
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-frame/` and
`out/captures/canonical-runtime/`.
