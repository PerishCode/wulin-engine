# Experiment 0005: GPU Region Compaction

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0006](../../docs/adr/0006-spatial-and-depth-convention.md),
  [ADR 0007](../../docs/adr/0007-object-id-perception-contract.md),
  [ADR 0008](../../docs/adr/0008-region-addressed-gpu-work.md)

## Hypothesis

A procedurally defined world can scale from approximately one million to sixteen million
logical instances without increasing active rendering work when instances are organized
into fixed regions, only the fixed active region window is dispatched, visible instance
references are compacted on the GPU, and one indirect draw consumes the compacted list.

## Scope

The experiment adds an explicit workbench load mode, a procedural region/instance
address space, compute reset and frustum-compaction pipelines, a compacted visible-index
buffer, one `ExecuteIndirect` draw, GPU timestamp and indirect-argument readback for
explicit probes, region-proxy object IDs, Sidecar load controls, and a Runseal scaling
workload.

It excludes persistent authored instance data, asset meshes, materials, animation,
occlusion culling, HZB, LOD, region streaming I/O, sparse resources, asynchronous copy
queues, ECS, arbitrary active-region sets, and production render-graph promotion.

## Workload

The canonical workload keeps one maximum 128x128 region address space. Each region
contains a procedural 32x32 grid of 1,024 upright quad instances. Configured worlds are
centered subsets with region-side lengths:

1. 32 regions: 1,048,576 logical instances.
2. 64 regions: 4,194,304 logical instances.
3. 128 regions: 16,777,216 logical instances.

For every world size, the active center is global region `[64, 64]` with radius `2`, so
exactly 25 regions and 25,600 candidate instances are eligible for dispatch. The camera
position is `[0, 30, 30]`, target is `[0, 0, 0]`, vertical FOV is 60 degrees, and the
accepted reverse-Z projection remains active.

Each size receives 16 warm-up probes and 64 measured probes. To remove idle-clock noise,
one probe records 64 identical compaction/draw iterations in a single command list and
reports per-iteration GPU timestamp averages plus the indirect instance count. One
renderer-owned color capture and one object-ID perception capture are retained for every
size under `out/captures/0005-gpu-region-compaction/`.

## Controlled variables

- Maximum region grid is 128x128; configured worlds are centered power-of-two subsets.
- Region width and depth are 16 meters.
- Every region contains 1,024 deterministic procedural instances.
- Active region radius is 2 and candidate count is exactly 25,600 at every world size.
- Compute thread-group size is 256; dispatch shape is `[25, 4, 1]`.
- Compacted capacity is fixed above the maximum candidate count.
- Probe iteration count is 64 for every size; normal visual frames execute one iteration.
- The compute shader performs reverse-Z clip-volume testing before appending a visible
  instance reference.
- The indirect argument contains one six-vertex instanced draw and is reset on the GPU.
- Region proxy IDs use a fixed procedural range derived from global region coordinates;
  zero remains background.
- Default calibration mode is unchanged and remains the startup mode.

## Metrics

- Configured region count and total logical instance count.
- Active region count, candidate instance count, compute dispatch shape, compacted
  visible count, and compaction ratio.
- Indirect draw count and submitted vertex/instance shape.
- GPU compaction, draw, and total milliseconds for every measured probe.
- Minimum, median, P95, P99, and maximum GPU time distributions per world size.
- CPU probe round-trip time, process identity, renderer/device errors, and Sidecar final
  process counts.
- Color, raw object-ID, diagnostic PNG, region semantic mapping, and fixed perception
  samples for every size.

## Acceptance criteria

- World size changes only configured region bounds and logical instance count; active
  region count, candidate count, dispatch shape, command count, and GPU resource capacity
  remain fixed.
- The three logical instance counts are exactly 1,048,576, 4,194,304, and 16,777,216.
- Every probe reports 25 active regions, 25,600 candidates, dispatch `[25, 4, 1]`, one
  indirect draw, and a visible count strictly between zero and candidate count.
- Visible count is identical across all probes and world sizes under the
  fixed camera and active window.
- The GPU median total time at the slowest world size is no more than 1.35 times the
  fastest median plus 0.02 ms; logical world growth must not create proportional GPU
  work.
- All GPU timestamps are finite and ordered: compaction, draw, and total are positive,
  and total covers both measured intervals.
- Color captures are pixel-identical across world sizes because the active world window
  and relative region colors are unchanged.
- Perception captures contain no unknown IDs and resolve visible pixels to the procedural
  global-region registry. Raw ID and diagnostic PNG hashes are identical across world
  sizes.
- Returning to calibration mode reproduces the Experiment 0003 default color hash and
  Experiment 0004 default object-ID hash.
- Every manifest/probe reports no device removal or renderer error.
- Final Sidecar status contains no target or broker process and `runseal :guard` passes.

## Environment

The final report records toolchain, revision, adapter, timestamp frequency, load
configuration, distributions, hashes, semantic evidence, and process cleanup.

## Reproduction

Run from the repository root with Sidecar 0.5.1 or newer installed from stable:

```powershell
runseal :region-load
```

## Results

Accepted results on 2026-07-12. GPU values are per-iteration averages from 64 iterations
per probe after 16 warm-up probes and across 64 measured probes:

| Region side | Logical instances | Visible | Compaction median | Total median | Total P95 | Total P99 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 32 | 1,048,576 | 18,928 | 0.00246 ms | 0.06555 ms | 0.07811 ms | 0.09411 ms |
| 64 | 4,194,304 | 18,928 | 0.00299 ms | 0.08510 ms | 0.11818 ms | 0.13365 ms |
| 128 | 16,777,216 | 18,928 | 0.00258 ms | 0.06947 ms | 0.08418 ms | 0.09832 ms |

All three sizes reported exactly 25 active regions, 25,600 candidates, dispatch
`[25, 4, 1]`, and one indirect draw per normal render iteration. The slowest/fastest
total median ratio was 1.298, below the preregistered bound. The visible count remained
18,928 and proved that the frustum path removed 6,672 candidates rather than merely
appending every active instance.

Every size produced the same color SHA-256 `9bd075106177...`, raw object-ID SHA-256
`8431f1c795ec...`, and diagnostic PNG SHA-256 `91c9fad2b269...`. Twenty visible region
proxies were semantically resolved with no unknown IDs; sample `[600, 600]` consistently
resolved to `load.region.064.065` ID `73921`. Color and ID non-background coverage was
identical.

After `load.disable` and camera reset, calibration color and raw ID hashes exactly
reproduced Experiments 0003 and 0004. All captures and probes reported no renderer error
or device removal, and final Sidecar status contained no process.

Median CPU probe round trips were 38.9-42.3 ms and include Sidecar transport, main-loop
wake-up, fence wait, readback, JSON encoding, and process scheduling. They are operator
latency rather than renderer CPU preparation time.

## Conclusion

Accepted. Region-addressed dispatch made active GPU work independent of configured world
extent across a sixteenfold logical-size sweep. The GPU produced the compacted draw count
and semantic frame directly; the CPU submitted no per-instance work.

## Promotion

ADR 0008 promotes region-addressed candidate generation, GPU compaction, and bounded
indirect submission as the baseline work model. The procedural renderer remains a
workbench experiment. The next load stage may replace procedural addresses with
persistent instance data while holding the accepted active-work shape constant.
