# Experiment 0022: Signed Camera Traversal

Status: Accepted

- Related ADRs: [ADR 0025](../../docs/adr/0025-signed-camera-traversal.md)

## Hypothesis

The accepted camera half-open mapping and one-in-flight plus latest-wins policy can
drive signed atomic composition at a frozen global origin: camera movement selects a
bounded local center, checked integer delta maps it to an exact signed global center,
and every scheduled, queued, blocked, and published target retains one matched
global/local identity without changing GPU work or growing request state.

## Scope

This experiment extends the existing `composition.traversal.enable` behavior according
to the currently published pair. A local pair starts the unchanged legacy session. A
global pair freezes its signed origin and validates that every legal local center for
the session can be added to that origin without signed overflow.

Camera XZ continues to map to the bounded local 128 by 128 alias with the accepted
16-meter half-open convention and edge clamp. The signed global center is then
`frozen_origin + (local_center - 64)` on each axis. One internal traversal target carries
both local and optional global configurations through desired, queued, blocked,
scheduled, and published state.

This experiment does not move the camera into the global-space owner, automatically
change the frozen origin, rebase render coordinates, prefetch, add loading
presentation, change formats, or migrate GPU and semantic identities. It proves signed
camera-driven cache identity within one explicit alias window, not an unbounded map.

## Workload

1. Reproduce Experiments 0018 and 0021 unchanged.
2. Publish a signed composition pair at `(2^40,-2^40)`, enable traversal, and validate
   the frozen global basis plus exact current global/local target.
3. Sample immediately before, exactly on, and immediately after positive and negative
   camera region boundaries. Require the accepted local ownership and exact signed
   global centers.
4. Walk an eight-region corridor. Require each automatic pair to retain 20 and upload
   five entries in both caches while complete old pair evidence remains visible until
   matched publication.
5. Hold terrain I/O while crossing one boundary, then move across several more
   centers. Require one pending global pair, one replaceable global target, maximum
   queue depth one, and only the held plus final latest-wins publications.
6. Request a camera center whose local terrain aliases are absent. Require one blocked
   signed target, no retry while idle, unchanged complete pair/attachments, and recovery
   only after a different valid camera target.
7. Disable traversal, move the camera, and require no work. Re-enable and require one
   exact global catch-up pair. Restart and reproduce the frozen basis and far mapping.
8. Reject traversal enable when the published origin cannot represent the session's
   legal local center range. Run 32 release boundary crossings and report automatic
   pair, per-half, operator, and composed GPU distributions.

## Controlled Variables

- Signed coordinates remain `i64`; local alias origin is `(64,64)`, active radius is
  two, and region side is 16 meters.
- Camera mapping, clamping, one pending pair, one latest desired target, blocked failure,
  50-slot caches, and 25 active entries retain the accepted Experiment 0018 contracts.
- The global origin is immutable for one traversal session. Changing it requires
  traversal disable and an explicit global pair publication.
- Terrain/object payloads, format V1, local GPU IDs, descriptors, stable keys, semantic
  IDs, shaders, grounding, LOD, and fixed submission remain unchanged.
- Correctness uses the debug namespace. Release timings use validation-disabled
  benchmark mode and make no speedup claim.

## Metrics

- Frozen global origin, local basis, desired/queued/blocked/scheduled/published local
  and global targets, pair tokens, transaction IDs, and mapping hashes.
- Desired changes, automatic attempts/schedules/publications, coalesced replacements,
  maximum queue depth, and publication sequence.
- Per-half retained/uploaded/evicted/resident counts and bytes, held complete-pair
  probes, grounding/LOD/contact evidence, and attachments.
- Pair publication, terrain I/O/copy, object generation/schedule/pending, composed GPU,
  operator observation, process, validation, device-removal, and residual lifecycle.

## Pass Criteria

- Far-origin boundary samples map to exact signed centers with no `f32` global
  conversion. Local mapping remains byte-compatible with Experiment 0018.
- Every automatic global schedule carries one matching local/global target into both
  cache reports and the published pair. Adjacent movement retains 20 and uploads five
  in both caches; resident revisit uploads zero.
- At most one pair and one queued target exist. A held crossing plus rapid movement
  publishes only the held target and final latest-wins target, with old complete pair
  evidence visible throughout the hold.
- Missing content blocks one exact signed target without idle retry or state mutation.
  A different valid target clears the block and publishes once.
- Traversal disable produces no work; re-enable catches up exactly once. Restart
  reproduces the signed basis, mapping, local output, and attachments.
- An origin whose legal session extent can overflow `i64` is rejected at enable without
  changing traversal state or terminating the renderer.
- Experiments 0018 and 0021, debug/release validation, Flavor, and Sidecar lifecycle
  pass without format, GPU, semantic, resource-bound, or submission changes.

## Evidence

The canonical workflow is:

```powershell
runseal :global-traversal
```

Generated evidence remains ignored under
`out/captures/0022-signed-camera-traversal/`.

## Results

The canonical workflow passed in 488.5 seconds, including the complete Experiment 0021
compatibility chain. The deterministic terrain pack contained 250 regions and hashed
to `0e842a766d48b2eb76e94b6e25ad7b030c9e73ab89213124556c62784490cb18`.

- Nine half-open boundary samples and both clamp extrema produced the exact local
  centers and signed centers around `(2^40,-2^40)`. The baseline global mapping hash was
  `c094fd79322493121fcbe881565e44f1504c46390ff665a96620a023334e3b1c`.
- Eight correctness corridor crossings retained 20 and uploaded five entries in each
  cache. The release sweep repeated this for 32 crossings with fixed 20,480-byte
  terrain and 102,400-byte object uploads.
- Held I/O plus desired centers 66, 67, and 68 produced two attempts, schedules, and
  publications: the held center and final center 68. Two queued replacements occurred,
  and maximum queued depth remained one.
- Missing terrain at local center `(68,96)` produced one attempt, zero schedules, zero
  publications, no idle retry, and byte-stable old-pair attachments. Moving to `(69,64)`
  cleared the block and published once.
- Disable suppressed movement work, re-enable performed one catch-up, the full legal
  extent overflow was rejected with `stream_failed`, and restart reproduced the global
  mapping plus all four attachment hashes.
- Debug correctness and validation-disabled release sessions reported no device loss.
  Experiment 0018 and Experiment 0021 compatibility workflows passed unchanged, and
  both Sidecar namespaces were stopped after collection.

For 32 release adjacent crossings, median/P95/P99 times in milliseconds were:

| Metric | Median | P95 | P99 |
| --- | ---: | ---: | ---: |
| Terrain schedule | 0.2841 | 0.4025 | 1.0198 |
| Terrain copy GPU | 0.002624 | 0.003136 | 0.003456 |
| Terrain I/O total | 0.0617 | 0.0839 | 0.1094 |
| Object generation | 0.0221 | 0.0709 | 0.0722 |
| Object schedule | 0.1596 | 0.2439 | 0.5541 |
| Pair publication | 0.3138 | 0.4440 | 1.0125 |
| Composed GPU | 0.117760 | 0.143360 | 0.147456 |
| Operator observation | 90.8056 | 101.9165 | 102.3571 |

## Conclusion

The hypothesis passes. The accepted bounded camera mapping and latest-wins scheduler
can carry one exact signed global/local target through both caches and atomic
publication while preserving all local, resource, GPU, and semantic contracts.

The result is deliberately window-scoped. It proves camera-driven signed cache
identity around one immutable origin; it does not establish automatic rebase,
predictive streaming, authored partitioning, or an unbounded world.

## Promotion

Promote the optional signed target into the established camera traversal state and
retain `runseal :global-traversal` as the regression workflow. ADR 0025 makes the
frozen-origin and checked-extrema rules binding.
