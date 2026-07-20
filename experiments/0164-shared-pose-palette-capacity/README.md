# Experiment 0164: Shared-Pose Palette Capacity

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-20
- Related ADRs: ADR 0167

## Hypothesis

The live skeletal path can size its GPU palette by the fixed 1,024-key shared-pose
domain instead of the 25,601-candidate scene domain. Removing the unreachable live
unique-pose branch and retaining the existing 128-bone stride should reduce palette
storage from 157,292,544 to 6,291,456 bytes without changing visible work, animation,
surface/shadow/occlusion results, fixed submission, data movement, or synchronization.

## Scope

- Prove that every live animated visible record uses the existing exact
  rig/clip/64-phase key in `0..1,024`.
- Size the sole palette resource from that fixed key domain.
- Remove the unreachable `unique_poses` live setting and its shader/oracle/report
  branches instead of retaining a compatibility mode.
- Preserve the fixed 128-bone palette stride, animation catalog, pose-key encoding,
  candidate/visible capacities, actor upload, descriptors unrelated to palette length,
  command sequence, and resource lifetime.
- Add focused static/runtime guards for the new capacity and retired branch.

Bone-stride tightening, palette paging, multi-actor storage, asset/catalog
generalization, source formats, gameplay, and host/prototype behavior are out of scope.

## Workload

1. Audit every palette index, descriptor element count, root constant, shader branch,
   CPU oracle, probe, report field, and acceptance read.
2. Capture a pre-change focused canonical-frame baseline with exact resource,
   workload, hash, copy/synchronization, and GPU-timing evidence.
3. Replace candidate-sized palette storage with
   `MAX_SHARED_POSES * BONE_COUNT * 48`.
4. Collapse cull/compact/pose and CPU-oracle execution to the sole shared-key path.
5. Run focused frame and actor GPU acceptance, the deep resource/lifecycle owner, and
   final canonical runtime integration.
6. Compare exact correctness/work counts and release GPU timing distributions with the
   recorded baseline.

## Controlled variables

- Reference Windows/D3D12 platform, pinned toolchain, Agility SDK, DXC, source packs,
  catalogs, camera, presentation ticks, and cooked content remain fixed.
- `RIG_COUNT=2`, `CLIP_COUNT=8`, `SAMPLE_COUNT=64`, and `BONE_COUNT=128` remain fixed.
- Streamed capacity remains 25,600 and the retained actor candidate remains the next
  exact index.
- Palette matrices remain 48-byte row-major affine transforms.
- Fixed dispatch/root-signature/resource-state/copy/fence ownership remains unchanged.
- No second palette, dynamic growth, paging fallback, compatibility flag, or benchmark
  mode is introduced.

## Metrics

- Palette bytes and descriptor element count; total execution-resource bytes; active
  poses, reused poses, evaluated bones, and written palette bytes.
- Visible/candidate/animated/meshlet/triangle counts; actor admission; surface,
  shadow, occlusion, color, object-ID, and palette-oracle results.
- Dispatch, descriptor, root-constant, copy, readback, barrier, fence, handle, thread,
  and private-byte evidence.
- Release GPU timestamp P50/P95/P99 for the maintained focused workload.
- Focused/full elapsed time, artifact bytes, tests, Flavor findings, and lifecycle
  cleanup.

## Acceptance criteria

- The sole live palette allocation is exactly 6,291,456 bytes, a 151,001,088-byte
  (144.006 MiB, 96.0%) reduction from the recorded baseline.
- Every live palette index is bounded by `MAX_SHARED_POSES`; candidate and visible
  capacities remain unchanged.
- `unique_poses` and its live shader/oracle/report branch are absent with a stable
  removal guard and no alias or fallback.
- Shared-frame workload counts, presentation, CPU/GPU palette agreement, surface,
  shadow, occlusion, color, and object-ID evidence remain exact.
- Dispatch count, root-signature/root-constant DWORD count, descriptor count, copy
  work, readback shape, barriers, fences, and synchronization ownership do not grow.
- Release GPU P50 does not regress by more than 10%; P95/P99 are recorded and must not
  reveal a repeatable structural regression.
- Focused frame/actor, deep resource/lifecycle, final runtime, repository guard, init,
  diff, and process-cleanup gates pass.

## Environment

Use the repository-pinned Rust/Deno/DXC toolchains, reference Windows 11 D3D12 host,
configured adapter/driver, and the existing debug/release Sidecar workflows. Record
exact versions and adapter/driver evidence from the maintained reports.

## Reproduction

```powershell
runseal :canonical-frame
runseal :canonical-actor
runseal :canonical-resources
runseal :canonical-runtime
runseal :guard
runseal :init
```

## Results

### Capacity and behavior

- The sole palette allocation is now exactly 1,024 slots x 128 bones x 48 bytes =
  6,291,456 bytes. The former candidate-sized allocation was 157,292,544 bytes,
  so the direct reduction is 151,001,088 bytes (144.006 MiB, 96.0%).
- The probe reports 1,024 slots, a 128-bone stride, 131,072 descriptor elements,
  and 6,291,456 storage bytes. Both the active-pose count and sampled palette slot
  are checked against the fixed domain.
- Candidate capacity remains 25,601, root constants remain 60 DWORDs, and the
  frame retains six skeletal dispatches. The unique-pose Rust/HLSL/oracle/report
  branch is absent and protected by the maintained structural guard.
- The canonical frame retained 968 active poses, 6,937 reused poses, 61,952
  evaluated bones, and 2,973,696 palette-write bytes. Color
  `8b13d214…4135` and object ID `01951615…a5b` are unchanged; CPU/GPU skeletal,
  surface, shadow, and occlusion oracles all pass.

### Resource and timing evidence

The pre/post `canonical-runtime-v20` release traversal distributions contain 32
samples per lane:

| Lane | Baseline P50/P95/P99 | Accepted P50/P95/P99 | P50 change |
| --- | --- | --- | --- |
| Reactive | 0.457728 / 11.666432 / 12.439552 ms | 0.408576 / 1.903616 / 10.097664 ms | -10.74% |
| Prepared | 0.663552 / 11.561984 / 12.633088 ms | 0.422912 / 6.139904 / 10.820608 ms | -36.27% |

The same-profile eight-publication checkpoint retained 506 handles and 24
threads while final process private bytes fell from 411,951,104 to 261,439,488:
150,511,616 bytes (143.539 MiB) less. The deeper owner completed 64
publications, a 60-second quiescent plateau, and 16 full lifecycles with stable
final 519 handles, 18 threads, and 262,778,880 private bytes.

Two focused debug runs exposed low-power short-pulse noise in the broader
mesh/surface timestamp segment while the adapter entered P8 at 210 MHz. The
directly relevant pose-evaluate P50 remained 0.318-0.319 ms versus the
0.328704-ms baseline, and the maintained continuous release distributions above
show no regression.

### Acceptance

- `canonical-frame-v12`: pass twice, final 47.656 seconds / 1,422,770 bytes.
- `canonical-actor`: pass, 87.186 seconds / 959,597 bytes.
- `canonical-resources-v2`: pass, 360.974 seconds / 49,970 bytes.
- `canonical-runtime-v20`: pass, 320.810 seconds / 7,540,145 bytes.
- Exact capacity tests, DXC shader compilation, workspace Clippy/release tests,
  11 acceptance-support tests, Sidecar plans, repository init, diff checks, and
  cleanup pass.
- Flavor reports zero denies and the same five existing warnings.
- Reference evidence used Rust/Cargo 1.94.1, Deno 2.9.1, Windows D3D12, and an
  NVIDIA GeForce RTX 4070 Ti SUPER with driver 610.74.

## Conclusion

Accepted. The fixed shared pose-key domain is the correct live palette capacity.
Candidate-sized storage and the unreachable unique-pose mode are not retained as
compatibility surfaces.

## Promotion

Promoted to the current runtime boundary and ADR 0167.
