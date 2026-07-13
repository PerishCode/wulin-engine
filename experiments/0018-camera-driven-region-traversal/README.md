# Experiment 0018: Camera-Driven Region Traversal

Status: Accepted

- Related ADRs: ADR 0021

## Hypothesis

A camera-derived CPU scalar policy can drive the accepted atomic terrain/object pair
without operator-sequenced scheduling: at most one pair is in flight, at most one
latest desired center is retained, every frame uses one complete published pair, and
continuous crossings or rapid teleports cannot expose mixed terrain/object snapshots
or create unbounded request work.

## Scope

This experiment adds the smallest camera-to-region control loop around the accepted
composition coordinator. The published pair fixes `worldRegionSide` and
`activeRadius`. Camera XZ selects an owning 16 meter region using half-open intervals;
the selected center is clamped so the active window remains inside that world.

The coordinator may have one in-flight pair and one replaceable desired center. While
a pair is pending, camera changes replace that single desired value. The complete old
pair remains renderable until both new halves stage and publish atomically. A failed
desired center is blocked and reported rather than retried every frame.

This experiment does not accept predictive prefetch, hysteresis, velocity estimation,
multiple in-flight pairs, cancellation after I/O begins, a general streaming graph,
floating origins, world partition authoring, collision, navigation, or server-driven
interest management.

## Workload

1. Reproduce the accepted arbitrary-Q8, automatic-terrain-LOD composition at center
   `(64,64)` and enable traversal from that published pair.
2. Validate camera-to-center ownership immediately before, exactly on, and immediately
   after positive and negative region boundaries. Validate clamping at every logical
   world edge.
3. Walk a deterministic corridor one region at a time. For every crossing, observe the
   old published token while the replacement is pending, then require one atomic
   publication matching the latest camera center.
4. Arm the terrain I/O gate, cross into one region to start a pair, and move through
   several additional centers while that pair is held. Require one in-flight pair, one
   replaceable queued target, bounded counters, and no intermediate schedule.
5. Release the gate. Require the held pair to publish complete, followed by exactly one
   pair for the latest desired center. Intermediate centers must never publish.
6. Teleport to a distant cooked center and back. Require the same old-pair continuity,
   atomic publication, exact grounding, terrain LOD/contact gate, and deterministic
   logical revisit evidence.
7. Disable traversal and move the camera. Require no scheduling or publication. Re-
   enable it and require one catch-up pair for the current camera center.
8. Restart the process and reproduce the baseline mapping, queue bound, traversal
   counters, grounding, LOD/contact, and attachment evidence.

## Controlled Variables

- Terrain/region format V1, 128-region canonical address space, 16 meter region side,
  5 by 5 active set, 25,600 arbitrary-Q8 objects, cache capacities, animation settings,
  semantic ranges, terrain LOD 2/6 policy, and 1280x720 attachments remain unchanged.
- Region ownership is `floor((worldPosition + 1032) / 16)` on X and Z, followed by
  clamping to the published world's legal center range. Exact boundaries belong to the
  positive region.
- The initial published pair owns immutable `worldRegionSide` and `activeRadius` for a
  traversal session. Changing either requires traversal disable and explicit pair
  publication.
- Camera observation and scheduling are main-thread CPU work. Terrain, grounding,
  animation, visibility, LOD, and rendering retain their accepted GPU paths and fixed
  submission counts.
- Correctness uses the debug Sidecar namespace. Release evidence uses the benchmark
  namespace with validation disabled. Raw reports remain ignored under `out/`.

## Metrics

- Camera position, mapped desired config, published config/token, pending config/token,
  queued config, and blocked config.
- Distinct desired changes, automatic attempts, accepted schedules, coalesced
  replacements, publications, maximum queued depth, and schedule/publication sequence.
- Old-pair probes while pending, terrain/instance config agreement, physical-slot
  divergence, exact ground and position hashes, LOD/contact evidence, and attachments.
- Pair publication duration, traversal catch-up duration, GPU stage distributions,
  validation state, device-removal state, and residual Sidecar processes.

## Pass Criteria

- Boundary mapping follows the registered half-open convention exactly and clamps every
  desired center to a legal active window without changing world side or radius.
- The coordinator reports at most one pending pair and maximum queued depth one. Rapid
  movement may replace the queued value but cannot append work, allocate by request
  count, or submit an intermediate center after a newer desired center exists.
- Every requested probe during a held transaction uses the complete old published
  terrain/object pair. Terrain and instance configs, logical region order, pair token,
  and fixture agree; no frame or probe exposes one new half with one old half.
- Releasing the held pair produces exactly two publications: the already in-flight
  center and the final latest-wins center. Skipped intermediate centers never publish.
- Continuous walking, teleport, revisit, traversal disable/enable, and restart preserve
  exact arbitrary-Q8 grounding, zero boundary mismatch, 25/25 terrain/instance physical
  slot divergence, automatic LOD validity, and the 0.125 meter fixture contact gate.
- Traversal-disabled camera movement produces no schedule. Re-enable catches up once.
  A failed desired config is reported and not retried until the desired center changes
  or traversal is explicitly re-enabled.
- GPU dispatch counts and resource bounds remain those accepted by Experiment 0017;
  camera traversal adds no GPU pass, per-center command recording, or request-sized
  allocation.
- Affected Experiment 0015-0017 workflows and standalone terrain LOD pass with no
  validation error, device loss, hidden fallback, unbounded growth, or residual Sidecar
  process.

## Evidence

The planned canonical workflow is:

```powershell
runseal :region-traversal
```

Generated evidence remains ignored under
`out/captures/0018-camera-driven-region-traversal/`.

## Results

The canonical corridor pack contains 135 deterministic regions and validates 223
neighbor edges with 7,359 exact sample comparisons and zero mismatch. The default
cooker invocation remains byte-compatible with Experiment 0017.

Nine controlled boundary samples reproduce the half-open convention on both axes and
clamp extreme camera positions to legal centers `(2,2)` and `(125,125)`. An eight-step
walk through centers 64-68 publishes exactly one complete pair per settled crossing.

With terrain I/O held at center 65, subsequent camera centers 66, 67, and 68 retain only
center 68. The traversal records two queued replacements, maximum queue depth one, and
schedule/publication counters both move from 18 to 20 after release: one complete held
pair followed by one complete latest-wins pair. A requested probe during the hold still
uses published center 64 for both terrain and objects while center 65 is pending.

The absent center-80 workload records exactly one automatic attempt, zero accepted
schedules, and zero publications. It remains blocked without retry for the observation
window, and the complete center-64 pair stays published. Traversal disable ignores
camera movement; re-enable submits one catch-up pair.

Baseline arbitrary-Q8 ground and position SHA-256 remain
`c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db` and
`509b4ffb49cdbdd29b40d9be2baf3b8c8030508060fcadc43932eb497eb03658`.
Baseline, logical revisit, and restart attachments are byte-identical. Center-96
teleport has a maximum selected-surface contact residual of 0.092285 meter, below the
registered 0.125 meter gate. Every probe retains exact boundary evidence and 25/25
terrain/instance physical-slot divergence.

Across 32 release center-64/65 transitions, Sidecar-observed catch-up median/P95/P99 is
164.527/201.766/236.246 ms. Pair publication median/P95/P99 is
49.648/50.717/51.674 ms, and requested combined GPU work is
1.136/3.334/3.933 ms. These values characterize the laboratory control path; they do
not establish a streaming service budget or frame-time improvement.

## Conclusion

The hypothesis passes. Camera movement can own region selection while the existing
atomic pair coordinator bounds request state and preserves complete old snapshots.
Continuous walking, held-I/O coalescing, teleport, failure, disable/catch-up, revisit,
and restart all retain the accepted grounding, LOD, contact, and fixed GPU submission
contracts.

The experiment accepts state consistency and bounded latest-wins scheduling, not
latency hiding. A teleported camera can temporarily look beyond the old active window,
and boundary oscillation has no hysteresis. Predictive prefetch, overlap policy,
cancellation, floating origins, and general world streaming remain future experiments.

## Promotion

Promote the camera-to-center convention, immutable traversal basis, single desired
slot, blocked-failure behavior, and camera-owned atomic scheduling as the workbench
baseline. Keep the policy workbench-owned until a later experiment establishes a
reusable streaming boundary.
