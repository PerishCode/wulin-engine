# ADR 0006: Spatial and Depth Convention

- Status: Accepted
- Date: 2026-07-12
- Supersedes: None
- Superseded by: None

## Context

Rendering, visibility, animation, physics, import, and semantic inspection need one
shared interpretation of axes, units, transform order, and depth. Leaving those choices
implicit would make later load experiments incomparable and asset conversion ambiguous.

Experiment 0003 rendered and inspected a fixed scene under one convention, changed its
camera through Sidecar, and reproduced identical default-camera pixels after restart.

## Decision

- World space is right-handed: `+X` right, `+Y` up, and camera-local forward `-Z`.
- One world unit is one meter.
- Math uses column vectors with `clip = projection * view * model * position`.
- D3D normalized device depth is `[0, 1]`.
- Perspective projection uses an infinite far plane and reverse-Z.
- Depth clears to `0.0`; opaque geometry writes depth and compares with `GREATER`.
- Camera and semantic scene state used for evidence are reported in frame manifests.

## Consequences

New runtime, shader, asset, and tool code must convert into this convention at ownership
boundaries. Legacy or source-format coordinates stay isolated in import code. A future
change requires a superseding ADR and migration evidence, not a local matrix workaround.

## Evidence

- [Experiment 0003](../../experiments/0003-spatial-calibration-scene/README.md) records
  deterministic default and alternate-camera frame artifacts, semantic object identity,
  and reverse-Z runtime state.
