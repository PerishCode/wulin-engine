# Experiment 0038: Camera-Visible Directional Shadows

Status: Accepted

## Hypothesis

The canonical renderer can produce and consume one deterministic directional hard-shadow map for
its camera-visible animated objects by reusing the accepted GPU visible list, camera-selected LOD,
and pose palette, while retaining authored presentation authority, fixed indirect submission,
camera-relative alias stability, bounded resources, and one renderer path.

## Scope

This experiment adds one fixed 1,024 by 1,024 D32 shadow map. A fixed orthographic light projection
covers the bounded canonical render window and uses the same normalized direction as the accepted
surface lighting. Before occlusion compaction, one depth-only mesh pass consumes the existing
camera-visible object list and its existing indirect count. It reads the same catalog geometry,
camera-selected LOD, arbitrary-Q8 ground results, yaw, skin bindings, and GPU pose palette as the
main surface pass.

The surface resolve reconstructs the visible triangle position, performs one nearest-texel depth
comparison with a fixed receiver bias, and removes only the direct-light contribution when the
receiver is shadowed. The shadow pass may not build a CPU draw list, recull objects, select another
LOD, reevaluate animation, copy content, or introduce a second presentation authority.

Terrain casting and receiving, off-camera light-frustum casters, a separate light-space culler,
shadow LOD, cascades, atlases, filtering, soft shadows, alpha testing, dynamic lights, gameplay
light authority, and Wulin content are out of scope. They remain later gates that may depend on
this depth-only object proof.

## Workload

1. Build one immutable light matrix from fixed finite values and prove the canonical 5-by-5 local
   render window is inside its orthographic XY and depth bounds.
2. Allocate one fixed depth target, DSV, SRV, and readback; record one depth-only indirect mesh
   dispatch from the pre-occlusion GPU-visible list after pose evaluation.
3. Reconstruct per-pixel object world position in the existing surface resolve and apply one
   deterministic nearest-depth hard-shadow comparison. Extend the CPU shade oracle with the same
   matrix, texel address, bias, and direct-light rule.
4. Publish depth-map hashes and occupancy, light-matrix identity, caster/dispatch counts, GPU time,
   and per-sample GPU/CPU shadow decisions. Require at least one controlled visible sample to be
   shadowed and at least one to remain lit.
5. Re-run held frames, source aliases and reordering, animation phases, all four I/O/copy holds,
   corruption rollback, traversal/prefetch/rollover, the 64-publication resource plateau, 16
   lifecycle cycles, and the full repository guard.

## Controlled Variables

- Signed schema-3 spatial, identity, and presentation authority; exact `i64` addressing; 50-slot
  residency; atomic terrain-first publication; grounding; LOD; material; occlusion; and the
  source-duration clock remain unchanged.
- The caster set is exactly the accepted pre-occlusion camera-visible object list. Its order,
  count, LOD, pose slot, stable key, material, yaw, and animation fields remain byte-identical.
- The light direction, projection, map size, depth convention, texel mapping, and receiver bias are
  renderer constants. No runtime control or content field is added.
- The root signature must remain within 64 DWORDs. Shadow resources are allocated once with the
  renderer and never grow, stream, or vary with content.
- The main visibility buffer, object-ID output, occlusion history, and terrain renderer retain
  their accepted ownership and formats.

## Metrics

- Light direction, matrix SHA-256, projection bounds, map dimensions/format/bytes, receiver bias,
  depth SHA-256, occupied/clear texels, and occupied depth range.
- Source casters, indirect shadow dispatches, existing visible/LOD/pose counters, root-constant
  DWORDs, descriptors, fixed frame dispatches, and GPU shadow time.
- Per-sample receiver position, light texel, receiver/stored depth, GPU/CPU shadow decision, RGBA8,
  and maximum channel delta.
- Held/alias/source/time hashes, traversal continuity, device state, handle/private-byte plateau,
  and descendant cleanup.

## Acceptance Criteria

- Exactly one 1,024-square D32 map and one fixed indirect shadow dispatch are present. Shadow
  caster count equals the skeletal pre-occlusion visible count, and no CPU draw list, new cull,
  new pose evaluation, content copy, publication, or variable resource is added.
- The shadow map contains both clear and occupied texels, every occupied depth is finite in
  `[0,1]`, and its hash is exact across held frames and camera-relative source aliases.
- Every sampled visible receiver agrees with the CPU oracle on light texel and shadow decision;
  maximum final-color channel delta remains within the accepted tolerance. The controlled frame
  observes at least one lit and one shadowed receiver and changes rendered color evidence from the
  no-shadow baseline.
- Authored time changes produce deterministic shadow-depth and final-color changes while held and
  loop-return frames reproduce exact shadow, surface, color, PNG, and object-ID evidence.
- Root-signature cost stays at or below 64 DWORDs. The 64-publication run has no transient resource
  growth after the new fixed baseline, all 16 lifecycle cycles clean up, no validation error or
  device removal occurs, and the complete repository guard passes.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0038-camera-visible-directional-shadows/`.

## Results

The direct workflow passed in 562 seconds. The controlled frame reused all 10,538 pre-occlusion
camera-visible objects as shadow casters through one depth-only indirect mesh dispatch. The fixed
1,024-square D32 map occupied 4,194,304 bytes, contained 88,557 occupied and 960,019 clear texels,
and bounded occupied depth to `[0.43303064, 0.60740846]`. Its light matrix hash was
`480ef336...41fd65`, and its controlled depth hash was `2415cfdd...1233d5`.

All six controlled surface samples matched the CPU oracle exactly on shadow texel, lit/shadowed
decision, material texel, and final RGBA8. One sample was shadowed and five were lit; maximum color
channel delta was zero. The controlled color/PNG hashes changed from the accepted no-shadow
baseline to `8b13d214...aa4135` / `e96e44cc...4450c6`, while object-ID remained exactly
`01951615...79da5b`.

The imported-duration Walk frames 0/42/43/85 produced shadow-depth hashes
`3cb3faf3...fd2a2`, `51d58c87...f7319`, `3cb3faf3...fd2a2`, and
`3cb3faf3...fd2a2`. Frames 43 and 85 exactly reproduced frame 0 shadow, surface, color, PNG, and
object-ID evidence, while frame 42 differed. Held frames, physical source reordering, source
revisit, movement revisit, and camera-relative aliasing also reproduced the required exact shadow
evidence.

The surface root signature uses 60 constant DWORDs plus one descriptor table, for a 61-DWORD
cost. The surface heap contains 98 descriptors. The controlled shadow pass measured 0.8704 ms on
the reference run. The 64-publication resource baseline was 531 handles and 414,076,928 private
bytes; peak handles remained 531, and the final sample was 516 handles and 413,552,640 bytes. All
32 reactive crossings, 32 prepared crossings, failure/rollback gates, and 16 lifecycle cycles
passed without device removal or descendants.

An initial acceptance harness required the six fixed samples to contain both lit and shadowed
receivers on every animated frame. That exceeded the predefined controlled-frame criterion and
rejected a valid time step despite zero GPU/CPU mismatches. The corrected gate retains exact
per-frame oracle agreement and applies coverage only to the controlled baseline.

## Conclusion

Accepted. Camera-visible animated objects now cast and receive one fixed deterministic
directional hard shadow through the existing visible-list, LOD, grounding, geometry, and pose
authorities. The proof adds one fixed resource set and one fixed indirect depth submission, but no
CPU draw list, light cull, alternate LOD, animation evaluation, content movement, or presentation
authority.

## Promotion

Promoted the fixed light projection, 1,024-square D32 map, depth-only reuse of the pre-occlusion
object stream, nearest receiver comparison, CPU shadow oracle, and stable shadow evidence into the
canonical surface owner. Terrain shadows, off-camera casters, light-space culling, shadow-specific
LOD, cascades, filtering, alpha testing, dynamic lights, and gameplay light authority remain later
experiments.
