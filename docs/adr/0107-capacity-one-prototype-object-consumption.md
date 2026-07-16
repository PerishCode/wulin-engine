# ADR 0107: Capacity-One Prototype Object Consumption

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0104 Capacity-One Prototype Object Consumption

## Context

ADR 0106 proves an exact committed action and a bounded green acknowledgement, but the acted-on
canonical object remains selectable and renderable afterward. Mutating the canonical source would
mix product lifetime with immutable streamed presentation, while a general entity registry would
promote an unproven ownership model.

## Decision

- The prototype owns one runtime-session-scoped consumed source-qualified identity. The successful
  projected Activated frame commits it atomically with the existing action acknowledgement.
- Consumption capacity is exactly one. A later action attempt is consumed as
  `CapacityExhausted` without resolving or replacing the stored identity.
- The committed identity is immediately supplied as the optional nearest-query exclusion. The
  bounded scan still validates and counts all 25,600 candidates; it skips only that exact identity
  during winner selection. Invalid or stale exclusions remain strict errors.
- The 12-successful-frame green acknowledgement remains unchanged. After it ends, the consumed
  identity becomes an immutable frame suppression input.
- Suppression is projected only after pending publication and only for a matching source/window.
  It is packed into the sole unused skeletal root word and rejects the exact active-index/local-ID
  pair at the beginning of the existing skeletal cull. Every downstream visible, animation,
  shadow, occlusion, and surface path therefore consumes the same reduced set.
- Same-source window departure retains the consumed identity and a return suppresses it again.
  Source replacement clears consumption after the successful replacement frame. Process restart
  naturally clears the runtime-session policy.

## Consequences

- Canonical packs, published snapshots, caches, and visible-record identity remain immutable.
- The visible record stays 56 bytes and skeletal constants stay 60 DWORDs. No pass, resource,
  descriptor, allocation, copy, readback, synchronization, or GPU-side mutable object registry is
  added.
- The exact suppression transition invalidates occlusion history through the existing frame
  signature; subsequent suppressed frames replay with compatible history.
- This decision proves only capacity-one prototype consumption. It does not authorize inventories,
  drops, respawn, save/load, interaction dispatch, multiple consumed identities, networking, or
  Wulin semantics.

## Evidence

All workspace tests, clippy, Deno checks, Flavor, and repository guards pass. `canonical-frame-v10`
passes in 40.920 seconds with exact one-candidate skeletal/shadow/occlusion removal, CPU/GPU
equality, replay, and clear restoration. `canonical-prototype-v22` passes in 75.750 seconds with
native F+Enter+W committing ID 496 as the immediate exclusion while deferring suppression behind
the acknowledgement.

The final-worktree `canonical-runtime-v12` passes in 265.079 seconds. ID 987 suppresses in source A,
unprojects in source B, suppresses again after source-A revisit, unprojects outside the same-source
window, and suppresses again on return. Five warm and eight measured resource publications retain
492 handles and 21 threads; private bytes settle from 427,048,960 to 426,463,232. The report retains
24 files / 25,346,262 bytes.
