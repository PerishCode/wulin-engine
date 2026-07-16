# ADR 0094: Committed Prototype Run Modifier

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0091 Committed Prototype Run Modifier

## Context

The accepted prototype owns fixed W/A/S/D reduction and atomically submits motion plus presentation
through one actor transaction. The normalized host already retains all nonzero `u8` virtual keys,
and the pinned Fox rig already contains canonical Survey, Walk, and Run clips. Run therefore needs
no host extension, engine action state, source cook, or renderer path.

The remaining decision is a product policy: whether a modifier changes both displacement and clip,
how diagonals normalize, and what happens when playable-boundary admission removes motion.

## Decision

- Held Shift modifies only nonzero W/A/S/D. It creates no toggle or retained gait state.
- Preserve Walk components 32/23 Q9 per step. Use Run components 64/45, with 45 the nearest integer
  diagonal normalization of 64.
- Carry `running` only in the private prototype command. Boundary admission recomputes it after its
  existing independent per-axis maximum-eight-step reduction.
- Select Survey for zero admitted motion, Walk for nonzero admitted motion without Run, and imported
  clip 2 for nonzero admitted Run. Preserve the existing exact eight-way yaw and committed
  stationary-facing lifetime.
- Motion and presentation remain fields of the sole transactional actor command. Zero-step and
  blocked candidates commit neither; no separate gait commit protocol is introduced.
- Generalize maintained native-input acceptance to ordered key batches and prove visible Shift+W.
- Add no acceleration, horizontal velocity, stamina, configurable binding/tuning, root motion,
  blending, air-control change, gameplay effect, networking, or Wulin policy.

## Consequences

- The prototype exposes one immediately usable Run modifier backed by already accepted source and
  GPU capabilities.
- Exact movement speed remains deterministic and fixed-step partition behavior is inherited from
  the existing transaction.
- Fully boundary-reduced, opposed, modifier-only, and focus-cleared input cannot publish Run
  presentation.
- Future movement dynamics or gameplay authority cannot treat this local boolean as persistent
  velocity, stamina, or network state; those remain separate experiments.

## Evidence

Experiment 0091 passes 83 runtime, 25 prototype, and 20 host tests. Focused tests prove exact
32/23 Walk and 64/45 Run components, all eight Run facings, Shift-only/opposed/focus behavior, and
maximum-batch boundary reduction.

`canonical-prototype-v15` passes in 72.651 seconds. A visible ordered Shift/VK 16 plus W/VK 87
process commits three steps at Z `-64`, moving exactly `0 -> -192`, with Survey-to-Run clip `0 -> 2`,
yaw `49,152`, epoch `1 -> 3`, three queries, grounded true, vertical velocity zero, following camera,
the existing traversal contract, and zero render blocks. All prior processes and lifecycle gates
remain exact. Init and guard pass with zero Flavor denies and five existing warnings.
