# ADR 0007: Object-ID Perception Contract

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

Experiment 0003 established deterministic color pixels and a shared spatial vocabulary,
but an agent could only understand the frame through scene state or visual inference.
Interaction, visibility, and later world-region experiments need an exact correlation
between visible pixels and stable semantic objects.

Experiment 0004 proved a second integer render target, bounded screen analysis, semantic
registry joins, stable occlusion samples, and deterministic results across restart.

## Decision

- Semantic object visibility is represented by an `R32_UINT` object-ID attachment.
- ID `0` means no semantic object. Nonzero IDs come from the versioned scene registry and
  are unique within that registry.
- Color and object ID are outputs of the same draw submission and share geometry,
  viewport, scissor, and reverse-Z depth testing.
- Screen coordinates use a top-left origin. Regions are integer half-open rectangles.
- `perception.capture` is the typed operator event for one synchronized color and ID
  capture with an optional bounded region and explicit sample points.
- Exact evidence is a tightly packed little-endian `u32` artifact. A deterministic PNG
  is diagnostic only and never the source for semantic classification.
- Full ID readback and CPU histogram analysis remain an explicit evidence path, not a
  per-frame runtime workload.

## Consequences

Agents and tests can identify visible and occluding objects without image heuristics.
New render paths that claim semantic perception must preserve the integer-ID and depth
relationship or explicitly supersede this contract.

Large-scene visibility and interaction systems should reduce, compact, or query IDs on
the GPU instead of continuously reading a full attachment back to the CPU. This ADR does
not prescribe that future implementation.

## Evidence

- [Experiment 0004](../../experiments/0004-object-id-perception/README.md) records exact
  ID hashes, semantic region histograms, fixed occlusion samples, alternate-camera
  differentiation, and restart determinism.
