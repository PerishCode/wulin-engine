# Experiment 0029: Authoritative Cooked Objects

Status: Accepted

- Related ADRs: [ADR 0032](../../docs/adr/0032-authoritative-cooked-object-payloads.md)

## Hypothesis

The accepted signed V2 object pack can be the sole content authority for canonical
composition without changing its payload schema or steady-state GPU execution. A fixed
probe-only readback of the 25 published object pages can drive CPU grounding, skeletal,
contact, semantic, and content oracles from the bytes actually visible to the GPU,
including legal records that the runtime cannot procedurally reproduce.

## Scope

`region-cooker` gains one deterministic authority fixture under the existing
1,024-record, 20,480-byte signed chunk schema. It preserves the accepted stable-seed
namespace and boundary continuity but changes interior Q9 positions and heights. The
fixture generator remains cooker-only; neither the workbench nor a shared runtime crate
can reconstruct it.

The async resident renderer gains one fixed 25-page readback resource. Only an explicit
composition probe transitions the published object pages to copy source, copies them in
active logical order, and restores shader-resource state. Ordinary frames, streaming,
prefetch, and publication record no readback work and retain their accepted submission
shape.

Composition evidence decodes those published bytes and uses them as every CPU oracle's
instance input. For an open cooked source, each observed page checksum is joined to the
exact signed region and compared with the pack index checksum. No persistent CPU object
payload mirror or independently evictable cache is introduced.

## Workload

1. Reproduce Experiment 0028 and its complete recursive compatibility chain unchanged.
2. Cook the authority fixture twice around `(2^40,-2^40)`. Require deterministic files,
   the same stable-seed namespace as arbitrary-Q8 compatibility content, a distinct
   complete-index source namespace, and payload bytes that the runtime fixture generator
   does not produce.
3. Publish the compatibility pack, then the authority pack with terrain fixed. Require
   terrain `25/0`, objects `0/25`, unchanged stable seeds, distinct content/position and
   attachment hashes, and one atomic pair.
4. Probe the authority pair. Require 25 active pages, 25,600 records, 512,000 copied
   bytes, one fixed readback allocation, exact active logical order, zero chunk mismatch,
   equal expected/observed checksum-index hashes, and CPU/GPU grounding and skeletal
   oracles with zero mismatch.
5. Move adjacent, diagonal, revisit, alias, prefetch, and rollover targets. Require the
   accepted `5/9/0/0` upload behavior, exact active-page authority after mixed retained
   and uploaded slots, and unchanged semantic inverse joins.
6. Hold object I/O and copy, inject missing/corrupt chunks, disable, and restart. Require
   complete old authority, no early readback identity, deterministic rollback/recovery,
   and no stale payload evidence.
7. Compare debug frames with and without a probe and run fixed release crossings. Record
   readback bytes/allocation and frame, pair, object I/O, and combined GPU distributions;
   make no speedup claim.

## Controlled Variables

- V2 header/index, stable-seed derivation, `InstanceRecord`, record/chunk counts, region
  cache keys, 50 object slots, 25 active entries, copy queue, descriptors, root constants,
  shaders, indirect execution, and atomic pair publication remain unchanged.
- Authority records remain finite, region-local, boundary-continuous, and exactly on the
  existing Q9 position lattice. No archetype, rotation, scale, variable-count, or public
  object-ID field is added.
- The readback resource is fixed at 512,000 bytes and has no source identity, eviction,
  or lifetime independent from the GPU resident cache. It is recorded only for a probe.
- Terrain content, camera, animation settings, capture settings, validation mode, and
  release sampling protocol remain fixed where comparisons require them.

## Metrics

- Pack file, source, stable-seed, index, chunk, payload, and authority fixture hashes.
- Active region order, physical slots, per-page observed/expected checksums, mismatch
  count, readback bytes/allocation, and decoded record/content hashes.
- Grounding numerators, position/boundary hashes, contact residuals, skeletal counters,
  animation/LOD/geometry oracles, semantic joins, and all frame attachments.
- Object I/O, copy/publication, probe, frame, and combined GPU median/P95/P99; worker,
  transaction, cache, readback-copy, descriptor, and dispatch capacities.

## Pass Criteria

- Experiment 0028 passes unchanged. Compatibility generated/cooked evidence remains
  equal and V1/default behavior remains unchanged.
- Authority content cannot equal runtime-generated records, yet uses the same stable
  seeds and existing V2 schema. Its distinct source invalidates object cache only.
- Every cooked probe consumes the records read back from the published GPU pages. All 25
  observed page hashes match the pack index in active signed-region order; no generated
  fixture is consulted for authority content.
- CPU/GPU grounding, skeletal, contact, semantic, and attachment evidence passes for
  authority data through movement, prefetch, rollover, holds, failures, and restart.
- Ordinary frames record zero active-page readback copies. Probe work is exactly 25
  copies and 512,000 bytes into one fixed resource. No CPU mirror, cache tier, unbounded
  allocation, validation error, lifecycle leak, or device removal occurs.

## Evidence

The canonical workflow will be:

```powershell
runseal :authoritative-cooked-objects
```

Generated evidence will remain ignored under
`out/captures/0029-authoritative-cooked-objects/`.

## Results

The complete recursive workflow passed. Experiment 0028 and its prior compatibility
chain passed unchanged before the authority-specific gate. The deterministic authority
pack contained 1,295 signed regions and 26,521,600 payload bytes. Its stable-seed
namespace remained `6007faad...b8`, matching arbitrary-Q8 compatibility content, while
its complete-index source namespace changed from `495b15c3...1a1` to
`616704b4...7ff6`.

The source switch retained/uploaded terrain `25/0` and objects `0/25`. The published
authority probe copied 25 pages and 512,000 payload bytes into one 524,288-byte D3D12
allocation. All 25,600 decoded records drove grounding, skeletal, contact, semantic,
and content evidence. The expected and observed active-index digest was
`a9499d93...332a`, every page checksum matched, and the active payload digest was
`1f1d2929...2431`.

Independent adjacent, diagonal, revisit, and alias cases preserved the accepted
`5/9/0/0` upload behavior. Prepared I/O/copy holds, rollover, missing/corrupt rollback,
disable, and restart retained complete old payload authority and recovered without stale
evidence. Ordinary frames and captures left readback counters unchanged. Each 32-sample
release sweep recorded exactly 32 probes and 800 page copies.

Reactive/prepared object I/O median/P95/P99 was `0.1470/0.1950/0.2262 ms` and
`0.1523/0.2067/0.2376 ms`; pair publication was `0.5161/0.9721/1.0771 ms` and
`0.3148/0.4425/0.5625 ms`. Combined composition GPU time was
`0.118784/0.141312/0.143360 ms` and `0.119808/0.141312/0.142336 ms`. No capacity,
validation, oracle, semantic, lifecycle, or device-removal failure occurred.

## Conclusion

Accepted. The signed V2 pack may be the sole canonical object content authority without
changing its schema or steady-state GPU execution. Explicit probes derive every runtime
oracle from the exact pages published to the GPU and join those pages to immutable pack
index checksums; no procedural reconstruction, persistent CPU payload mirror, second
cache, or ordinary-frame readback is required.
