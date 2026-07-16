# Experiment 0099: Typed Canonical Object Resolution

Status: Accepted

## Hypothesis

A source-qualified canonical object identity can be resolved on demand against the sole committed
CPU snapshot with typed source-replacement and window-departure outcomes, allowing a later product
consumer to recognize ordinary lifetime loss without parsing errors, retaining copied content, or
adding recurring work.

## Scope

- Add one tagged `CanonicalObjectResolution`: `resolved`, `source-replaced`, or
  `outside-published-window`.
- Replace `Runtime::query_canonical_object` with `Runtime::resolve_canonical_object`; retain no old
  method, overload, alias, fallback, or workbench verb.
- Replace `canonical.objects.query` with strict `canonical.objects.resolve` and expose the typed
  result plus an optional terrain position only for `resolved`.
- Preserve hard failures for pre-publication access, out-of-range authored IDs, malformed committed
  snapshot/page shape, duplicate or missing authored IDs, and record/identity divergence.
- Reuse the committed immutable CPU pages with zero allocation, source I/O, GPU work, wait,
  synchronization, publication mutation, or automatic reload.

Retained prototype targets, automatic resolution cadence, selection/disappearance policy,
highlighting, interaction, gameplay-persistent IDs, networking, and Wulin semantics are out of
scope.

## Workload

1. Prove exact resolution remains independent of physical triple order.
2. Prove all three typed outcomes and strict invalid-ID, malformed page, missing/duplicate ID, and
   pre-publication failures.
3. Compare every resolved object and terrain position with the independent schema-3 source oracle.
4. Reject missing namespace payload and the retired `canonical.objects.query` verb.
5. Publish source A, replace with B, resolve stale A as `source-replaced`, revisit A, resolve stale B
   likewise, and resolve current A exactly.
6. Move the active window while retaining the same source and resolve a departed owner region as
   `outside-published-window`.
7. Preserve object/terrain failed-publication rollback, restart, nearest results, GPU replay,
   traversal, bounded resources, and lifecycle cleanup.

## Controlled Variables

- Schema 3, namespace derivation, source-addressed cache, CPU/GPU page lifetime, atomic pair
  publication, nearest scan, prototype observation, renderer, formats, and assets remain unchanged.
- Resolution validates caller ID range and committed snapshot shape before returning a nonfatal
  lifetime outcome.
- Source replacement is checked before region/page lookup; window departure is checked only when
  the identity's source remains current.
- No resolver call is added to the prototype or any frame loop.

## Metrics

- Resolution outcome, exact object identity/content, terrain position, and failure error family.
- Resolver allocation/source-read/GPU-copy/readback/fence/synchronization counters.
- A/B source outcomes, adjacent-window outcome, rollback/restart equality, GPU hashes, traversal
  samples, process resources, lifecycle cycles, workflow time, and artifact bytes.

## Acceptance Criteria

- A current in-window valid identity returns `resolved` with the same independently verified object.
- A valid identity from another source returns `source-replaced` without accessing replacement
  content; a valid current-source identity outside the active window returns
  `outside-published-window`.
- Invalid authored IDs and malformed/missing/duplicate committed data remain failures rather than
  nonfatal outcomes.
- The old Runtime method and workbench verb are absent and explicitly guarded against restoration.
- Existing exact/nearest, rollback, restart, traversal, GPU, resource, and lifecycle evidence stays
  exact with zero resolver-side work.
- No retained target, recurring resolution, registry, gameplay identity, interaction, new resource,
  format/asset change, networking, or Wulin behavior is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 A/B
packs centered at `(2^40, -2^40)`, and maintained focused/full acceptance workflows.

## Evidence

All 95 engine-runtime tests pass. The private resolver suite proves physical-order equality, all
three outcomes, caller-ID validation, malformed snapshot/page rejection, missing/duplicate authored
ID rejection, and unchanged bounded nearest behavior. All prototype/workbench tests pass; strict
Clippy and selected Deno type checks report no issue.

`canonical-frame-v6` passes in 18.164 seconds. Pre-publication and authored ID 1024 fail in the
resolution error family; a zero namespace returns `source-replaced`, a same-source region outside
the radius-2 window returns `outside-published-window`, and five exact samples return `resolved`.
The old verb returns `unknown_event`. Independent namespace is `84e88c79…99d9`; first/replay color
hash remains `8b13d214…4135` and object-ID hash remains `01951615…da5b`.

The final-worktree `canonical-runtime-v7` passes in 249.862 seconds. Source A
`e7f1045b…2b56` and B `1cd5cc78…e3a` both make the opposite identity `source-replaced`; A revisit
resolves, adjacent departure returns `outside-published-window`, and both failed publications plus
restart retain `resolved`. The workflow records 30 successful resolutions, three expected hard
failures, one old-verb rejection, 32+32 traversal samples, and two lifecycle cycles.

Resource convergence uses six warm and eight measured publications. Handles remain 492, threads
remain 21, and private bytes move from 424,329,216 to 424,501,248 (+172,032) under the unchanged
16 MiB allowance. The report retains 24 files and 25,346,200 bytes. Repository guard passes with
zero Flavor deny issues.

## Conclusion

Accepted. A source-qualified address now has an explicit on-demand lifetime resolution contract;
ordinary source/window loss is typed, while invalid caller or committed state remains fatal.

## Promotion

Promoted `CanonicalObjectResolution`, the sole `Runtime::resolve_canonical_object` facade, strict
`canonical.objects.resolve`, typed A/B/window gates, and a removal guard. Promoted no old query
compatibility, retained target, recurring work, registry, gameplay identity, interaction, new
resource, format/asset change, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype -p workbench
cargo clippy --locked -p engine-runtime -p prototype -p workbench --all-targets -- -D warnings
runseal :canonical-frame
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-frame/` and
`out/captures/canonical-runtime/`.
