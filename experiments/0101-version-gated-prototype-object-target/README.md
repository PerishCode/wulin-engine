# Experiment 0101: Version-Gated Prototype Object Target

Status: Accepted

## Hypothesis

One typed stamp derived from the already-authoritative published composition pair can let the
prototype retain a source-qualified object target and revalidate it only after a successful
publication, eliminating resolver work on unchanged frames without copying canonical object content
or inventing persistent gameplay identity.

## Scope

- Expose `CanonicalObjectSnapshot { publication_token, source_namespace }` as a read-only typed
  Runtime value. The token is monotonic only within one live Runtime.
- Promote the prototype's capacity-one F observation result into one identity-only target carrying
  its last validated snapshot and `resolved`/`outside-published-window` availability.
- Associate explicit nearest acquisition with the current snapshot before the query; Runtime
  mutation requires exclusive access, so the consecutive read-only calls observe one snapshot.
- After each successful canonical frame, read the snapshot only when a target exists. Resolve the
  target exactly once if and only if its stamp changed.
- Retain a same-source target as unavailable outside the published window, restore it on a later
  same-source publication, and clear it on source replacement.
- Let a successful empty explicit scan clear the target. Preserve pending intent and the complete
  previous target on scan or validation failure.

Highlighting, interaction/action effects, copied canonical object state, gameplay-persistent IDs,
automatic nearest scans, line of sight, facing, 3D distance, navigation/collision, networking,
multiple actors, formats/assets, and Wulin semantics are out of scope.

## Workload

1. Prove duplicate F admission and zero-step retention, successful candidate acquisition, explicit
   empty-result clearing, and malformed acquisition rollback.
2. Prove equal snapshot stamps produce no resolution request.
3. Advance through same-source window departure and revisit, requiring unavailable then resolved
   state under exactly one request per changed stamp.
4. Replace the source, require typed `source-replaced`, and clear the target so a later old-source
   revisit cannot silently reacquire intent.
5. Reject unchanged-stamp, mismatched resolved identity, and impossible outcome/source combinations
   without partially mutating target state.
6. Preserve target state while Reset/Suspended cancel only pending observation intent.
7. Launch native F+W and verify exact post-commit origin, independent nearest oracle, identity-only
   target, and optional same-frame traversal-publication revalidation.
8. In focused/full workbench workflows, compare every snapshot token/namespace with the published
   pair, prove A/B and adjacent changes, and prove both failed pair types retain the prior stamp.
9. Preserve GPU replay, traversal, resource convergence, restart, and lifecycle evidence.

## Controlled Variables

- The object source, schema-3 pages, cache, atomic pair publication, exact resolver, bounded nearest
  scan, renderer, GPU resources, and host input normalization are unchanged.
- The target stores only `CanonicalObjectIdentity`, `CanonicalObjectSnapshot`, and availability. It
  does not store the nearest `CanonicalObject`, terrain position, presentation, or distance.
- No target means no per-frame snapshot read. An equal target snapshot means no resolver call.
- Source replacement is checked before window membership by the accepted typed resolver.
- A resolver error remains terminal at the product boundary; policy computes and validates the
  complete next target before mutation.

## Metrics

- Policy acquisition/clear/rollback state and validation requests for equal/changed stamps.
- Snapshot publication token and source namespace at resolution, nearest acquisition, A/B
  replacement, adjacent departure, failed pair, and native target evidence.
- Native target identity, availability, copied-state absence, observation/validation token ordering,
  and independent source-oracle equality.
- Focused/full duration, GPU hashes, resource samples, artifact bytes, and lifecycle cycles.

## Acceptance Criteria

- No retained target exists without a complete source-qualified identity and last validated live
  Runtime snapshot.
- Unchanged frames eliminate exact resolver work; each changed publication causes at most one
  validation for the retained target.
- Same-source departure retains unavailable identity and later same-source publication can restore
  it; source replacement clears it.
- Empty explicit scan clears; invalid scan/resolution state preserves the complete previous policy.
- Native F+W retains only the independently verified identity and follows any completed traversal
  publication before readiness.
- Failed object/terrain pair publication does not change the typed snapshot stamp.
- Existing resolver/nearest/GPU/resource/restart/lifecycle evidence remains exact.
- No automatic nearest scan, copied canonical object, engine-owned product target, JSON-parsed live
  policy, gameplay persistence, highlight, interaction, new resource, format/asset change,
  networking, or Wulin behavior is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 A/B
sources centered at `(2^40, -2^40)`, and maintained focused/full acceptance workflows.

## Evidence

All 90 engine-runtime tests and all prototype tests pass. Six object-target policy tests cover
admission, empty clear, rollback, equal-stamp work elimination, window departure/revisit, source
replacement, invalid validation, and host discontinuity. Strict Clippy and selected Deno checks pass.

`canonical-frame-v7` passes in 27.122 seconds. Typed snapshot token/source equals the exact published
pair, pre-publication access fails, resolution/nearest work remains zero, and existing color/object-ID
replay hashes remain `8b13d214…4135` / `01951615…da5b`.

`canonical-prototype-v19` passes in 71.515 seconds. Native F+W observes committed local Z `-32` Q9
and independently selects authored local ID 496 at delta `(160,0)` and squared Q18 distance 25,600.
Acquisition occurs at token 1; the same canonical frame publishes traversal token 2, after which the
identity-only target is revalidated as resolved at token 2. Ordinary/restarted processes retain no
target, and all existing process/lifecycle gates pass.

The final-worktree `canonical-runtime-v9` passes in 253.412 seconds. Source A token 1
`e7f1045b…2b56` changes to source B token 9 `1cd5cc78…e3a`; the same-source adjacent window advances
token 10 to 11 and reports the retired target as `outside-published-window`. Object-corrupt and
terrain-corrupt pair failures both retain exact token 21 and the complete A namespace. Existing 30
successful/three rejected resolutions, 28 successful/three rejected nearest queries, 32+32
traversal observations, rollback, restart, GPU replay, and two lifecycle cycles pass.

Resource convergence settles after four warm publications and measures eight. Handles remain 492,
threads remain 21, and private bytes move from 424,878,080 to 426,065,920 (+1,187,840) under the
unchanged 16 MiB allowance. The report retains 24 files totaling 25,346,346 bytes. Repository guard
passes with zero Flavor deny issues; the five retained warnings are unchanged ownership pressure
outside this experiment. The commit hook passes the same final repository gate.

## Conclusion

Accepted. The prototype retains one identity-only object target and keeps it current at successful
frame boundaries with snapshot-change-gated exact resolution. Unchanged frames eliminate resolver
work; window departure preserves unavailable intent, while source replacement clears it.

## Promotion

Promoted `CanonicalObjectSnapshot`, the typed Runtime stamp getter, prototype-owned capacity-one
target policy, changed-stamp resolution cadence, native token-1-to-token-2 evidence, A/B/window/
rollback snapshot gates, and source guards. Promoted no copied object state, automatic nearest scan,
engine-owned target, gameplay-persistent identity, highlight, interaction, resource, format/asset
change, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p prototype
cargo clippy --locked -p engine-runtime -p prototype -p workbench --all-targets -- -D warnings
runseal :canonical-frame
runseal :canonical-prototype
runseal :canonical-runtime
runseal :guard
```

Generated reports remain ignored under `out/captures/`.
