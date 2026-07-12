# Experiment 0003: Spatial Calibration Scene

- Status: Accepted
- Owner: PerishCode
- Created: 2026-07-12
- Related ADRs: [ADR 0003](../../docs/adr/0003-native-workbench-control-plane.md),
  [ADR 0005](../../docs/adr/0005-capture-collection-contract.md),
  [ADR 0006](../../docs/adr/0006-spatial-and-depth-convention.md)

## Hypothesis

The workbench can render a deterministic procedural 3D calibration scene under one
explicit spatial convention, expose camera and semantic object state through Sidecar,
and reproduce the same renderer-owned pixels after process restart while a controlled
camera mutation produces a different frame.

## Scope

The experiment includes one D3D12 graphics pipeline, a compiled HLSL vertex/pixel shader,
procedural indexed geometry, a reverse-Z depth buffer, fixed semantic scene objects,
main-thread camera control, semantic object inspection, and spatial state in frame
manifests.

It excludes assets, model import, ECS, object-ID render targets, pixel picking, textures,
PBR materials, shadows, animation, GPU-driven culling, Render Graph, resize handling, and
general scene authoring.

## Workload

The canonical workload starts the workbench through installed stable Sidecar and checks
the semantic object registry. It then captures:

1. The default camera twice in one process.
2. One alternate camera pose in the same process.
3. The default camera again after a Sidecar restart.

Capture IDs and camera poses are fixed. Generated PNG and JSON artifacts overwrite the
ignored `out/captures/0003-spatial-calibration-scene/` collection; each frame manifest
identifies the Experiment 0003 scene and camera state.

## Controlled variables

- Right-handed world coordinates.
- `+X` points right, `+Y` points up, and local camera forward is `-Z`.
- World units are meters.
- Column-vector transform semantics: `clip = projection * view * model * position`.
- D3D normalized depth is `[0, 1]` with infinite reverse-Z projection.
- Depth clears to `0.0`; opaque depth comparison is `GREATER`.
- Default camera uses the versioned pose declared by the scene module.
- Alternate camera position is `[-9.0, 5.0, 10.0]` and target is `[0.0, 1.0, -3.0]`.
- Vertical field of view is 60 degrees and near plane is 0.1 meters.
- Window, adapter, D3D12, Agility SDK, debug layer, VSync, and Sidecar controls remain as
  fixed by Experiments 0001 and 0002.

## Metrics

- Raw pixel and PNG SHA-256 for every controlled camera state.
- Pixels differing from the clear-color reference pixel.
- Camera pose, projection parameters, coordinate convention, and scene revision embedded
  in every frame manifest.
- Stable semantic object IDs, names, transforms, bounds classes, and display colors.
- Process IDs before and after Sidecar restart.
- GPU submission/readback, row copy, hash, encode, and artifact preparation timing.
- Renderer last error, device removal state, and final Sidecar process counts.

## Acceptance criteria

- The shader is compiled by pinned DXC and the workbench creates a D3D12 graphics PSO
  with depth testing enabled.
- The rendered frame contains a ground reference, positive world axes, and at least three
  spatial marker objects with stable semantic IDs.
- `camera.status`, `camera.set_pose`, and `camera.reset` operate through typed Sidecar
  events and all camera mutation is applied on the window/render thread.
- `scene.list_objects` reports exactly the versioned calibration objects used for draws.
- Frame manifests report the agreed handedness, axes, units, transform convention,
  reverse-Z state, camera pose, and scene revision.
- The two same-process default-camera captures have identical raw pixel and PNG hashes.
- The alternate camera capture differs from the default capture in both raw pixel and PNG
  hashes.
- The default capture contains at least 100,000 pixels different from the clear-color
  reference pixel.
- The default camera after Sidecar restart has the same raw pixel and PNG hashes as before
  restart.
- Every manifest reports no device removal and no last renderer error.
- Final Sidecar status contains no target or broker process, and the repository guard
  passes.

## Environment

Frame manifests and the final acceptance report record the actual revision, process,
camera, scene, renderer, image, hash, and timing evidence.

## Reproduction

Run from the repository root with Sidecar 0.5.1 or newer installed from stable:

```powershell
runseal :spatial-scene
```

## Results

Accepted results on 2026-07-12:

| Capture | Process | Frame | Non-background pixels | Pixel SHA-256 prefix | PNG SHA-256 prefix | GPU/readback |
| --- | ---: | ---: | ---: | --- | --- | ---: |
| Default 1 | 11996 | 11 | 408,439 | `8f0fc6e9a49b` | `8e4ccc799793` | 1.48 ms |
| Default 2 | 11996 | 12 | 408,439 | `8f0fc6e9a49b` | `8e4ccc799793` | 1.38 ms |
| Alternate | 11996 | 13 | 469,465 | `475d6712c585` | `56865cb3439a` | 1.43 ms |
| Default restart | 21856 | 12 | 408,439 | `8f0fc6e9a49b` | `8e4ccc799793` | 1.51 ms |

The typed registry reported all eight expected object IDs and names. Every manifest
reported `calibration-v1`, the accepted coordinate and reverse-Z contract, the exact
camera pose, no device removal, and no renderer error. Final Sidecar status contained no
target or broker process. Manual inspection confirmed that both camera views contain the
expected checker ground, positive axes, colored markers, and occlusion relationships.

## Conclusion

Accepted. The workbench now provides a deterministic spatial visual vocabulary shared by
the renderer, Sidecar controls, semantic inspection, and frame evidence. Stable wrong
output remains possible in principle, so visual inspection complements rather than
replaces the hash and manifest checks.

## Promotion

ADR 0006 promotes the spatial and depth conventions to repository policy. Rendering code
remains workbench-owned until a later load experiment establishes a reusable engine
boundary. The next experiment may add bounded region rendering and an object-ID
perception path on this accepted scene.
