# ADR 0167: Shared-Pose Palette Capacity

- Status: Accepted
- Date: 2026-07-20
- Experiment: 0164 Shared-Pose Palette Capacity

## Context

The canonical skeletal scene has 25,600 streamed candidates plus one retained actor
candidate. Its palette resource was sized for one 128-bone palette per candidate,
reserving 157,292,544 bytes.

The live renderer constructs only shared-pose settings. Shared palette slots are the
existing exact rig/clip/phase key, and the fixed catalog domain contains two rigs,
eight clips per rig, and 64 sampled phases: 1,024 keys. At the retained 128-bone,
48-byte affine-transform stride, that domain requires 6,291,456 bytes.

The historical unique-pose workload proved a worst case in Experiment 0010, and ADR
0013 explicitly states that its candidate-sized reservation is evidence rather than a
permanent memory policy. No current Runtime, frame transaction, wrapper, or maintained
operator can select the unique branch.

## Decision

- Make `MAX_SHARED_POSES` the sole palette-slot capacity.
- Retain the exact 128-bone palette stride and existing rig/clip/phase key.
- Remove the unreachable `unique_poses` live setting and shader/oracle/report branches
  without an alias, fallback, or compatibility mode.
- Preserve candidate/visible capacity, fixed CPU submission, root signature/root
  constants, descriptor ownership, resource states, copy/readback work, barriers,
  fences, and immutable frame use.
- Reject any palette index outside the fixed shared-key domain in focused probes and
  static guards.
- Defer bone-stride tightening, palette paging, multi-actor capacity, and catalog
  generalization to separate experiments.

## Consequences

- The sole palette shrinks by 151,001,088 bytes, from approximately 150.006 MiB
  to 6 MiB.
- Live shared animation work and every downstream surface/shadow/occlusion consumer
  should remain structurally unchanged.
- The historical fully unique workload will no longer be a live engine mode. Requiring
  per-candidate unique palettes in the future needs a new bounded storage experiment
  rather than restoration of the retired branch.
- The tighter fixed capacity becomes a prerequisite for later bounded multi-actor
  work.

## Evidence

Experiment 0164 reduced the exact allocation from 157,292,544 to 6,291,456
bytes while retaining 25,601 candidates, 60 root DWORDs, six skeletal
dispatches, 968 active poses, 6,937 pose reuses, exact CPU/GPU oracles, and the
fixed color/object-ID hashes.

The same-profile process checkpoint fell from 411,951,104 to 261,439,488 private
bytes. Two 32-sample release traversal lanes improved P50 from 0.457728 to
0.408576 ms and from 0.663552 to 0.422912 ms; P95/P99 also decreased. Focused
frame/actor GPU, 64-publication/16-lifecycle deep resources, full runtime,
repository guard, init, and cleanup gates passed.
