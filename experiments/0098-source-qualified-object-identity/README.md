# Experiment 0098: Source-Qualified Object Identity

Status: Accepted

## Hypothesis

The exact `.wlr` source namespace already committed with the CPU object pages can qualify every
owner-region/local-ID address, preventing stale references from silently aliasing another source
without adding a registry, allocation, source read, or gameplay-persistent identity.

## Scope

- Define one `CanonicalObjectIdentity` as object source namespace, owner `RegionCoord`, and authored
  local ID.
- Make `CanonicalObject` carry that identity as its sole address representation.
- Replace the unqualified exact Runtime lookup with one identity-qualified input; retain no overload,
  optional namespace, alias, or fallback.
- Emit the same identity from bounded nearest results while preserving spatial conversion and tie
  ordering.
- Require the strict workbench payload to carry exactly 64 lowercase hexadecimal namespace digits
  and reject the previous payload schema.
- Extend independent exact/nearest `.wlr` oracles to hash the format-authoritative header plus index.

Prototype target retention, selection/disappearance policy, gameplay-persistent IDs, interaction,
highlighting, facing/LOS, 3D distance, navigation/collision, networking, multiple actors, and Wulin
semantics are out of scope.

## Workload

1. Prove exact lookup output is source-qualified and independent of physical triple order within one
   snapshot.
2. Reject namespace mismatch before any region/page lookup, plus invalid local ID, outside window,
   malformed page, pre-publication, and missing namespace payload.
3. Independently hash header plus index and compare exact/nearest identities for every focused query.
4. Preserve exact terrain-position conversion, nearest radius/tie behavior, zero query-side work,
   and first/replay GPU hashes.
5. Publish source A, replace it with physical-order source B, reject the stale A identity, compare
   unchanged raw/spatial content under a different namespace, revisit A, and reject stale B.
6. Preserve adjacent same-source identity, both failed-pair retentions, restart, two traversal modes,
   bounded resource checkpoint, and lifecycle cleanup.
7. Preserve the prototype's native F+W one-shot observation under the new identity result.

## Controlled Variables

- `.wlr` schema 3, source namespace derivation, stable-seed namespace, source-addressed cache, CPU/GPU
  page lifetime, publication, renderer, nearest scan, and prototype policy remain unchanged.
- Source namespace is copied from the immutable published snapshot and costs no query allocation.
- Nearest tie order remains distance squared, owner region X/Z, and authored local ID; every candidate
  in one scan necessarily has the same source namespace.
- Source namespace changes with the exact pack header/index, including physical payload ordering. It
  qualifies a source snapshot and does not promise identity across recooks or content revisions.
- Stable-seed namespace cannot substitute for source qualification because it intentionally remains
  equal across A/B sources.

## Metrics

- Exact source namespace, owner region, authored ID, raw object, terrain position, nearest deltas,
  squared distance, and candidate count.
- Source mismatch, unqualified payload, invalid address, malformed snapshot, and pre-publication
  rejection shapes.
- A/B namespace difference, revisit restoration, failed-pair/restart equality, traversal samples,
  resource samples, lifecycle cycles, workflow time, artifact bytes, and GPU hashes.

## Acceptance Criteria

- No exact object result exists without one complete source-qualified identity.
- Exact lookup rejects a non-current namespace and cannot return content at the same region/local ID
  from another source.
- A/B sources have different identities while their order-independent raw and spatial query content
  remains equal; revisiting A restores its exact namespace.
- Failed object or terrain publication retains the prior qualified identity atomically.
- Independent source-byte oracles reproduce every namespace and object field.
- Query work remains zero-allocation/source-I/O/GPU-copy/readback/fence/synchronization.
- No unqualified API/protocol, compatibility path, retained product target, persistent gameplay ID,
  new resource, format/asset change, networking, or Wulin behavior is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 A/B
packs centered at `(2^40, -2^40)`, and maintained focused/full acceptance workflows.

## Evidence

All 95 engine-runtime tests pass. The private query suite proves identity output, source mismatch,
physical order, malformed state, bounded nearest, signed edges, and exact radius/tie behavior. Public
position tests preserve closed-edge normalization under the nested identity. Clippy passes with
warnings denied and the selected Deno modules type-check.

`canonical-frame-v5` passes in 13.731 seconds. The independent header-plus-index SHA-256 oracle
matches source namespace `84e88c…99d9`; exact and nearest queries return that identity, source
mismatch and the old payload fail strictly, and all query work counters remain zero. First/replay
color hash remains `8b13d214…4135`; object-ID hash remains `01951615…da5b`.

`canonical-prototype-v18` passes in 75.899 seconds. Native F+W still observes the exact committed
actor output and its full source-qualified nearest result equals the independent oracle. Existing
prototype startup, input, movement, presentation, camera, jump, traversal, boundary, Escape,
restart, and cleanup gates remain exact.

The final-worktree `canonical-runtime-v6` passes in 238.700 seconds. Source A `e7f104…2b56` and source B
`1cd5cc…e3a` retain equal raw/spatial content but distinct qualified identities; each stale identity
fails after the opposite publication, and A revisit restores its exact namespace. Both failed pair
types, restart, 32+32 traversal samples, six-publication warm plus eight measured publications, and
two lifecycle cycles pass. Resources hold 492 handles and 21 threads; final private bytes are 299,008
above baseline under the unchanged 16 MiB allowance. The report retains 24 files and 25,346,259
bytes.

Repository guard passes with zero Flavor deny issues. Repeated A/B identity orchestration resides in
the object integration owner, leaving the full wrapper at 488 lines without an exemption.

## Conclusion

Accepted. Canonical object addresses are now qualified by their exact committed source and cannot
silently alias after replacement. This is a snapshot/source identity, not a persistent gameplay ID
or retained selection.

## Promotion

Promoted `CanonicalObjectIdentity`, public `ObjectSourceNamespace`, strict identity lookup, nested
exact/nearest results, independent namespace oracle, stale-source gates, and one removal guard.
Promoted no unqualified compatibility path, registry, allocation, target state, gameplay identity,
interaction, new resource, format change, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype -p workbench
runseal :canonical-frame
runseal :canonical-prototype
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/canonical-frame/`,
`out/captures/canonical-prototype/`, and `out/captures/canonical-runtime/`.
