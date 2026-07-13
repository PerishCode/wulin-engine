# ADR 0023: Signed Terrain Addressing

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0022 accepted exact signed global coordinates and bounded camera-relative rendering,
but all streaming paths still identify logical residency with a format-V1 `u32` region
ID. That value also selects pack content, GPU placement, and semantic projection.
Changing all of those roles together would couple global identity to format, shader,
perception, and composition migrations before any signed cache behavior was measured.

Experiment 0020 tests a narrower terrain-only boundary. It preserves the accepted
local terrain pipeline while proving that signed logical identity can own cache
residency at distant anchors without false hits or work proportional to coordinate
magnitude.

## Decision

- A requested terrain window contains signed 64-bit global origin and center region
  coordinates plus an active radius. The experiment's local format origin is fixed at
  `(64,64)`.
- Mapping subtracts signed global origin from global center with checked integer
  arithmetic, adds the bounded local origin, and validates the resulting `LoadConfig`.
  Global coordinates are never directly converted to `u32` or `f32`.
- Global terrain cache identity is the pair of signed global region and local format-V1
  region ID. The local ID is an explicit content binding: changing the alias for the
  same global key cannot reuse different pack content.
- Legacy terrain scheduling remains in a separate key namespace with no global key.
  Its request and serialized probe contracts remain unchanged.
- The background reader, terrain pack V1, default-heap slots, descriptors, shaders,
  GPU placement, semantic IDs, fixed dispatch shape, and payloads continue to consume
  only bounded local IDs.
- Reservation includes complete global/local assignments. I/O validation, copy
  submission, and immutable frame-boundary publication carry the same assignments;
  cancellation before worker submission and existing failure rollback leave the
  committed cache and published snapshot unchanged.
- Global probe evidence hashes signed global keys and local content bindings independent
  of physical slots. Existing active-mapping evidence continues to hash physical slot
  placement separately.
- Capacity remains 50 physical slots and 25 active entries, with one bounded I/O request
  and one bounded copy transaction. Coordinate magnitude creates no additional work or
  allocation.

## Consequences

Anchors at zero and plus/minus 2^40 regions produce distinct exact global mapping hashes
while sharing the canonical local terrain payload, GPU work, and attachments. Reusing
the same local aliases at another global anchor uploads all 25 regions instead of
false-hitting. One-region movement retains 20 entries and reads/uploads five; a resident
revisit retains all 25 and reads/uploads zero.

Signed terrain identity can now become an input to a later atomic terrain/object global
composition experiment. That later gate must decide how object pack lookup, object
cache identity, GPU stable keys, semantic identity, camera traversal, and render-origin
policy consume global coordinates together.

This decision does not change terrain format V1, authored pack partitioning, GPU terrain
coordinates, terrain semantic IDs, object residency, atomic composition, automatic
camera traversal, prefetch, collision, navigation, networking, or persistence. The
fixed local alias window remains an explicit experimental adapter, not a general world
format.

## Evidence

- [Experiment 0020](../../experiments/0020-signed-terrain-addressing/README.md) records
  exact far anchors, cache overlap, alias rebinding, I/O/copy holds, rejection, restart,
  compatibility, attachments, and release distributions.
- Experiments 0013 and 0018 passed unchanged; Experiment 0018 recursively passed its
  complete 0017 through 0015 compatibility chain.
- Across 32 release adjacent moves, each transaction read and uploaded exactly five
  regions. Across 32 resident revisits, each transaction read and uploaded zero.

## Reproduction

```powershell
runseal :global-terrain
```

The command writes the ignored report to
`out/captures/0020-signed-terrain-addressing/acceptance.json`.
