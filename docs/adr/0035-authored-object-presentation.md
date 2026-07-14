# ADR 0035: Authored Object Presentation

- Status: Accepted
- Date: 2026-07-14
- Supersedes: ADR 0034 object-format clause
- Superseded by: None

## Context

Experiment 0031 converged the runtime onto signed terrain and schema-2 objects with
authoritative spatial records and authored local IDs. The live skeletal and surface
paths still derived archetype, material, yaw, animation enablement, clip, phase offset,
and variation from the stable identity key. That made the fixture deterministic but left
no explicit content authority for presentation and blocked representative asset work.

Experiment 0032 tests the narrower prerequisite: make those properties cooked data while
retaining the accepted spatial ABI, stable identity, signed addressing, 50-slot cache,
terrain-first composition, fixed GPU submission, traversal, and lifecycle contracts.

## Decision

- The only live object pack is signed schema 3. Each region contains exactly 1,024
  equally indexed 20-byte spatial records, 4-byte authored local IDs, and 16-byte
  presentation records.
- A presentation record explicitly selects archetype, material, Q16 yaw, and either a
  static sentinel or packed animation clip, phase offset, and variation.
- The three planes share one region checksum, source namespace, background read,
  cache-slot reservation, copy transaction, fence completion, active mapping,
  publication, readback authority check, and rollback result.
- Object GPU residency retains 50 fixed resources per plane. A cold 25-region window
  performs 25 page copies per plane; reuse and movement copy only newly assigned slots.
- Stable keys remain identity evidence only. Live shaders and CPU oracles must not derive
  presentation properties from stable identity or physical record order.
- Runtime time progression may advance the authored animation phase offset. It does not
  choose the authored clip or variation.
- Schema 2 has no live compatibility reader, writer, migration utility, or synthesized
  presentation fallback.

## Consequences

- Physically reordered object triples preserve all identity-keyed behavior and output.
- Cooked presentation mutations can change mesh, surface, orientation, or animation
  without changing spatial authority, semantic identity, grounding, or contact.
- The object cache owns 50 additional 16 KiB default-heap buffers, one bounded upload
  arena, 50 SRVs, and one bounded active-presentation readback allocation.
- The controlled catalogs still contain synthetic geometry, animation, and materials.
  General asset import, variable-size assets, texture/mesh streaming, and production art
  require later experiments.

## Evidence

Experiment 0032 passed the direct 426.1-second workflow with exact cold/adjacent/diagonal
triple copy counts, two physical orders, four isolated presentation mutations, GPU-read
triple checksums, all hold/failure/movement/traversal gates, a warmed 64-publication
resource plateau, and 16 complete lifecycle cycles. The ignored report is generated at
`out/captures/0032-authored-object-presentation/acceptance.json`.
