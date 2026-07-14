# Experiment 0033: Deterministic Temporal Presentation

Status: Accepted

## Hypothesis

The canonical runtime can advance authored animation continuously at one catalog tick per
rendered canonical frame while retaining exact deterministic pause/set/step behavior,
stable spatial/identity authority, fixed GPU submission, and zero content data movement.

## Scope

This experiment adds one renderer-owned presentation clock to the accepted schema-3
runtime. Its active tick is modulo the fixed 64-sample animation catalog. It runs by
default after canonical content publishes and advances once after each submitted canonical
frame. A compact `canonical.time.*` control surface exposes status, pause, resume, bounded
set, and paused-only step for deterministic evidence.

The clock only offsets the authored animation phase. It does not select archetype,
material, yaw, animation enablement, clip, phase offset, or variation. Gameplay clocks,
network synchronization, wall-clock interpolation, camera motion, asset import, root
motion, and Wulin content are out of scope.

## Workload

1. Publish one canonical schema-3 window with the clock paused at tick 0 and capture full
   GPU/CPU, attachment, and GPU-read payload evidence.
2. Step to tick 1 without scheduling content. Require animation/surface/capture
   change while presentation payload, stable identity, terrain, grounding, contact,
   publication token, mappings, and copy counters remain exact.
3. Step another 63 ticks and require the tick-0 GPU/CPU, surface, and attachment evidence
   to repeat exactly.
4. Resume automatic time, observe at least eight canonical frame advances, pause again,
   and require a changed frame with no source, residency, or publication activity.
5. Hold an object-copy transaction while time is running. The old complete pair must
   continue animating; publication remains atomic and the clock never waits on a content
   half.
6. Re-run all object/terrain I/O/copy holds, physical reorder, presentation variants,
   corruption rollback, restart,
   traversal/prefetch/rollover, the 64-publication resource plateau, and 16 lifecycle
   cycles with acceptance captures frozen at explicit ticks.

## Controlled Variables

- Schema-3 triple authority, signed `i64` addressing, 50-slot caches, terrain-first
  composition, camera, terrain LOD, visibility, surface resolve, and catalogs remain fixed.
- One tick maps to one of 64 existing animation samples. Automatic time is frame-count
  based, not wall-clock based.
- Clock set accepts only `0..63`. Step is bounded, allowed only while the clock is paused,
  and reduces modulo 64.
- The clock persists across source switches, pair publications, traversal, prefetch, and
  rollover within a process. A process restart begins from tick 0 and running state.
- Existing deterministic evidence freezes and sets the clock explicitly instead of
  depending on asynchronous frame counts.

## Metrics

- Active tick, running state, automatic advance count, manual step count, and wrap count.
- Publication token/count, per-plane upload/copy counts, active mappings, payload hashes,
  stable identities, grounding, and contact before and after time-only changes.
- GPU/CPU skeletal counts, validated palette sample, surface texel/color samples,
  color/PNG/object-ID/diagnostic hashes, and fixed dispatch counts at controlled ticks.
- D3D12 validation/device state, handle/private-byte plateau, and lifecycle descendants.

## Acceptance Criteria

- Paused tick 0, tick 1, and wrapped tick 0 produce exact GPU/CPU oracle results. Tick 1
  changes animation presentation; wrapped tick 0 repeats the initial evidence.
- Time-only changes retain the exact published pair, spatial/identity/presentation payload
  hashes, object IDs, terrain, grounding, contact, active mappings, and all object/terrain
  upload/copy counters.
- Automatic mode advances at least eight frames and changes presentation without a content
  transaction. Pause prevents automatic advancement; invalid set/step requests are rejected
  transactionally.
- A held old pair keeps advancing presentation while the incomplete new pair remains
  invisible. Release publishes one complete new pair without resetting or skipping clock
  ownership.
- Every canonical frame retains four terrain and five skeletal dispatches; no validation
  error, device removal, resource growth, or lifecycle residue occurs.
- Live presentation time has one owner and no wall-clock, stable-key, source-publication,
  or physical-order fallback.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0033-deterministic-temporal-presentation/`.

## Results

The direct workflow passed in 449.1 seconds. Explicit tick 0 and tick 1 produced equal
spatial, identity, presentation-payload, grounding, contact, terrain, and publication
evidence while producing different GPU/CPU surface samples and different color, PNG,
object-ID, and diagnostic attachment hashes. Stepping another 63 ticks returned to tick
0 and reproduced the initial stable frame exactly. Invalid tick 64 and running-state step
requests were rejected without changing the clock.

Automatic mode advanced 11 submitted canonical frames before the workflow paused it at
tick 11. The published pair and all object/terrain transaction reports remained exactly
unchanged. During a held object-copy transaction, the old published token 14 remained
visible while pending token 15 was incomplete; the clock advanced another 11 frames and
the old pair produced changed surface and capture evidence. Releasing the gate published
token 15 without changing the paused tick or clock counters.

Physical reorder, four presentation variants, all four I/O/copy holds, two corruption
rollbacks, restart, prepared rollover, 32 reactive crossings, and 32 prepared crossings
passed at explicit frozen ticks. The warmed resource baseline was 531 handles and
397,922,304 private bytes. No resource sample exceeded 531 handles; the final sample
after 64 publications was 516 handles and 396,902,400 private bytes. All 16 lifecycle
cycles left no process descendant.

## Conclusion

Accepted. Canonical animation time now has one renderer owner, deterministic operator
control, modulo-64 behavior, and a frame-submission boundary. Time changes presentation
without becoming content authority or a source/cache/publication dependency.

## Promotion

Promoted the renderer-owned presentation clock and the narrow `canonical.time.*`
operator/evidence surface. ADR 0036 records the durable boundary. Wall-clock
interpolation, gameplay/network clocks, root motion, general asset import, and Wulin
content remain later gates.
