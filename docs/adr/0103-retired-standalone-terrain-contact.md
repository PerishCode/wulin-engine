# ADR 0103: Retired Standalone Terrain Contact

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0100 Retired Standalone Terrain Contact

## Context

Exact terrain/body contact began as a caller-owned pure proof and later became an input to fixed
motion, planar translation, retained actor simulation, and the bounded canonical probe. The public
`Runtime::resolve_terrain_contact` method and matching workbench verb are now used only to repeat the
same arithmetic in diagnostic acceptance. They do not drive Prototype 0 or another engine owner.

Keeping the standalone chain would preserve two process-level ways to prove one private invariant,
plus a payload/dispatch/support surface that future product code could mistake for a live mutation or
gameplay authority. The generic canonical probe already checks 225 deterministic contact transitions,
and focused pure tests retain detailed validation/error coverage.

## Decision

- Delete `Runtime::resolve_terrain_contact`, the `CanonicalTerrainContact` protocol path, and the
  `canonical.terrain.contact` workbench verb without a compatibility alias or replacement.
- Delete standalone contact acceptance support and response-schema evidence.
- Keep `resolve_body_contact` private to the engine and preserve its use by motion, translation, and
  canonical probe owners.
- Keep contact result/classification types public where current public motion/translation result
  contracts expose them.
- Keep one 225-body process witness in the generic canonical probe and all focused pure tests.
- Replace the older dense-probe unknown-event witness with one current direct-verb witness rather
  than accumulating retired history events.
- Extend the stable removal guard to reject restoration of either dense or standalone surfaces and
  to require the private authority plus bounded witness.

## Consequences

- Product and engine callers have no standalone copied-value contact entry point. Contact is an
  internal invariant of the owners that actually consume it.
- Workbench protocol and full acceptance shrink together; an old direct request fails generically as
  an unknown event.
- The same arithmetic remains independently tested and process-proven without a duplicate Runtime
  route.
- Exact terrain height remains public because prototype grounding and simulation consume it.
- No contact arithmetic, gameplay policy, renderer/GPU work, resource owner, source/format/asset,
  networking, or Wulin behavior changes.

## Evidence

Experiment 0100 passes the repository guard with zero Flavor deny issues and all Rust/Deno/resource
tests. `canonical-runtime-v8` passes in 247.249 seconds with exactly one current old-verb rejection,
no dense-probe event, unchanged 75/75/75 contact classifications and 75 corrections, unchanged
contact hashes, A/B/rollback/restart/traversal equality, and two clean lifecycle cycles.

Resources remain 492 handles and 21 threads; private bytes rise 376,832 under the unchanged
allowance. The final divisible-by-twenty operator cleanup removes 20,264,755,259 bytes of ignored
compiler/generated data from `target/` and `out/` after all verification and commit hooks.
