# ADR 0056: Retained Terrain-Body Lifecycle

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0055 accepts one complete caller-owned terrain-body tick, but every body value still enters and
leaves by copy. A live driver cannot safely retain such a value in the host or workbench without
splitting engine state ownership. Conversely, introducing a general actor store now would require
an arbitrary capacity, free-list policy, entity taxonomy, and component model before a workload
needs them.

The smallest ownership proof is one neutral retained terrain body. Capacity one is a controlled
boundary rather than a player policy: it proves runtime ownership and lifecycle invalidation while
deferring multi-actor allocation until evidence exists.

## Decision

- `Runtime` owns one optional retained `TerrainBodyMotion`. No host, workbench, scene, or renderer
  field duplicates it.
- A successful spawn issues the next checked nonzero `u64` generation and returns the exact stored
  motion with its handle. Spawn while occupied fails without mutation.
- Read and despawn require the exact live generation. Despawn returns the exact removed value and
  empties the slot; respawn advances the generation so an older handle cannot alias the new body.
- Generation exhaustion is an explicit failure. It does not wrap to zero, reuse an old handle, or
  partially occupy the slot.
- The retained slot does not query terrain, consume simulation steps, observe input or time, drive
  frames, or bind presentation. Stored advance remains a separate experiment.
- Keep capacity at one until an independently specified multi-body workload can choose and measure
  bounded storage policy. Do not generalize this slot into an ECS by anticipation.

## Consequences

- The engine now has a concrete process-local simulation-state owner that a later stored-advance
  transaction can use without moving authority into an application.
- Generation handles make destroy/replace and stale caller state observable even at capacity one.
- This decision does not yet establish actors, components, object-presentation identity, horizontal
  velocity, controls, live schedule driving, or persistence across process restart.
- Widening the slot later is an explicit storage/identity decision, not a hidden compatibility
  requirement of this API.

## Evidence

Experiment 0053 added the single slot and strict spawn/read/despawn diagnostics. All 54 focused
runtime tests passed, including empty, occupied, wrong/stale handle, exact rollback, respawn, and
generation-exhaustion coverage.

The 18.48-second real-process gate executed the lifecycle twice across a full process restart.
Both runs issued generations 1 then 2, preserved exact signed fixed-point motion, rejected every
invalid operation, and produced SHA-256
`74f1b0e22b17fdc603d66082773e0824e0a54307364b0e57c1162f4bc1e11ced`. Restart began empty;
simulation and presentation status remained unchanged with zero terrain, source, GPU, frame,
renderer, or synchronization work per operation. `runseal :guard` passed. The long canonical
workflow was not run because the frame and renderer do not consume the new CPU slot.
