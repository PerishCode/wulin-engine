# ADR 0049: Exact Terrain Body Contact

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0047 established one exact committed-snapshot terrain height query, but deliberately excluded
body and contact semantics. A later locomotion or gravity experiment needs a non-penetration
transaction whose coordinates, failure behavior, and ownership do not depend on camera-relative
render state, terrain LOD, or a runtime actor store.

Introducing a persistent body collection now would combine contact with the later actor identity,
lifetime, and transform stage. Introducing time, gravity, tolerance, or downward snapping would
also make it impossible to isolate whether exact contact or step policy is responsible for a
result. Experiment 0046 tests the narrower dependency first.

## Decision

- Define a caller-owned terrain body value containing accepted signed-region/local-Q9 horizontal
  position plus signed Q16 center height and positive Q16 half-height. Public vertical components
  are bounded to signed `i32`; checked `i64` arithmetic owns intermediate separation/correction.
- `Runtime` exposes one read-only contact resolver. It samples the last committed terrain snapshot
  once, classifies the body's initial foot as separated/touching/penetrating, and returns the exact
  terrain height, signed separation, upward correction, and resolved body.
- Positive separation is never snapped downward. Exact zero remains touching. Negative separation
  receives only the minimum upward correction required to make the resolved foot equal terrain.
- The runtime does not retain, identify, time-step, render, or otherwise mutate the caller's body.
  Unavailable terrain, invalid body shape, identity mismatch, arithmetic overflow, and
  unrepresentable resolution fail explicitly without a previous-state or float fallback.
- The existing camera/LOD contact probe remains render-quality evidence and is not reused as
  physical contact authority.
- Dense 230,400-body evidence executes only through the explicit diagnostic acceptance command.
  Generic composition probes retain a 225-body deterministic witness covering every active region,
  terrain triangle, and contact class, so snapshot transitions remain observable without repeating
  dense work at every probe.

## Expected Consequences

- Later movement/gravity experiments can depend on one exact non-penetration primitive while
  separately choosing simulation time, velocity integration, input mapping, and grounded policy.
- Exact touching is intentionally strict. Any skin width, tolerance, slope limit, step height,
  footprint, or downward support probe requires its own evidence rather than silently entering this
  primitive.
- Body storage and identity remain deferred to the runtime-actor stage. A later host may retain one
  returned value without making this resolver responsible for lifetime.
- The committed terrain snapshot remains physically owned by the renderer; the body API receives
  no cache slot, tile, descriptor, projection, or render-LOD surface.

## Evidence

Experiment 0046 resolved 230,400 controlled bodies with exactly 76,800 results in each class,
76,800 minimum upward corrections, and zero oracle mismatch. Direct gates rejected unavailable,
invalid, outside-window, and unrepresentable inputs. The 225-body witness remained exact through
holds, rollback, reorder, alias, traversal, rollover, restart, resource plateau, and 16 lifecycle
cycles. The final 701.5-second direct workflow passed with no controlled GPU hash change, handle
growth, validation error, or device removal.
