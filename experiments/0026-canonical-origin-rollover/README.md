# Experiment 0026: Canonical Origin Rollover

Status: Accepted

- Related ADRs: [ADR 0029](../../docs/adr/0029-canonical-origin-rollover.md)

## Hypothesis

The accepted canonical terrain/object pair can change its bounded local origin without
changing signed identity or exposing a mixed coordinate frame. A canonical-only
traversal policy that recenters an out-of-band axis at local region 64, combined with
an equal camera translation applied only at pair commit, is sufficient to make origin
rollover atomic, deterministic, bounded, and visually continuous.

## Scope

Canonical traversal keeps each local active center axis inside the inclusive safe band
`[32,96]`. Camera mapping first derives the exact signed desired center from the current
published origin. If an axis leaves the band, that axis's new origin becomes the exact
desired signed center and its local center becomes 64. Axes still inside the band keep
their origin and local center.

The camera position and target translate by the opposite integer-region origin delta
only when the matched terrain/object pair commits. The published pair, traversal basis,
and camera therefore enter the new local frame together. Pending, held, rejected, and
failed targets continue to expose the complete old pair and old camera frame.

Format-V1 local and signed traversal retain their frozen-origin policy, status shape,
boundary mapping, and attachments exactly. Canonical manual schedules remain available.
Prefetch, authored/cooked objects, persistent public IDs, collision, navigation,
networking, and a general floating-origin framework remain out of scope.

## Workload

1. Reproduce Experiment 0025 unchanged, including its complete V1/V2 compatibility
   chain, and reproduce Experiment 0022 frozen-origin traversal evidence exactly.
2. Publish one fixed signed V2 terrain/object window around `(2^40,-2^40)` through local
   alias 97, place the camera in that alias, then enable canonical traversal. Require an
   automatic same-window normalization to alias 64 with 25 retained, zero I/O/upload,
   exact stable content, oracles, semantic joins, and byte-identical attachments.
3. Move one region at a time through the positive and negative safe-band boundaries.
   Require a single-axis movement to retain at least 20/upload at most five and the
   boundary publication to atomically change origin, reset only the crossed axis to 64,
   and apply one exact camera translation without changing the global desired center.
4. Repeat X, Z, diagonal, positive, and negative rollovers near `i64`-safe far anchors.
   Validate checked arithmetic, bounded aliases, exact signed joins, stable seeds,
   projection, grounding, contact, animation, LOD, geometry, and CPU/GPU aggregates.
5. Hold terrain I/O, terrain copy, and object copy across rollover. Move the camera to
   several newer targets while held. Require one in-flight pair, one latest target, the
   complete old camera/frame, and only the held plus final publication after release.
6. Request missing/corrupt terrain beyond a rollover boundary. Require no basis/camera/
   pair mutation, one blocked target without retry churn, recovery on a different valid
   target, and reuse of valid immutable object cache work.
7. Disable traversal across a boundary and require no rollover. Re-enable and require
   one catch-up publication. Restart and reproduce the base normalization exactly.
8. Run release sweeps over 32 ordinary adjacent moves and 32 rollovers/normalizations.
   Report both halves, rebase deltas/counters, pair publication, GPU composition,
   capture, operator observation, validation, process, and device status.

## Controlled Variables

- Terrain/object source identity, signed keys, payload formats, 50-slot caches, 25-entry
  snapshots, active radius, and atomic pair publication remain unchanged.
- Centered projection, camera translation convention, root constants, descriptors,
  render targets, fixed dispatch/indirect submission, reverse-Z, LOD, grounding, and
  animation settings remain unchanged.
- Rollover is derived only from a committed canonical pair and the fixed safe band.
  Callers cannot toggle a GPU projection mode or supply a rebase delta.
- Origin and desired-center arithmetic remains checked `i64`. GPU and camera deltas are
  bounded integer-region differences converted exactly to `f32` meters.
- Correctness uses the debug workbench. Release uses validation-disabled benchmark mode
  and makes no speedup claim.

## Metrics

- Old/new signed origins and centers, local centers, per-axis region/meter deltas,
  rollover count, cumulative delta, desired/queued/pending/published targets, and
  latest-wins counters.
- Terrain/object source, cache slots, retained/uploaded/evicted/resident counts, bytes,
  object content/stable-seed hashes, projection and exact semantic inverse entries.
- Camera before/held/committed positions and targets, view hash, grounding/contact,
  skeletal/terrain CPU/GPU aggregates, color/PNG/object-ID/diagnostic hashes, samples,
  and unknown/collision/mismatch counts.
- I/O, generation, copy, pending, pair publication, combined GPU, capture, operator,
  validation, process, device-removal, and lifecycle distributions.

## Pass Criteria

- Experiment 0025 and Experiment 0022 pass unchanged. Noncanonical traversal keeps its
  frozen-origin behavior and omits all rollover status fields.
- Same-window normalization retains all 25 terrain/object entries with zero transfer and
  exact frame evidence. It performs one basis change and one equal camera translation.
- Every rollover preserves exact signed desired center, resets only crossed axes to 64,
  keeps local centers/radius legal, and changes basis/camera only with atomic pair commit.
- Ordinary adjacent and single-axis boundary movement retain at least 20/upload at most
  five in both caches; a two-axis diagonal retains at least 16/uploads at most nine.
  Extra resident hits are accepted only when both halves agree and transfer bytes match
  the reduced upload count. No source, stable-seed, semantic, grounding, animation,
  geometry, attachment, or CPU/GPU oracle mismatch occurs.
- Three holds and latest-wins expose no mixed old/new origin, camera, cache snapshot, or
  attachment. Failure, disable/catch-up, and restart are deterministic and bounded.
- Root constants, cache/request/copy capacity, GPU submission, debug/release validation,
  Flavor, Sidecar lifecycle, and device status pass.

## Evidence

The complete recursive workflow passed on 2026-07-14 in 818.5 seconds. It reproduced
Experiment 0025 and its compatibility chain, then passed debug correctness, restart,
and validation-disabled release sweeps on the reference D3D12 adapter. Debug validation
was enabled, release validation was disabled as required, and neither process reported
device removal.

Same-window normalization moved local alias 97 to 64 with an exact `(-33,0)` region
camera delta. Both caches retained 25 regions, uploaded zero, transferred zero bytes,
and preserved color, PNG, object-ID, diagnostic, stable-seed, semantic-join, and oracle
evidence. Positive and negative X/Z boundaries retained/uploaded `20/5` in both halves;
positive and negative diagonals retained/uploaded `16/9`.

Terrain I/O, terrain copy, and object copy holds retained the complete old pair, basis,
camera, and attachments until the latest target committed. Missing and corrupt terrain
left the old coordinate frame intact without retry churn; retry reused valid immutable
object work. Disable/catch-up and restart reproduced exactly, while V1 status continued
to omit rollover state.

The 32 ordinary release samples reported composition GPU median/P95/P99 of
`0.121856/0.143360/0.144384 ms` and pair-publication
`0.3640/0.4858/0.8151 ms`. The 32 rollover samples reported
`0.106496/0.118784/0.119808 ms` and `0.6103/0.6911/0.8410 ms`, respectively. These are
reference observations rather than a speedup claim; all fixed submission, capacity,
validation, semantic, and CPU/GPU oracle gates passed.

Canonical reproduction:

```powershell
runseal :canonical-origin-rollover
```

The ignored structured report is
`out/captures/0026-canonical-origin-rollover/acceptance.json`.
