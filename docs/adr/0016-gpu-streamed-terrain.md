# ADR 0016: GPU Streamed Terrain

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADRs 0010 and 0011 established bounded asynchronous region publication and canonical
cooked instance storage. They did not establish a terrain representation, continuity
invariant, or GPU terrain execution path. Reinterpreting instance `region-format` V1
would have coupled unrelated payload semantics and hidden the terrain-specific shared
edge requirement.

Experiment 0013 tested the smallest terrain contract needed before LOD, editing, object
composition, collision, or content work: exact same-resolution edges from disk through
GPU expansion, bounded publication under movement, and an unchanged old frame under
I/O, copy, or validation failure.

## Decision

- Terrain owns an independent versioned pack. Each 16-meter region has one fixed 4 KiB
  payload containing a 33x33 signed 16-bit height lattice and 32x32 byte material field.
  Heights use 1/256-meter units; byte order, zero padding, index order, 4 KiB alignment,
  and SHA-256 coverage are canonical.
- Offline generation addresses every height by global integer sample coordinate.
  Adjacent payloads duplicate the complete shared edge byte-identically, and the writer
  rejects any generated neighboring mismatch before publication.
- One bounded background reader verifies requested chunks after a protected 50-slot
  reservation. One dedicated copy queue publishes a complete immutable row-major active
  mapping only at a frame boundary. I/O failure cancels before copy; active and in-flight
  slots cannot be overwritten.
- Active capacity is 25 regions. CPU always records one 400-group mesh dispatch and one
  `[25,2,1]` seam-oracle dispatch. Each active region expands into sixteen fixed 8x8-cell
  patches; inactive mesh groups emit zero work. CPU does not read GPU counts before
  submission.
- The GPU seam oracle decodes the same payload bytes used by mesh expansion and compares
  all 33 samples of every active horizontal and vertical neighbor edge. CPU and GPU
  counts and mismatches must agree exactly.
- Terrain writes the shared color and `R32_UINT` semantic attachments. Region semantics
  occupy the registered terrain ID range and join through the existing perception path.
  Probe readback is requested evidence only and is not part of frame submission.
- `terrain-format` and the offline cooker are reusable promoted owners. Streaming,
  committed resources, mesh execution, and control-plane integration remain owned by
  the workbench while later terrain architecture is still experimental.

## Consequences

The accepted path renders 25 regions as 400 patches, 32,400 vertices, and 51,200
triangles with one fixed mesh dispatch. Forty shared edges and 1,320 decoded samples are
checked by both CPU and GPU with zero mismatch. Radius, movement, cache history, holds,
failure recovery, and restart do not alter submission shape or canonical evidence.

Logical cache payload is 204,800 bytes and copy timing adds one 16-byte readback, but 50
individually committed 4 KiB resources consume 3,276,800 allocation bytes on the
reference device because of allocation granularity. The bound is acceptable evidence
for publication correctness, not an endorsement of the final allocation strategy.
Packed heaps or tiled resources require their own measurement.

Same-resolution continuity is accepted; terrain LOD and cross-resolution stitching are
not. Fixed 400-group overdispatch is also an experiment contract, not a general terrain
budget. Material blending, virtual texturing, authored data, editing, collision,
navigation, grounding, object composition, terrain occlusion, and broad compatibility
remain unaccepted.

## Evidence

- [Experiment 0013](../../experiments/0013-gpu-streamed-terrain/README.md) records pack
  validation, radius and boundary sweeps, CPU/GPU seam equality, movement and cache
  behavior, independent I/O/copy holds, corruption rollback, semantic attachments,
  restart determinism, optimized distributions, and prior-path regressions.
