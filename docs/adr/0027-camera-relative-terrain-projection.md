# ADR 0027: Camera-Relative Terrain Projection

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0026 made V2 terrain content and GPU residency independent of bounded local aliases,
but rendering still used the alias `region_id` for positions, LOD patch coordinates,
and object IDs. Compensating camera and terrain by equal local offsets was
mathematically equivalent but not bit-equivalent after absolute `f32` view transforms,
and local object IDs changed perception attachments.

Experiment 0024 tests whether an explicit bounded projection can remove both leaks
without sending signed coordinates to the GPU, changing HLSL, or expanding established
root constants and render targets.

## Decision

- Every committed terrain snapshot derives one `TerrainProjection`. Format V1 has no
  source namespace and uses an exact passthrough projection. V2 source identity selects
  the canonical projection; callers cannot toggle the mode.
- V2 active row/column and radius project into the existing 128 by 128 region grid
  centered at `(64,64)`. The projected region ID replaces the local alias only in the
  high bits of the existing `slot | region_id << 6` GPU mapping constant.
- Existing terrain HLSL continues to derive mesh positions, patch coordinates, LOD, and
  `R32_UINT` object IDs from that bounded projected region ID. No signed coordinate,
  physical slot, source hash, new root constant, descriptor, or shader branch becomes
  semantic identity.
- Before building the terrain view-projection matrix, CPU camera position and target
  subtract `(local active center - 64) * 16` meters. Integer region offsets are bounded
  before exact conversion to `f32`; the same projected camera drives the CPU LOD oracle.
- Probe evidence exposes the alias center/offset separately from a canonical table. Each
  table entry joins active index, signed region, centered render offset, projected
  semantic region ID, and object ID. The table validates inverse mapping and collisions.
- Projected object IDs are frame-local handles. They are stable for one ordered global
  window across alias changes, but persistence and external identity remain the signed
  region in the CPU table.
- V1 root constants, camera, local IDs, status JSON, probe shape, attachments, and
  terrain behavior remain unchanged. V2 format, cache, I/O, copy, publication, and
  source-namespace contracts remain unchanged.

## Consequences

One signed 25-region window renders identically through local centers 2, 64, 96, and
125. Full-resolution and LOD color pixels, PNG, object-ID bytes, diagnostic PNG, sample
evidence, projected camera, view matrix, semantic table, geometry, and CPU/GPU oracles
are byte-identical. Every rebind retains all 25 canonical slots with zero I/O/upload.

Adjacent global windows still retain 20 and upload five exact payloads. Their bounded
projection shape remains constant while the inverse table changes to the new signed
regions. I/O/copy holds keep the complete old projection and attachments until one
frame-boundary publication.

This decision removes the terrain rendering and perception blocker for origin rollover.
It does not accept persistent object IDs, V2 composition, camera-relative generated
objects, cooked global objects, automatic traversal/rebase, authored partitioning,
prefetch, collision, navigation, networking, or an unbounded map.

## Evidence

- [Experiment 0024](../../experiments/0024-camera-relative-terrain/README.md) records
  four alias extrema, full-resolution and LOD exact attachments, semantic inversion,
  adjacent movement, holds, restart, compatibility, and release distributions.
- Experiment 0023 and its complete recursive compatibility chain passed unchanged.
- Across 32 release alias rebinds, every transaction retained 25 entries with zero I/O
  and upload. Across 32 adjacent moves, every transaction retained 20 and uploaded five
  4 KiB payloads.

## Reproduction

```powershell
runseal :camera-relative-terrain
```

The command writes the ignored report to
`out/captures/0024-camera-relative-terrain/acceptance.json`.
