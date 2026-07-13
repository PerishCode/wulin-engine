# ADR 0028: Canonical Generated-Object Composition

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0027 made V2 terrain rendering and perception independent of bounded local aliases,
but generated objects still derived cache identity, positions, animation keys, and
semantic IDs from format-V1 local regions. Signed V2 terrain therefore could not enter
atomic composition or automatic origin rollover without changing object content and
attachments at every alias rebind.

Experiment 0025 tests one integrated replacement: canonical procedural object identity
and payloads, shared camera-relative projection, and matched V2 terrain/object
publication. The replacement must preserve every legacy path and established GPU
capacity, root-constant, submission, grounding, animation, and perception contract.

## Decision

- Canonical generated-object cache identity is `{object source namespace, signed
  region}`. The source namespace hashes the procedural generator revision and fixture;
  terrain source identity is deliberately excluded.
- Canonical object payloads contain region-local positions and a CPU-derived stable
  seed. They contain no local alias, physical cache slot, projected semantic region,
  terrain source, or camera state.
- Arbitrary-Q8 positions derive exact signed-region fractions through modular integer
  arithmetic. Signed coordinates remain CPU identity and are never converted directly
  to GPU `f32` values.
- V2 composition derives one centered `TerrainProjection` from the committed terrain
  source and applies it to both halves. The existing mapping word carries bounded
  instance slot, terrain slot, and frame-local semantic region fields.
- Object shaders reconstruct bounded positions from region-local payloads and the
  projected semantic region. Canonical stable seeds drive appearance and animation;
  projected semantic regions drive frame-local object handles.
- The CPU skeletal oracle uses the same projected camera, positions, stable-key
  formula, grounding, and LOD inputs as the GPU. Probe evidence joins every object
  handle to object source, stable seed, signed region, and the matching terrain
  projection entry.
- Canonical snapshots are valid only through atomic composition. Standalone meshlet,
  skeletal, and surface modes reject them explicitly rather than interpreting them as
  legacy local payloads.
- Atomic publication remains the visibility boundary. A failed terrain half preserves
  the complete old pair. Valid immutable object cache work completed for the failed
  pair may remain resident and be reused by a retry without becoming visible early.
- Local and signed format-V1 reservations, payloads, hashes, status shapes, root
  constants, shaders, attachments, and operator workflows remain unchanged.

## Consequences

One signed 25-region terrain/object window is byte-identical through local centers 2,
64, 96, and 125. Both caches retain all 25 entries with zero I/O or upload. Adjacent
movement retains 20 and uploads five 4 KiB terrain payloads plus five 20 KiB generated
object payloads while fixed GPU submission and exact CPU/GPU oracles remain unchanged.

Switching terrain namespaces misses all terrain entries but retains all generated
objects because their source is independent. Terrain I/O, terrain copy, and object copy
holds expose only the complete old pair. Missing or corrupt terrain and standalone
mode requests cannot mutate the published pair; restart reproduces the base frame.

This decision removes the generated-object and V2 composition blockers for automatic
origin rollover. It does not accept rollover policy itself, authored or cooked global
objects, persistent public object IDs, surface/occlusion promotion for canonical
objects, prefetch, collision, navigation, networking, or an unbounded world.

## Evidence

- [Experiment 0025](../../experiments/0025-canonical-object-composition/README.md)
  records recursive compatibility, alias extrema, exact semantic joins, source
  independence, three independent holds, failure/retry, restart, and release
  distributions.
- Experiment 0024 and the complete recursive compatibility chain passed unchanged.
- Across 32 release alias rebinds, both halves retained 25 with zero transfer bytes.
  Across 32 adjacent moves, both halves retained 20 and uploaded five with no identity
  mismatch, semantic collision, unknown ID, oracle mismatch, or device removal.

## Reproduction

```powershell
runseal :canonical-object-composition
```

The command writes the ignored report to
`out/captures/0025-canonical-object-composition/acceptance.json`.
