# ADR 0011: Cooked Region Storage

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0010 accepted asynchronous GPU publication using deterministic in-memory payload
generation. It intentionally excluded a runtime disk format and background I/O, so it
did not prove that the resident cache could consume real indexed data without changing
its queue and publication guarantees.

Experiment 0008 introduced an offline cooker, a sparse immutable region pack, strict
runtime validation, and one gated background reader between cache planning and GPU copy
submission.

## Decision

- Canonical runtime region data uses an explicit versioned little-endian format owned by
  `crates/region-format`. Rust memory layout does not define disk layout.
- Runtime packs are immutable outputs of an offline tool. The engine opens them read-only
  and never treats hand-edited or legacy source data as canonical runtime data.
- A fixed header declares version and canonical dimensions. A sorted unique index owns
  region IDs, aligned absolute payload ranges, flags, and per-chunk SHA-256 values.
- Opening a pack validates only its header and complete index. Payload chunks are sought,
  read, hashed, decoded, and semantically validated only when requested.
- Cache planning reserves region-to-slot assignments without mutating committed cache
  state. Payload generation or I/O then materializes exactly that reservation.
- An I/O or validation failure cancels the reservation before copy submission. Cache and
  published GPU snapshot commit remain exclusively tied to the accepted copy-fence and
  frame-boundary publication path.
- Worker count, request channel, completion channel, cache reservation, and GPU transfer
  are explicit bounded capacities. Experiment 0008 accepts one of each.
- Payload source and preparation time are explicit in transfer reports. Cooked reads do
  not masquerade as procedural generation.
- The deterministic I/O gate is test instrumentation only and must not become runtime
  throttling policy.

## Consequences

Disk corruption and missing regions fail before GPU state changes, and region movement
performs I/O proportional to newly entering regions rather than total world extent. The
fixed format rejects silent schema drift and pays one SHA-256 plus structured decode per
loaded chunk.

Version 1 intentionally stores uncompressed fixed-size procedural instance records.
Compression, alternate record schemas, memory mapping, multiple workers, priorities,
cancellation, prefetch, remote storage, production assets, and legacy import remain
unaccepted. A later experiment may add them only while preserving bounded requests,
reservation rollback, strict validation, and immutable publication.

## Evidence

- [Experiment 0008](../../experiments/0008-cooked-region-io/README.md) records deterministic
  recooking, indexed incremental reads, held-I/O frame continuity, exact generated versus
  cooked upload hashes, visual and semantic reproduction, corruption rollback, restart
  determinism, and fixed worker/channel capacities.
