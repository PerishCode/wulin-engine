# ADR 0095: Committed Camera-Relative Locomotion

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0092 Committed Camera-Relative Locomotion

## Context

Prototype camera orbit already commits one of four exact quarter-turn actor-relative rigs, but
W/A/S/D still authors fixed world-axis motion. After Q/E changes the view, forward input therefore
no longer follows the selected camera orientation. Walk/Run scale, transactional presentation,
boundary admission, and runtime camera/actor methods are already sufficient.

The remaining decision is how the current camera action and locomotion share orientation without
advancing application camera state before runtime acceptance or introducing another controller.

## Decision

- Prepare one pure camera candidate before locomotion reduction and use that same candidate for the
  current command and later checked runtime camera mutation.
- Rotate local integer displacement by orbit 0 `(x,z)`, orbit 1 `(z,-x)`, orbit 2 `(-x,-z)`, or
  orbit 3 `(-z,x)` before playable-boundary admission.
- Preserve Walk 32/23 and Run 64/45 scale, final admitted Run derivation, exact world-motion yaw,
  Survey/Walk/Run policy, and actor transaction semantics.
- Commit camera policy state only after the existing runtime mutation succeeds. Actor fractional or
  blocked work may still accompany a committed camera action but cannot publish motion/presentation.
- Keep the existing terminal ordering if camera mutation fails after actor work; do not invent a
  cross-subsystem rollback transaction for a process that cannot continue.
- Prove same-sample visible E+W as orbit 1 plus world-X Walk, matching camera rig/traversal, and zero
  normal-path block.
- Add no arbitrary-angle input, pointer/gamepad transport, smoothing, acceleration, velocity, root
  motion, blending, runtime controller/API, compatibility path, gameplay, networking, or Wulin
  policy.

## Consequences

- W/A/S/D and held-Shift Run now remain spatially coherent with every committed quarter-orbit view.
- Same-sample Q/E plus motion uses one candidate orientation rather than mixing old movement with a
  new rendered camera.
- Facing remains derived from final world displacement and follows existing committed actor output.
- Camera and actor keep their existing independent runtime transactions and failure surfaces.

## Evidence

Experiment 0092 passes 83 runtime, 26 prototype, and 20 host tests. The new table proves forward,
right, and diagonal Run under all four exact rotations.

`canonical-prototype-v16` passes in 72.161 seconds. One visible ordered E/VK 69 plus W/VK 87 process
emits two steps at world X/Z `-32/0`, moves X exactly `0 -> -64`, and commits Walk clip `0 -> 1`, yaw
`32,768`, epoch `1 -> 3`, two queries, grounded true, and zero blocks. The same frame commits camera
orbit 1, exact rig/anchor, and `[baseX+1,baseZ-1]` traversal. All prior process and lifecycle evidence
remains exact. Init and guard pass with zero Flavor denies and five existing warnings.
