# Experiment 0092: Committed Camera-Relative Locomotion

Status: Accepted

## Hypothesis

The prototype can use its pure current-sample quarter-orbit camera candidate to rotate exact local
W/A/S/D Walk/Run commands into world XZ before playable-boundary admission, then preserve the
existing actor and camera commit rules without adding a second movement path or retained transform.

## Scope

Keep local input reduction and gait scale unchanged: Walk uses 32/23 Q9 and Run uses 64/45 Q9.
Apply one exact integer quarter rotation selected by the same camera candidate later submitted to
the existing runtime camera mutation:

- orbit 0: `(x, z)`;
- orbit 1: `(z, -x)`;
- orbit 2: `(-x, -z)`;
- orbit 3: `(-z, x)`.

Prepare the camera candidate before locomotion authoring. Use the rotated world command for existing
boundary admission, presentation/facing selection, and the sole actor transaction. Continue to
commit the camera policy index only after `Runtime::set_actor_relative_camera` accepts that same
candidate.

## Workload

1. Prove all four rotations for local forward Walk, local right Walk, and forward-right diagonal
   Run, including exact magnitude and `running` preservation.
2. Preserve all stationary, opposing, focus-loss, gait presentation, facing, and playable-boundary
   policy tests.
3. Extend ordered native input acceptance with one visible E/VK 69 then W/VK 87 batch.
4. Require its first nonzero actor transaction to use orbit-1 world displacement `(-32, 0)`, Walk
   clip 1, and yaw 32,768 rather than the old world-forward `(0, -32)` command.
5. Require the same readiness frame to publish committed camera orbit 1, exact actor-relative rig,
   following camera, rotated traversal desire, grounded stability, and zero render block.
6. Preserve exact default/restart/world-orbit-0 Walk/Run, stationary E, Jump, Escape, boundary,
   failure, and Sidecar lifecycle evidence.
7. Run focused Rust/Deno checks, `runseal :canonical-prototype`, `runseal :init`, and
   `runseal :guard`.

## Controlled Variables

- Camera rig values, edge-driven candidate/commit state, runtime camera validation, camera-before-
  frame ordering, and camera-driven traversal remain unchanged.
- Gait scale, diagonal normalization, boundary reduction, actor schedule/transaction, gravity,
  terrain contact/query, presentation epoch, render admission, and frame behavior remain unchanged.
- The rotation is a sign/axis permutation over private integers. It creates no matrix, float global
  coordinate, retained heading transform, or engine camera/locomotion controller.
- Fractional or render-blocked actor work commits no motion/presentation; the existing independent
  valid camera mutation may still commit its action for subsequent samples.
- A camera mutation failure after actor work remains terminal under the pre-existing application
  ordering. No next sample can observe a silently split camera authority.

## Metrics

- Exact Walk/Run commands for every orbit and their final presentation headings.
- Ordered native key/process/window evidence.
- Sample elapsed time, step/query counts, input/output position, vertical state, actor identity,
  presentation/epoch, grounded witness, camera rig/anchor, traversal state, and block count.
- Existing process equality, focused test counts, workflow duration, Flavor result, and ownership
  diff.

## Acceptance Criteria

- Each orbit performs exactly the declared integer rotation without changing Walk/Run magnitude or
  adding state. Orbit 0 remains byte-for-byte equivalent to the accepted world-axis behavior.
- Same-sample E+W authors orbit-1 world X `-32`, Z `0`, Walk clip 1, and yaw 32,768, and commits that
  motion only through the existing actor transaction.
- The matching camera candidate commits orbit 1 only after runtime acceptance and anchors the moved
  actor before its readiness frame; traversal observes the same rotated camera.
- Existing W, Shift+W, E, Space, Escape, boundary, restart, and failure witnesses remain exact.
- No reference-host or engine/runtime code, cross-subsystem transaction, compatibility route,
  arbitrary-angle steering, renderer/GPU/resource/synchronization/source/format/asset, gameplay,
  networking, or Wulin boundary is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference host, four exact prototype
camera rigs, normalized held/edge input, finite sandbox, and existing actor/camera runtime methods.

## Evidence

Focused checks pass 83 engine-runtime, 26 prototype, and 20 reference-host tests. Eight locomotion
tests include one table proving every orbit for forward Walk, right Walk, and diagonal Run while all
accepted gait, opposing-input, focus-loss, presentation, and committed-facing cases remain exact.
Rust and Deno formatting/type checks pass. Sidecar status/PID reads moved from the growing process
orchestrator into the existing `prototype/process.ts` owner; `host.ts` remains 491 lines without
raising or excluding the Flavor limit.

`canonical-prototype-v16` passed in 72,161.266 ms. Its dedicated visible process reported ordered
E/VK 69 then W/VK 87 key-down messages through `prototype-native-input-v4`. One 43,243,300 ns Ready
sample emitted two fixed steps with command X/Z `-32/0`; actor local X moved exactly `0 -> -64` while
Z stayed 0, with two terrain queries. The committed transition changed Survey clip 0/yaw 0 to Walk
clip 1/yaw 32,768 and epoch `1 -> 3`, retained generation 1, vertical velocity 0, and final grounded
true.

The same readiness record committed orbit 1 with rig `[12,4,-9] / [-3,-1,0]`, anchored the moved
actor at camera `[11.875,6.166015625,-9] / [-3.125,1.166015625,0]`, desired exact traversal center
`[baseX+1,baseZ-1]`, scheduled once, and reported no block, failure, prefetch, queue, or rollover.
Default, restart, native-W, Shift+W, stationary E, Space, Escape, 15-second boundary, failure, and
Sidecar lifecycle processes retained their accepted invariants.

No engine, reference-host, renderer/GPU, resource, synchronization, source, lifecycle, format,
asset, or Wulin code changed, so `canonical-actor` and the long canonical runtime workflow were not
required. Init and final guard passed with zero Flavor denies and five existing warnings.

## Conclusion

Accepted. The prototype now maps local Walk/Run intent through the exact current camera candidate
and commits the resulting world motion/facing through the existing actor transaction. Camera state
still commits only after the existing runtime mutation; no second movement or camera authority was
created.

## Promotion

Promoted one pure four-state integer locomotion rotation, candidate-before-command application
ordering, ordered visible E+W evidence, and exact camera-relative process assertions. Promoted no
host/engine state, runtime API, compatibility path, arbitrary steering, new asset, movement dynamics,
gameplay effect, networking, or Wulin behavior.
