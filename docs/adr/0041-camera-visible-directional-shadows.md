# ADR 0041: Camera-Visible Directional Shadows

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

The canonical surface path had deterministic directional diffuse lighting but no shadow
visibility. The accepted renderer already owns one GPU camera-visible list, camera-selected LOD,
arbitrary-Q8 ground results, skin bindings, and evaluated pose palette. Creating a separate CPU
draw list, light culler, animation path, or object authority to add shadows would duplicate live
state before that complexity had experimental evidence.

Experiment 0038 tests the narrow reusable boundary: one fixed hard-shadow map generated from the
existing pre-occlusion camera-visible object stream and consumed only by object surface resolve.

## Decision

- The canonical surface renderer owns one immutable 1,024 by 1,024 `D32_FLOAT` shadow resource,
  DSV, SRV, and probe readback. They are allocated once with the renderer.
- One fixed finite orthographic matrix uses the normalized `[-0.45, 0.8, 0.3]` light direction and
  covers the bounded local canonical window. Renderer creation rejects a projection that does not
  contain that window.
- After GPU pose evaluation and before occlusion compaction, one depth-only indirect mesh pass
  consumes the existing camera-visible list and its existing count. It reuses camera LOD,
  arbitrary-Q8 ground, yaw, catalog geometry, skin bindings, and palette without reculling,
  reevaluating, or copying them.
- Surface resolve reconstructs the visible triangle world position, addresses one nearest shadow
  texel, and classifies it with a fixed `0.0015` receiver-depth bias. Shadow removes the direct
  diffuse and metallic contribution but preserves ambient light.
- The CPU shade oracle uses the same position, light matrix, texel mapping, captured depth, bias,
  and lighting rule. Probe evidence includes depth hash/occupancy, caster and dispatch counts,
  per-sample decisions, and timing.
- The surface root signature uses 60 constant DWORDs and one descriptor table, for a 61-DWORD
  D3D12 cost. The heap has 98 descriptors. Canonical frames have six fixed skeletal/surface
  submissions, including exactly one shadow dispatch.

## Consequences

- Camera-visible animated objects cast and receive deterministic hard shadows with no CPU draw
  list, new presentation field, content copy, alternate LOD, second pose evaluation, or variable
  resource allocation.
- Shadow depth participates in exact held-frame, source-order, revisit, camera-relative alias, and
  source-duration loop evidence. Animation changes caster silhouettes through the accepted palette.
- The caster boundary is deliberately camera visibility, not light visibility. Off-camera casters,
  terrain casting/receiving, a light-space culler, shadow LOD, cascades, filtering, soft shadows,
  alpha testing, dynamic lights, and gameplay light authority are not defined by this decision.

## Evidence

Experiment 0038 passed the direct 562-second workflow. The controlled 10,538-caster map contained
88,557 occupied texels; all six sampled receivers matched the CPU oracle exactly, with one
shadowed and five lit. Walk frames 43 and 85 reproduced frame 0 shadow and final attachments
exactly, while frame 42 changed them.

The 64-publication run held peak handles at the 531 baseline and ended at 516 handles with private
bytes below baseline. All source/failure/rollback gates, 32 reactive and 32 prepared crossings,
and 16 lifecycle cycles passed. Generated evidence is ignored under
`out/captures/0038-camera-visible-directional-shadows/`.
