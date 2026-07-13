# ADR 0018: Atomic Terrain-Object Composition

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADRs 0010, 0012, and 0016 independently accepted immutable instance publication,
direct skeletal mesh execution, and full-resolution streamed terrain. None established
that two independently completed streams could become visible as one scene state, or
that objects could consume terrain data without CPU per-object preparation. Publishing
either new half early would combine different logical regions and invalidate grounding,
depth, and semantic evidence.

Experiment 0015 tested the smallest composition boundary: the existing 25-region,
25,600-instance fixture joined to exact cell-center terrain heights while both accepted
renderers share their reverse-Z depth and object-ID attachments.

## Decision

- A workbench-owned coordinator assigns one monotonically increasing pair token to a
  terrain transaction and an instance transaction with the same complete `LoadConfig`.
  Transfer completion may stage either half, but renderer-visible publication occurs
  only when both transaction IDs, configs, and canonical logical mappings match.
- Published and staged slots from both caches remain protected. A one-half hold keeps
  rendering the complete old pair. Failure discards a staged counterpart and records a
  bounded rollback; a retry may reuse verified cache payloads without exposing them
  before the new pair commits.
- Composition instance reservation uses high-slot-first placement while standalone
  instance and terrain paths retain their accepted policies. This makes physical
  divergence measurable without changing standalone behavior. Logical region order,
  not physical slot equality, owns the join.
- Each packed active mapping stores the instance slot in the low six bits and terrain
  slot in the next six bits. The existing fixed skeletal cull dispatch reads the two
  terrain diagonal endpoints for every cell-center candidate, writes one signed integer
  numerator, and culls at that grounded center. Mesh emission reads the same numerator.
- The bounded ground buffer contains at most 25,600 signed 32-bit values. It is copied
  to 102,400 bytes of readback only for a requested probe. The CPU independently
  reconstructs every numerator from decoded terrain tiles and requires exact equality.
- Terrain LOD, surface resolve, and occlusion are disabled. Full-resolution terrain and
  direct skeletal mesh passes share the existing color, `R32_UINT` object-ID, and
  reverse-Z depth targets. The first subpass clears semantic and depth attachments once;
  the second preserves them. Both submission orders remain available as a validation
  control.
- Standalone terrain and skeletal publication, rendering, probes, and controls remain
  independent. This decision does not promote a general scene graph, ECS, or engine
  streaming API.

## Consequences

One immutable pair now spans independently scheduled I/O and copy queues without
requiring CPU object enumeration or synchronized physical cache layouts. Canonical
terrain and instance mappings differ in all 25 slots, yet all 25,600 ground numerators
and all grounded skeletal aggregates match independent CPU oracles exactly.

Grounding is fused into the accepted cull dispatch, so its timing is reported together
with classify rather than by a duplicate timing-only workload. The extra persistent GPU
resource and requested readback are each 102,400 bytes. Pair publication remains one
in-flight bounded transaction and can be delayed by presentation/frame-boundary
latency; optimized measurements characterize that cost but do not establish a speedup.

The accepted formula applies only to the generated cell-center fixture on the fixed
terrain diagonal. Arbitrary coordinates, barycentric triangle selection, authored
placements, terrain LOD error policy, foot placement, slope orientation, IK, collision,
navigation, surface resolve, occlusion, transparency, and broad compatibility remain
unaccepted.

## Evidence

- [Experiment 0015](../../experiments/0015-atomic-terrain-object-composition/README.md)
  records pair staging, exact GPU/CPU grounding, independent physical mappings, shared
  attachments, both render orders, movement, holds, corruption rollback, retry,
  restart, visual captures, and optimized distributions.
- Canonical GPU and CPU ground hashes are identical over 25,600 values with zero
  mismatch. Grounded skeletal counters and terrain geometry/edge counters exactly
  match their CPU oracles.
- Terrain-first and object-first color and object-ID buffers are byte-identical and
  contain both known semantic ranges with no unknown ID.
- Terrain I/O, terrain copy, and instance copy holds preserve the complete old pair.
  A corrupted terrain payload after instance staging discards the instance half,
  preserves old attachments, and retries successfully without restart.
- Debug and release Sidecar namespaces complete without validation error, device
  removal, fallback, unbounded growth, or residual process.

## Environment

The ignored acceptance report records repository and tool revisions, Windows, adapter
and driver, D3D12 capabilities, both cache mappings and allocations, pair and stream
transactions, ground and attachment hashes, cameras, holds, failure/retry evidence,
process identities, and release distributions.

## Reproduction

```powershell
runseal :composition
```

The command writes the ignored report to
`out/captures/0015-atomic-terrain-object-composition/acceptance.json`.
