# Experiment 0100: Retired Standalone Terrain Contact

Status: Accepted

## Hypothesis

The standalone terrain-contact Runtime/inspect chain can be deleted without losing an engine or
Prototype 0 dependency because the sole private pure contact contract is already consumed by motion,
translation, and the canonical probe's bounded transition witness.

This divisible-by-twenty phase can also reclaim ignored compiler/generated data after verification
without removing source or accepted evidence.

## Scope

- Delete `Runtime::resolve_terrain_contact`.
- Delete the `CanonicalTerrainContact` protocol variant, payload decoder, dispatch body, and
  `canonical.terrain.contact` route.
- Delete the standalone contact acceptance support and its pre-publication/direct response gates.
- Replace the older recurring `canonical.terrain.contact.probe` rejection with exactly one current
  `canonical.terrain.contact` unknown-event witness.
- Extend the removal guard to forbid both dense historical and standalone contact surfaces while
  requiring private `resolve_body_contact` and the 225-body canonical probe witness.
- After full verification and the commit hook, delete ignored `target/` and `out/` roots and measure
  their absence.

Changing contact arithmetic/result types, terrain height queries, motion/translation/actor policy,
the generic probe, renderer/GPU work, source formats, assets, networking, and Wulin behavior is out of
scope.

## Workload

1. Audit every public Runtime method and live reference; prove the standalone contact chain has no
   product consumer.
2. Delete the full method/protocol/dispatch/support chain rather than retaining an alias or parser.
3. Send the retired direct verb once and require generic `unknown_event`; stop sending the older
   dense-probe verb.
4. Run pure contact, motion, translation, actor, canonical-probe, protocol, and repository tests.
5. Run full canonical acceptance with existing A/B, rollback, restart, 32+32 traversal, bounded
   resources, and two lifecycle gates.
6. Inventory absolute ignored roots, verify both are repository children, and remove compiler and
   generated outputs only after all code verification/commit work.

## Controlled Variables

- `resolve_body_contact`, `TerrainBodyContact`, classification/correction arithmetic, fixed motion,
  translation, actor transactions, and probe sample set remain unchanged.
- The generic probe remains the sole process-level direct contact witness: 225 bodies split evenly
  across three classifications.
- Exact terrain height remains a public product dependency and is not part of this cleanup.
- Resource cleanup affects only ignored `target/` and `out/`; committed source, documentation,
  redistributable assets, `.task`, credentials, and repository metadata remain untouched.

## Metrics

- Forbidden-symbol/reference count and retired direct/dense event counts.
- Contact witness classifications, correction count, result/identity hashes, and mismatches.
- Full workflow time, A/B/rollback/restart/traversal equality, handles, threads, private bytes,
  lifecycle cycles, and persistent report bytes.
- Compiler/generated file counts and bytes before and existence after cleanup.

## Acceptance Criteria

- No live Runtime method, protocol variant/decoder/route, dispatch, standalone support file, or full
  acceptance response remains for direct terrain contact.
- The direct old verb returns `unknown_event` exactly once; the older dense-probe verb is no longer a
  recurring acceptance event.
- Private pure contact tests and the exact 225-body canonical witness remain unchanged.
- Full runtime correctness, traversal, resource, and lifecycle evidence passes under the existing
  thresholds.
- `target/` and `out/` are absent after final cleanup, reclaiming their measured pre-cleanup bytes.
- No compatibility alias, product change, source/format/asset change, networking, or Wulin behavior
  is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains and Windows D3D12 reference runtime. Cache
inventory and cleanup operate on absolute repository-child paths through native PowerShell.

## Evidence

All repository Rust tests pass, including 95 engine-runtime tests, exact private contact arithmetic,
motion/translation partitions and rollback, actor simulation, and canonical probe validation. Deno
format/type checks, strict Clippy, resource-policy tests, Sidecar doctor/plan, forbidden scans, and
repository guard pass with zero Flavor deny issues.

`canonical-runtime-v8` passes in 247.249 seconds. `canonical.terrain.contact` is rejected once as
`unknown_event`; `canonical.terrain.contact.probe` has no event. The generic witness remains exactly
75 separated, 75 touching, 75 penetrating, and 75 corrected. Result hash remains
`2cd0d711…4306`; identity-keyed hash remains `16446f14…5c9a`.

All A/B object resolution/nearest evidence, two failed publications, restart, 32+32 traversal
samples, five-publication resource convergence, eight measured publications, and two lifecycle
cycles pass. Handles remain 492, threads remain 21, and private bytes move from 425,598,976 to
425,975,808 (+376,832) under the unchanged 16 MiB allowance. The report contains 24 files totaling
25,346,234 bytes.

Before cleanup, `target/` held 35,590 files/10,908,689,993 bytes and `out/` held 5,394
files/9,356,065,266 bytes: 40,984 ignored files and 20,264,755,259 bytes combined. After the final
commit hook, both verified repository-child roots were removed and are absent.

## Conclusion

Accepted. Terrain contact remains one private pure engine contract with focused tests and one
bounded canonical witness; its duplicate public diagnostic chain and accumulated historical witness
are gone. The divisible-by-twenty compiler/generated resource debt is reclaimed.

## Promotion

Promoted the expanded contact-removal guard, sole current unknown-event witness, current runtime
revision, and measured cleanup record. Promoted no replacement contact API, compatibility alias,
product capability, resource owner, format/asset change, networking, or Wulin behavior.

## Reproduction

```powershell
runseal :guard
runseal :canonical-runtime
```

The final cache deletion is intentionally not part of a repeatedly invoked repository wrapper; it
is an operator action at the divisible-by-twenty checkpoint after verification and commit hooks.
