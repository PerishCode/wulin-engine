# ADR 0033: Canonical Object Identity Plane

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0032 accepts GPU-published cooked records as canonical object authority, but stable
keys, archetype selection, yaw, animation clips, and pose phases still consume physical
record ordinal. Reordering or eventually removing records would therefore silently
change the behavior of surviving objects even when their authored bytes were unchanged.

Experiment 0030 tests an explicit identity input without changing `InstanceRecord`,
cache capacity, active-set size, indirect submission, or the GPU-read authority model.

## Decision

- Signed object payload schema 2 stores exactly 1,024 20-byte records followed by 1,024
  unique `u32` authored local IDs. IDs form the region-local range `0..1023`; physical
  payload size is 24,576 bytes and the pack checksum covers both ordered planes.
- Payload schema 1 remains physically unchanged and is interpreted with ordinal local
  IDs. It incurs no identity I/O and no transaction identity copy when a slot already
  carries ordinal identity.
- The asynchronous cache owns 50 record resources and 50 fixed 4 KiB identity
  resources. Startup initializes every identity resource to the ordinal page once.
  Publishing explicit IDs, or restoring ordinals after explicit IDs, copies the
  identity page under the same reservation, copy fence, cache generation, and atomic
  composition token as its record page.
- Record and identity descriptors are published together. Meshlet and skeletal shaders
  use the published local ID for every stable behavior key; physical ordinal is storage
  position only.
- Composition probes read the 25 active record pages and 25 active identity pages into
  fixed resources. They reconstruct the exact physical `[records][IDs]` payload,
  validate it against signed pack-index checksums, and feed the same GPU-read pairs to
  CPU oracles.
- Authored local IDs are unique only inside one exact signed region. They are content
  identity inputs, not semantic attachment values or persistent gameplay/network IDs.

## Consequences

Cookers may reorder records without changing render, animation, grounding, contact, or
attachment behavior. A schema-2 source switch or five-region movement adds one 4 KiB
identity copy per uploaded region, while steady frames, retained regions, CPU
submission shape, cache capacities, and descriptor enumeration remain bounded.

The accepted logical capacity is 200 KiB across 50 committed default-heap identity
resources, plus a 200 KiB upload arena, 50 resident descriptors, and one fixed 100 KiB
active identity readback. Committed-resource alignment makes the physical default-heap
allocation larger and it remains explicitly reported. Record and identity state must
remain one transaction; mixed generations are errors.

This decision does not accept sparse occupancy, variable record counts, authored
archetype/rotation/scale, persistent public IDs, collision, navigation, networking,
legacy import, or mod content.

## Evidence

- [Experiment 0030](../../experiments/0030-canonical-object-identity-plane/README.md)
  records recursive compatibility, deterministic order variants, exact dual-plane GPU
  authority, movement, alias, holds, rollback, restart, and release distributions.
- Two physically distinct 1,295-region packs produced equal identity-keyed behavior and
  byte-identical attachments. Each probe validated 25,600 unique IDs and all 25 exact
  signed page checksums after 614,400 bytes of GPU readback.
- Two 32-sample sweeps copied exactly 800 record and 800 identity pages. No validation,
  capacity, oracle, lifecycle, or device-removal failure occurred.

## Reproduction

```powershell
runseal :canonical-object-identity
```

The command writes the ignored report to
`out/captures/0030-canonical-object-identity-plane/acceptance.json`.
