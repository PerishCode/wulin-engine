# ADR 0010: Asynchronous Region Publication

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

ADR 0009 established bounded default-heap residency and transactional cache publication,
but Experiment 0006 performed copies and a CPU fence wait on the direct queue. That
operator transaction proved storage behavior without proving frame-overlapped transfer.

Experiment 0007 held a dedicated copy queue behind an unsignaled test fence while the
direct queue continued presenting and capturing the previously published snapshot.

## Decision

- Asynchronous region payloads use individually state-owned default-heap resources
  addressed through a bounded SRV descriptor table. Resource-state ownership is not
  tracked at sub-buffer byte ranges.
- Physical capacity must preserve the complete currently published active set while a
  disjoint requested set uploads. For 25 active regions, the proven minimum is 50 slots.
- The currently published active-slot mapping is immutable until all requested copies
  complete. Experiment 0007 publishes its 25 indices through root constants.
- Reused non-active slots transition to copy-destination state on the direct queue. The
  copy queue waits for that direct fence; the CPU does not.
- Copy completion is polled at frame boundaries. Uploaded slots transition to shader-
  resource state on the direct queue before the new snapshot is first consumed.
- In-flight transaction count, staging memory, descriptor count, and physical residency
  are explicit fixed capacities. Experiment 0007 permits one transaction and one
  1,024,000-byte persistent upload arena.
- A busy request is rejected without changing planned or published state. Ordinary
  frames never wait for a copy fence.
- The deterministic copy gate is test instrumentation only and must not become runtime
  scheduling policy.

## Consequences

Copy latency cannot expose partially updated active data or block the direct frame loop.
Per-slot resources spend more allocation metadata and alignment than one packed buffer,
but they make cross-queue write ownership explicit without relying on unsupported
subresource state tracking for buffer ranges.

The accepted model does not yet provide background I/O, cooked formats, decoding,
priority, cancellation, or multiple in-flight requests. Those systems may replace the
single upload arena and scheduler only while preserving immutable publication, protected
active slots, bounded resources, and visible backpressure.

## Evidence

- [Experiment 0007](../../experiments/0007-async-region-publication/README.md) records
  held-copy frame continuity, exact old-snapshot evidence, bounded rejection, queue/fence
  progression, incremental transfers, protected eviction, GPU probes, and restart
  determinism.
