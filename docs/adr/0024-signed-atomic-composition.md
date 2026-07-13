# ADR 0024: Signed Atomic Composition

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0023 accepted signed global identity for terrain residency while object residency
and atomic composition remained keyed only by a bounded format-V1 region ID. Reusing
that local ID at distant anchors could therefore make terrain exact while objects
false-hit an unrelated cache entry, or allow the two halves to describe different
logical windows.

Experiment 0021 tests the narrowest shared boundary. It retains the generated object
fixture and every accepted local GPU, format, and semantic contract while requiring one
signed window to own both cache identities and one atomic pair publication.

## Decision

- `GlobalRegionConfig` is the shared streaming address owner. It maps one signed 64-bit
  origin, center, and radius to an ordered 25-entry signed/global-local window with
  checked integer arithmetic and a fixed local origin of `(64,64)`.
- Terrain and generated-object cache keys both contain the signed global region and the
  bounded local content ID. Legacy local scheduling remains in a distinct namespace
  whose global key is absent.
- `composition.global.schedule` maps the window once, reserves both caches with that
  same immutable configuration, and records the global configuration on both
  transaction reports.
- Pair validation requires matching local and global configurations, transaction IDs,
  ordered local IDs, and complete staged halves. One frame-boundary commit publishes
  both snapshots and one physical-slot-independent global mapping hash.
- Terrain pack preflight occurs before generated-object materialization. A missing
  terrain alias cancels both reservations and submits no generated object payload.
- One terrain I/O, one terrain copy, one generated-object copy, and one composition pair
  remain the bounded in-flight capacities. Each terrain and object cache remains 50
  physical slots with 25 active entries.
- Terrain format V1, region format V1, generated instance records, GPU descriptors,
  local `region_id`, stable GPU keys, semantic IDs, shaders, grounding, LOD, and fixed
  submission continue to consume bounded local IDs only.
- Global scheduling is explicit and rejects active camera traversal. Automatic
  traversal and render-origin policy are not changed by this decision.

## Consequences

Equal local aliases at zero and plus/minus 2^40 regions produce three distinct exact
global pair hashes while composed local probes and attachments remain byte-identical.
Changing to an unseen global origin uploads all 25 entries in both caches instead of
false-hitting. Adjacent movement retains 20 and uploads five in both halves; a resident
revisit retains 25 and uploads zero.

Terrain I/O, terrain copy, and generated-object copy can each finish after the other
half without exposing a mixed pair. The complete old pair remains visible until both
halves stage and one matched commit occurs. Missing aliases, range failures, and signed
overflow do not change reservations, committed caches, pair counters, mappings, or
attachments.

This decision accepts signed identity for the deterministic generated-object cache, not
a signed cooked-object pack or persistent world format. It does not accept automatic
global traversal/rebase, authored partitioning, prefetch, collision, navigation,
network coordinates, or direct signed coordinates in GPU and semantic contracts.

## Evidence

- [Experiment 0021](../../experiments/0021-signed-atomic-composition/README.md) records
  exact far anchors, matched movement, three independent holds, transactional rejection,
  alias rebinding, restart, compatibility, attachments, and release distributions.
- Experiment 0020 passed unchanged and recursively replayed Experiments 0018 through
  0015. The local composition and traversal contracts remained unchanged.
- Across 32 release adjacent moves, both halves uploaded exactly five entries. Across
  32 resident revisits, both halves uploaded zero bytes.

## Reproduction

```powershell
runseal :global-composition
```

The command writes the ignored report to
`out/captures/0021-signed-atomic-composition/acceptance.json`.
