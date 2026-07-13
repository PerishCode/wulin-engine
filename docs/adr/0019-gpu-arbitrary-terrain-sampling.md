# ADR 0019: GPU Arbitrary Terrain Sampling

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0018 accepted exact GPU terrain grounding only for one object at the center of each
terrain cell. That fixture always sampled the shared triangle diagonal and could not
prove that the composed path followed the actual terrain mesh for arbitrary placement
or remained continuous when two owning regions described the same world-space edge
position.

Experiment 0016 isolates that missing capability without changing terrain format V1,
region format V1, pair publication, physical cache ownership, or fixed submission.

## Decision

- Composition may select an `arbitrary-q8` fixture before its first pair publication.
  Fixture identity is frozen into pending and published pair state. Changing fixture
  after publication requires a process restart because retained instance slots encode
  fixture-specific positions.
- Arbitrary XZ positions use a deterministic 1/512 meter lattice. Each 0.5 meter cell
  therefore has exact integer Q8 fractions `u` and `v`. Boundary rows and columns use
  fractions zero or 256 so adjacent owning regions can describe the same position.
- The accepted skeletal cull dispatch converts the exactly representable position to a
  region-local lattice coordinate and evaluates the same two triangles emitted by the
  terrain mesh. It writes one signed Q16 ground numerator per candidate. Mesh emission
  consumes that same buffer; there is no sampling-only dispatch.
- The requested probe regenerates the frozen fixture, evaluates every numerator with
  integer CPU arithmetic, hashes absolute XZ positions and ground values, classifies
  triangle coverage, and compares same-position samples across all active neighbor
  edges.
- The CPU skeletal oracle consumes the same frozen instance records and explicit ground
  denominator only for requested validation. Normal frame submission remains the
  accepted three terrain and five skeletal dispatch or indirect operations.
- Cell-center composition remains the default and preserves its accepted `/512`
  encoding and exact hash. Standalone instance generation is unchanged.

## Consequences

The composed path now supports deterministic arbitrary positions inside their owning
regions while preserving exact agreement with the terrain mesh and continuity on
region boundaries. The canonical 25-region fixture validates 25,600 Q16 values, both
terrain triangles, the shared diagonal, 40 logical neighbor edges, and 1,280 paired
boundary positions.

Fixture-specific instance payloads cannot be reinterpreted in place. A future runtime
fixture or authored placement transition must explicitly invalidate or republish those
slots rather than relaxing the restart invariant.

This decision does not accept sampling outside the owning region, terrain LOD
composition, normals, slope-oriented frames, feet or IK, collision, navigation,
authored asset placement, a general scene query, or a reusable engine subsystem. The
implementation remains workbench-owned until another experiment establishes that
boundary.

## Evidence

- [Experiment 0016](../../experiments/0016-gpu-arbitrary-terrain-sampling/README.md)
  records exact GPU/CPU values, position and ground hashes, triangle coverage, boundary
  pairs, independent physical mappings, both pass orders, cameras, movement, revisit,
  teleport, restart, compatibility, and optimized distributions.
- GPU and CPU produce the same SHA-256
  `c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db`
  over all 25,600 signed Q16 values with zero mismatch.
- All 1,280 boundary comparisons have identical XZ bits and ground numerators. Terrain
  and instance physical mappings still differ in all 25 active logical regions.
- Terrain-first and object-first color and object-ID attachments are byte-identical,
  contain both known semantic ranges, and contain no unknown IDs.
- The canonical workflow reruns Experiment 0015 and preserves its exact cell-center
  hash before accepting arbitrary sampling.

## Reproduction

```powershell
runseal :terrain-sampling
```

The command writes the ignored report to
`out/captures/0016-gpu-arbitrary-terrain-sampling/acceptance.json`.
