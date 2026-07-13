# ADR 0013: GPU Skeletal Crowd Execution

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0012 established GPU-generated meshlet work for real static geometry, but every
visible object still used an immutable vertex stream. It did not prove that large
animated crowds could classify animation work, reuse shared poses, evaluate bounded
bone hierarchies, and skin meshlets without CPU-visible character, pose, bone, or
meshlet enumeration.

Experiment 0010 added a deterministic animation catalog and extended the accepted
cooked resident path through GPU animation classification, pose-key compaction,
hierarchical palette evaluation, and four-weight linear blend skinning.

## Decision

- Animated-object classification is part of GPU visibility work. CPU code may publish
  animation parameters and bounded capacities, but it does not enumerate visible
  animated characters.
- Shared poses are addressed by stable `(clip, quantized phase)` keys, compacted on the
  GPU, and evaluated once per active key. A bounded unique-pose mode must remain
  available to expose the per-visible-character worst case through the same pipeline.
- Skeleton hierarchy order is parent-before-child and has an explicit maximum depth.
  The accepted laboratory path evaluates up to 128 bones per pose and publishes 3x4
  row-major affine palettes.
- Mesh shaders apply four packed normalized influences per vertex before the existing
  instance transform, reverse-Z projection, color target, and semantic object-ID target.
- Measured submission remains fixed at one reset dispatch, one cull/classify dispatch,
  one pose-key compact dispatch, one indirect pose dispatch, and one indirect mesh
  dispatch. Workload changes may alter indirect arguments and GPU work only.
- Visible records, shared keys, unique poses, palettes, counters, samples, and indirect
  arguments have fixed capacities. Overflow is an error, not permission to enumerate or
  allocate from the CPU hot path.
- Validation requires exact aggregate agreement with a deterministic CPU oracle and
  bounded sampled-palette error. The oracle and readbacks are experiment controls, not
  ordinary frame dependencies.
- Correctness runs under the D3D12 debug layer. Performance distributions use a separate
  Sidecar-managed release workbench with the debug layer disabled, explicit preheat,
  per-workload warm-up, and 32 samples.
- Observed GPU timings characterize scaling on the reference machine; they are not
  architecture thresholds or hardware compatibility promises.

## Consequences

The accepted scene path can animate the canonical 18,928 visible-object crowd while CPU
submission remains independent of visible character, active pose, bone, and geometry
counts. Shared-pose work scales with pose diversity; fully unique work remains bounded
at 25,600 poses and exposes the intended worst case.

The current worst-case palette reserves 157,286,400 bytes so unique 128-bone execution
cannot grow storage at runtime. That reservation is accepted evidence, not a permanent
engine memory policy. A later experiment may introduce palette paging or tighter
publication only if it preserves bounded storage, immutable frame use, and fixed CPU
submission.

The deterministic rig, clips, stable-key assignment, integer animation tick, and skin
bindings are laboratory fixtures. Asset import, runtime compression, retargeting,
animation graphs, blending policy, root motion, IK, morphs, cloth, ragdolls, materials,
and character gameplay remain unaccepted.

## Evidence

- [Experiment 0010](../../experiments/0010-gpu-skeletal-crowds/README.md) records the
  catalog hashes, exact GPU/oracle counters, palette samples, animated/bone/pose/LOD
  sweeps, debug and release processes, timing distributions, deterministic captures,
  movement, restart, and prior-path regressions.
