# ADR 0119: Native Held Camera Repeat

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0116 Native Held Camera Repeat

## Context

`HostInput` suppresses any key transition whose requested down state already equals the retained
held state. Unit tests cover this rule, but no real Win32 Prototype session proved it across the
bootstrap/readiness boundary. Camera E is the strict live oracle because an erroneous repeated
press edge would advance the four-state orbit before same-batch W locomotion is derived.

## Decision

- Maintain one real-process session that posts E-down before readiness without a key-up, then
  posts repeated E-down plus W-down after readiness and exits after a bounded delay.
- Require readiness orbit 1 and exact orbit-one negative-X/zero-Z Walk output from the same
  post-readiness batch.
- Keep state-change suppression in the sole existing `HostInput` owner. Add no input history,
  action queue, controller state, repeat flag, product report field, or compatibility schema.

## Consequences

- Duplicate native down suppression now has exact adapter-to-product process evidence.
- The harness gains one bounded acceptance-only session; host input, camera, locomotion, Runtime,
  and renderer behavior remain unchanged.
- This decision does not authorize key-repeat gameplay behavior, remapping, action buffering,
  input replay, another camera owner, or engine/GPU/resource changes.

## Evidence

Experiment 0116 passed `canonical-prototype-v32` in 109.679 seconds. PID 20468 retained one exact
window while startup E-down committed orbit 1, then repeated E-down and W-down were posted 2.2142
ms apart and Escape followed after 205.2474 ms. Completion produced exactly 11 orbit-one Walk
steps, X delta -352 Q9, Z delta 0, clip 1, yaw 32,768, and epoch `1 -> 24`. Actor identity/shape,
zero vertical velocity, clock discontinuity/stall counts, object state, render-block count, and
the two-value clean completion remained exact.
