# Experiment 0091: Committed Prototype Run Modifier

Status: Accepted

## Hypothesis

The prototype can consume held Shift as a stateless modifier of nonzero W/A/S/D locomotion, select
one fixed two-times cardinal scale with an independently nearest-normalized diagonal plus the
already cooked Fox Run clip, and commit both through the sole actor transaction without adding host
storage, retained gait state, or a second movement path.

## Scope

Keep the accepted Walk cardinal/diagonal components at 32/23 Q9 per fixed step. When Shift is held
and W/A/S/D reduces to nonzero motion, select Run components 64/45; 45 is the nearest integer to
`64 / sqrt(2)`. Shift alone and opposing movement inputs remain stationary.

Carry one private `running` fact only in the prototype's local command. Playable-boundary admission
recomputes it from the final admitted displacement, and presentation selects imported clip 2 only
when that displacement remains nonzero. The runtime continues to receive only its existing motion
and presentation fields.

## Workload

1. Prove exact Walk and Run cardinal/diagonal reduction for all relevant held-key combinations.
2. Prove Shift-only, opposing axes, focus loss, and irrelevant inputs cannot retain Run motion.
3. Prove Survey/Walk/Run selection, all eight exact Q16 facing values, and stationary committed-yaw
   retention through the existing presentation policy.
4. Prove the larger Run displacement against the maximum-eight-step playable-boundary candidate,
   including exact one-Q9 region crossing rejection and clearing of the local Run fact.
5. Generalize the maintained native-input helper to ordered key batches and add one visible-window
   Shift/VK 16 then W/VK 87 process.
6. Require that process to publish `deltaZQ9 = -64`, imported clip 2, exact fixed-step displacement,
   local phase-zero transition, grounded stability, camera/traversal identity, and zero render block.
7. Preserve exact default/restart/Walk/camera/Jump/Escape/boundary/Sidecar lifecycle evidence and run
   focused Rust/Deno checks, `runseal :canonical-prototype`, `runseal :init`, and `runseal :guard`.

## Controlled Variables

- `HostInput` remains the same fixed 96-byte held/pressed/released state; its generic `u8` key
  representation already includes Shift.
- Rational schedule, actor transaction, gravity, step-up, terrain contact/query, playable bounds,
  animation epoch, camera anchoring, traversal, render admission, and frame ordering are unchanged.
- Imported Fox clip 2, source duration, sampled poses, GPU palette, surface, shadow, and occlusion
  paths are already canonical resources; no source cook or runtime binding changes.
- Run is held-state derivation only. There is no toggle, queue, stamina, acceleration, horizontal
  velocity, retained gait state, or configuration surface.
- Air control is unchanged: held locomotion is reduced identically whether grounded or airborne.

## Metrics

- Exact local command components and Run fact for cardinal, diagonal, opposing, stationary, and
  focus-loss cases.
- Exact native process/window/key identities and ordering.
- Fixed-step count, input/output position, actor identity, vertical state, terrain-query count,
  presentation/epoch, grounded witness, camera/traversal state, and render-block count.
- Focused test counts, maintained workflow duration, Flavor result, and implementation diff.

## Acceptance Criteria

- Walk remains exactly 32/23 Q9. Shift plus nonzero W/A/S/D produces exactly 64/45 Q9 and Run clip
  2; Shift alone or fully opposed motion produces zero displacement and Survey clip 0.
- Boundary admission preserves Run for any remaining admitted axis and clears it when all movement
  is reduced. Runtime failures remain strict and no playable-boundary behavior is weakened.
- A visible-window ordered Shift+W process commits one nonzero batch with exact `-64 * step_count`
  Z displacement, Run presentation/facing, zero vertical velocity, exact grounded true witness, one
  query per step, following camera, unchanged traversal contract, and zero normal-path block.
- Existing processes remain byte/number exact under their accepted command and policy invariants.
- No reference-host or engine/runtime code, second movement transaction, compatibility path,
  renderer/GPU/resource/synchronization/source/format/asset, gameplay, networking, or Wulin boundary
  is added.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference host, generic normalized
held-key state, finite prototype sandbox, imported Fox rig bank 1, and the sole actor transaction.

## Evidence

Focused checks pass 83 engine-runtime, 25 prototype, and 20 reference-host tests. Seven locomotion
tests cover exact Walk/Run cardinal and diagonal components, Shift-only and opposed inputs, focus
cleanup, Survey/Walk/Run selection for all eight facings, and committed stationary yaw. Five
boundary tests include the exact maximum Run candidate: local Z 3583 plus eight 64-Q9 steps remains
inside the region, while 3584 crosses by one Q9 and reduces to stationary with `running = false`.
Rust formatting and Deno formatting/type checks pass.

The first final guard correctly denied the expanded process orchestrator at 527 lines and the
five-word Run helper name. The exact stationary/Walk/Run/Jump command fixtures moved into the new
48-line `prototype/simulation.ts` owner, leaving `host.ts` at 487 lines, and the helper name was
reduced without changing its evidence schema or behavior. The next complete guard passed at zero
denies; no Flavor limit or exclusion changed.

`canonical-prototype-v15` passed in 72,650.881 ms. Its dedicated visible process reported ordered
Shift/VK 16 and W/VK 87 key-down messages through `prototype-native-input-v4`. One 47,311,700 ns
Ready sample emitted three fixed steps with command Z `-64`; actor local Z moved exactly `0 -> -192`
with three terrain queries. The same committed transition changed Survey clip 0/yaw 0 to imported
Run clip 2/yaw 49,152, changed animation epoch `1 -> 3` at local phase zero, retained generation 1,
vertical velocity 0, and final grounded true. The actor-relative camera followed the final position,
the existing single traversal desire remained exact, and render-block count was zero.

Default, restart, native-W, native-E, Space, Escape, 15-second boundary, failure, and Sidecar
lifecycle processes retained their accepted invariants. No engine, reference-host, renderer/GPU,
resource, synchronization, source, lifecycle, format, asset, or Wulin code changed, so
`canonical-actor` and the long canonical runtime workflow were not required. Init and final guard
passed with zero Flavor denies and five existing warnings.

## Conclusion

Accepted. Shift now selects one exact Run gait only as part of a final nonzero prototype locomotion
command. Its larger fixed displacement and imported presentation commit atomically through the
existing actor transaction; there is no retained Run state or alternate execution path.

## Promotion

Promoted fixed 64/45 Run components, local admitted-command gait derivation, imported clip-2
selection, ordered native key-batch acceptance, and one exact Shift+W real-process witness. Promoted
no host/engine storage, runtime API, compatibility route, new asset, tuning surface, movement
dynamics, gameplay effect, networking, or Wulin behavior.
