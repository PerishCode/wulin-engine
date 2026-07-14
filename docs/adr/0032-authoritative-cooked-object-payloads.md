# ADR 0032: Authoritative Cooked Object Payloads

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

ADR 0031 accepts signed V2 canonical object storage and bounded streaming, but its
compatibility fixture is procedurally reproducible. The composition probe still rebuilt
records on the CPU for grounding, skeletal, contact, semantic, and content oracles.
Consequently, the accepted runtime did not prove that arbitrary legal pack records were
the sole content truth actually consumed by the GPU.

Experiment 0029 tests that authority boundary without expanding `InstanceRecord`, pack
V2, cache capacity, descriptors, shaders, submission, or publication behavior.

## Decision

- Cooked canonical object bytes published in the resident cache are the sole instance
  input to composition probes and their CPU oracles. Runtime fixture regeneration is not
  an accepted oracle path.
- The async resident renderer owns one fixed 512,000-byte active-payload readback
  resource. D3D12 allocation size on the reference platform is 524,288 bytes. Only an
  explicit composition probe records its 25 ordered page copies; ordinary frames,
  captures, streaming, prefetch, and publication record none.
- A probe transitions each published active slot from shader resource to copy source,
  copies exactly 20,480 bytes in active logical order, and restores shader-resource
  state before submission completes.
- A cooked transaction carries the 25 exact signed-region index checksums into its
  immutable published snapshot. These checksums are private metadata, not a payload
  mirror or serialized transaction API. The probe hashes each GPU-read page and joins
  it to its signed region and expected checksum.
- Generated compatibility probes use the same GPU readback and decoded-record oracle
  path but expose no pack authority because no index checksums exist.
- The authority fixture is cooker-only. It preserves boundary records and the accepted
  stable-seed namespace while changing interior Q9 positions and heights. No runtime or
  shared fixture module may reconstruct it.
- Readback capacity, probe count, copy count, observed/expected index hashes, and payload
  hash are explicit evidence. A checksum mismatch, missing active page, state mismatch,
  or ordinary-frame copy is a failure.

## Consequences

Canonical composition can now validate authored records that the runtime cannot
procedurally produce. Grounding, animation, contact, semantic joins, and content hashes
describe the bytes visible to GPU execution, including retained/uploaded mixtures after
movement, prefetch, rollover, source changes, failures, and restart.

The accepted cost is one fixed probe resource and 25 copies only when an operator asks
for a composition probe. No CPU payload cache, independent eviction policy, worker,
descriptor mapping, shader branch, or steady-state transfer is added.

This decision does not accept a heterogeneous object schema, archetypes, rotation,
scale, variable record counts, persistent gameplay identity, collision, navigation,
networking, legacy import, or mod content.

## Evidence

- [Experiment 0029](../../experiments/0029-authoritative-cooked-objects/README.md)
  records recursive compatibility, deterministic authority cooking, source switching,
  exact active-page checksum joins, movement, holds, rollover, rollback, restart, and
  release distributions.
- The authority pack retained the accepted stable-seed namespace while receiving a
  distinct complete-index source namespace. All 25 active page checksums matched after
  GPU readback, and every CPU/GPU oracle passed for 25,600 records.
- Two 32-sample release sweeps recorded exactly 800 page copies each. Ordinary frames
  and captures recorded zero copies, and no validation or device-removal failure
  occurred.

## Reproduction

```powershell
runseal :authoritative-cooked-objects
```

The command writes the ignored report to
`out/captures/0029-authoritative-cooked-objects/acceptance.json`.
