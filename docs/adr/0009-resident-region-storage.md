# ADR 0009: Resident Region Storage

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

ADR 0008 bounded active GPU work independently of logical world size, but Experiment
0005 generated instance data procedurally in shaders. It did not prove persistent GPU
storage, incremental data movement, cached revisits, or bounded eviction.

Experiment 0006 replaced procedural candidate generation with persistent instance
records in a 49-region default-heap cache while preserving the accepted 25-region
compaction and indirect-draw shape.

## Decision

- Renderable region payloads reside in bounded default-heap storage. Upload resources
  are staging inputs and are not the ordinary frame's shader data source.
- A compact active-region mapping separates world-region identity from physical cache
  slots. Active rendering addresses cache slots through this mapping.
- CPU metadata owns deterministic residency and LRU decisions. An eviction candidate
  must not belong to the requested active set.
- A stream request is an explicit transaction: plan the next cache state, generate or
  decode only missing regions, stage changed slots and the active mapping, submit
  copies, wait for completion, then commit the CPU cache state.
- Failed or incomplete GPU work must not publish the planned cache state.
- Unchanged render, probe, and capture frames perform no implicit instance generation
  or upload. Region data movement is visible through typed transaction reports.
- Experiment 0006's synchronous direct-queue transaction and deterministic CPU record
  generator are validation mechanisms, not the final asset-streaming architecture.

## Consequences

Adjacent movement scales data transfer with newly entered regions, and cached revisits
transfer only the active mapping. World movement cannot grow resident instance storage
beyond its declared capacity. The existing GPU compaction, indirect submission, and
object-ID contracts consume stored data without learning about streaming policy.

The accepted fence wait deliberately blocks the operator transaction. Background I/O,
decoding, copy-queue scheduling, upload-ring reuse, and frame-overlapped publication
remain separate experiments. They must preserve transactional publication and explicit
transfer accounting rather than introducing implicit per-frame uploads.

## Evidence

- [Experiment 0006](../../experiments/0006-resident-region-streaming/README.md) records
  exact adjacent/revisit/teleport transfer volumes, bounded eviction, GPU probe
  distributions, visual and semantic hashes, and restart determinism.
