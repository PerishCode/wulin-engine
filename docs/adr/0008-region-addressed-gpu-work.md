# ADR 0008: Region-Addressed GPU Work

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

The accepted calibration and perception paths proved exact pixels and semantic IDs, but
they did not establish how a large world avoids turning total object count into per-frame
CPU or GPU work.

Experiment 0005 represented one to sixteen million logical procedural instances in a
fixed global region address space. Only a 25-region active window generated candidates;
a compute pass compacted frustum-visible references and one indirect draw consumed them.

## Decision

- Large-world work is addressed by explicit fixed-size regions. Configured world extent
  does not imply scanning or submitting every logical object.
- Only active regions generate candidate work. The active region list/window and
  instances per region determine dispatch size.
- Visibility compaction is GPU-owned and produces a compact reference buffer plus an
  indirect argument. The CPU does not read visible instances to build draw calls.
- A render class consumes its compacted instances through a bounded number of indirect
  submissions; Experiment 0005 uses one.
- Procedural region proxies occupy a versioned semantic ID range so load frames preserve
  the object-ID perception contract.
- Explicit performance probes may batch identical iterations in one command list to
  remove idle-clock noise. Reported GPU values are per-iteration averages and must record
  the batch size.
- Calibration remains the startup mode. Synthetic load modes require explicit Sidecar
  configuration and must be reversible without changing calibration evidence.

## Consequences

Future world, streaming, vegetation, crowd, and animation systems should make inactive
regions cost no dispatch or draw preparation work. Total logical world size remains
metadata until regions become active.

Experiment 0005 uses procedural instance addresses and therefore does not prove storage,
upload, streaming, animation, LOD, or occlusion behavior. Those concerns require new
experiments with this region/compaction boundary held constant.

## Evidence

- [Experiment 0005](../../experiments/0005-gpu-region-compaction/README.md) records fixed
  candidate/visible counts, fixed command shape, stable semantic frame hashes, and nearly
  flat GPU distributions while logical world size grows sixteenfold.
