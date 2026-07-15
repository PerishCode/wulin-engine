# ADR 0052: Canonical Terrain Position Translation

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0047 accepted signed region plus half-open local-Q9 terrain query positions. ADR 0049 reused
that value for caller-owned body contact, and ADR 0051 reused it for fixed vertical motion. The
identity is therefore no longer query-specific, but the current name and module expose no canonical
way to translate across a region edge.

Horizontal motion or actor transforms cannot safely depend on caller-specific normalization.
Driving the live schedule before a movable state owner exists would still advance only counters.
Conversely, composing horizontal movement with contact now would decide unrestricted upward
correction before step-height and slope policy have evidence.

Experiment 0049 isolates the missing lower-level dependency: one canonical terrain position and
one exact planar translation operation.

## Decision

- Replace `TerrainQueryPosition` directly with `TerrainPosition`; do not retain an alias, deprecated
  constructor, or second coordinate representation.
- Preserve signed `i64` region coordinates, local Q9 denominator 512, and the half-open
  `[-4096, 4096)` local interval on both axes.
- Translate caller-supplied signed `i32` Q9 displacement with Euclidean normalization over the
  8192-unit region side, then apply both region deltas through checked `RegionCoord` offset.
- Return a new copied value only after both axes and the signed region are valid. Failure has no
  partial output or runtime mutation.
- Keep terrain lookup, contact, vertical integration, time, input, actors, and rendering outside
  the translation operation.

## Consequences

- Terrain query, contact, vertical motion, later planar motion, and later actor transforms can name
  one canonical horizontal identity.
- Callers cannot create alternate seam conventions or duplicate negative-coordinate remainder
  behavior.
- The accepted operation expresses displacement only. Speed, acceleration, collision, slope,
  support, input, and scheduling remain later policies.
- The direct rename is intentionally breaking inside the pre-release repository and creates no
  compatibility surface.
- Pure tests and repository guard are proportionate evidence; a permanent runtime diagnostic or
  repeated GPU/lifecycle run would add cost without covering the changed behavior.

## Evidence

Experiment 0049 passed 37 focused runtime/workbench tests and `runseal :guard` with zero Flavor
denies. Its test-only 65,536-case sweep covered the complete local domain, far signed regions, and
full-width signed `i32` displacement. Every translation matched an independent `i128`
absolute-lattice oracle; result and replay SHA-256 were
`8bf1a9181426aadf6970009165f1269e9358463c58e2cca734435a5bc02ff683`. Explicit tests proved
positive, negative, diagonal, and multi-region crossings, inverse and partition invariance, and
transactional `i64` boundary failure.

All live query/contact/motion consumers use `TerrainPosition`; no old alias or constant vocabulary
remains. The pure value change preserved the serialized boundary and changed no process, frame,
renderer, GPU resource, or lifecycle path, so no permanent diagnostic or full canonical run was
added.
