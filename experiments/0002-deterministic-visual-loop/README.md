# Experiment 0002: Deterministic Visual Loop

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0003](../../docs/adr/0003-native-workbench-control-plane.md),
  [ADR 0004](../../docs/adr/0004-frame-artifact-contract.md)

## Hypothesis

The native workbench can convert one explicitly controlled D3D12 frame into a
project-owned PNG and structured frame manifest such that repeated captures of the same
state produce the same raw pixel SHA-256, a visible state change produces a different
hash, and Sidecar can correlate every artifact with the live runtime instance.

## Scope

The experiment includes a typed `workbench.capture` event, an explicit render-target to
copy-source transition, texture readback with row-pitch removal, deterministic PNG
encoding, raw pixel hashing, frame manifests, and a Runseal acceptance workflow.

It excludes desktop screenshots, scene geometry, cameras, depth, object IDs, assets,
materials, Render Graph, continuous video capture, asynchronous encoding, and capture
compatibility outside the reference platform.

## Workload

The acceptance workload starts the workbench through installed stable Sidecar, pauses
continuous rendering, and captures the fixed 1280x720 back buffer under these states:

1. Color A captured twice in one process.
2. Color B captured once after a visible control mutation.
3. Color A captured again after a Sidecar restart.

Capture IDs are fixed. The experiment overwrites prior generated artifacts under
`out/experiments/0002-deterministic-visual-loop/` so stale files cannot be mistaken for
additional evidence.

## Controlled variables

- Windows 11 x64 reference system.
- NVIDIA GeForce RTX 4070 Ti SUPER.
- Rust 1.94.1 MSVC toolchain.
- `windows` crate 0.62.2.
- DirectX 12 Agility SDK package 1.619.4, SDK version 619.
- Workbench client size 1280x720 and `R8G8B8A8_UNORM` swap-chain format.
- D3D12 debug layer enabled through the development build.
- VSync enabled; capture requests are issued while the frame loop is paused.
- Color A is `[0.08, 0.42, 0.24, 1.0]` and Color B is
  `[0.55, 0.08, 0.16, 1.0]`.
- Installed stable Sidecar owns every tested process instance.

## Metrics

- Raw tightly packed RGBA byte count and SHA-256.
- Encoded PNG byte count and SHA-256.
- Capture width, height, format, frame index, process instance, and git revision.
- GPU readback row pitch and allocation size.
- CPU capture, row-copy, hash, encode, and file-write duration.
- Workbench renderer state and last error at capture completion.
- Final Sidecar target and broker process counts.

## Acceptance criteria

- Every PNG is copied from the D3D12 back buffer; desktop capture APIs are not used.
- The PNG dimensions and manifest dimensions are exactly 1280x720.
- The frame manifest reports `R8G8B8A8_UNORM`, the reference adapter, no device removal,
  and the Sidecar-launched process identity.
- The two same-process Color A captures have identical raw pixel SHA-256 values.
- Color B has a different raw pixel SHA-256 from Color A.
- Color A after Sidecar restart has the same raw pixel SHA-256 as before restart.
- PNG and manifest files exist at the paths returned by `workbench.capture`.
- A capture may wait for its submitted GPU work, but frames without a capture preserve
  the existing buffered fence-recycling path and add no unconditional wait.
- Sidecar restart replaces the workbench process, and final stop leaves no target or
  broker process.
- The repository guard passes with the capture workflow included.

## Environment

Each frame manifest records revision, process ID, Sidecar launch state, adapter, feature
level, dimensions, pixel format, clear color, frame index, byte counts, hashes, and
capture timing.

## Reproduction

Run from the repository root with Sidecar 0.5.1 or newer installed from the stable
channel:

```powershell
runseal :visual-loop
```

Individual captures remain available for operator use:

```powershell
runseal :workbench start
runseal :workbench pause
runseal :workbench capture operator-check
runseal :workbench stop
```

## Results

Accepted results on 2026-07-12:

| Capture | Process | Frame | Pixel SHA-256 prefix | PNG SHA-256 prefix | GPU submit/readback | Pre-manifest write |
| --- | ---: | ---: | --- | --- | ---: | ---: |
| Color A 1 | 24124 | 10 | `3c48cae0702e` | `35d399ee1f32` | 1.94 ms | 198.61 ms |
| Color A 2 | 24124 | 11 | `3c48cae0702e` | `35d399ee1f32` | 1.25 ms | 197.39 ms |
| Color B 1 | 24124 | 12 | `6490eeac5503` | `d2bb2f27e172` | 1.49 ms | 201.88 ms |
| Color A restart | 24244 | 7 | `3c48cae0702e` | `35d399ee1f32` | 1.38 ms | 205.60 ms |

Every image contained 3,686,400 tightly packed RGBA bytes. The D3D12 copy footprint
reported a 5,120-byte row pitch and 3,686,400-byte readback allocation, so this fixed
width required no padding while the implementation still removed row pitch explicitly.

The development build spent approximately 50-52 ms hashing and 124-129 ms encoding each
PNG on the CPU. Those operator-only costs are not a frame-performance gate. The normal
render path did not execute capture transitions, wait for the capture fence, hash pixels,
or encode files.

All manifests reported the reference adapter, D3D12 feature level 12_1,
`R8G8B8A8_UNORM`, debug layer enabled, Sidecar launch identity, and no device removal or
last error. Final Sidecar status contained no target or broker process.

## Conclusion

Accepted. The workbench can produce deterministic, project-owned frame artifacts from
the D3D12 back buffer and correlate them with typed runtime state. Repeated fixed state
survived process restart without pixel or encoded PNG drift, while a controlled visible
mutation produced distinct hashes.

## Promotion

The workbench capture path becomes the observation substrate for the first spatial
calibration scene. No reusable engine crate is promoted: GPU readback, artifact encoding,
and manifests remain workbench-owned until another production consumer establishes a
shared owner.
