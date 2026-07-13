# ADR 0021: Camera-Driven Region Traversal

- Status: Accepted
- Date: 2026-07-13
- Supersedes: None
- Superseded by: None

## Context

ADR 0018 accepted atomic terrain/object pair publication, and ADR 0020 accepted terrain
render LOD while object grounding remains exact. Moving the visible region still
required two independent operator actions: change the camera, then submit a matching
composition schedule. That sequence proves the data path but is not a map traversal
loop.

Allowing every camera event to append work would make request memory and stale I/O grow
with input rate. Cancelling a pair after I/O or copy begins would add a second
transaction problem before the existing atomic publication path has established a need
for it. Experiment 0018 tests a smaller bounded control policy.

## Decision

- Camera traversal is a workbench-owned CPU scalar policy around the accepted atomic
  composition coordinator. It adds no GPU pass, request-sized allocation, or dynamic
  command submission.
- A traversal session inherits immutable `worldRegionSide` and `activeRadius` from its
  initial published pair. Changing either requires traversal disable and an explicit
  pair publication.
- Camera XZ maps to 16 meter half-open region intervals using
  `floor((position + 1032) / 16)`. Exact boundaries belong to the positive region. The
  result is clamped to the legal center range for the inherited active window.
- The coordinator permits one in-flight terrain/object pair and one optional desired
  config. Camera changes replace that desired value; they never append to a request
  queue. An in-flight pair is allowed to finish, then at most one pair is scheduled for
  the latest desired center.
- Every frame renders one complete published pair. The old pair remains active until
  both replacement halves stage and commit under one pair token at a frame boundary.
- A synchronously or asynchronously failed desired config is exposed as blocked state.
  It is not retried every frame. A different desired center or explicit traversal
  re-enable clears the block.
- Manual composition scheduling and fixture changes require traversal disable.
  Disabling composition also disables traversal. Sidecar exposes typed traversal
  enable/disable controls and the bounded counters through composition status.

## Consequences

Camera movement now closes the accepted region-selection and atomic-publication loop
without operator scheduling. Input rate cannot grow transaction memory: the maximum
queued depth is one, and a held pair plus several camera changes produces only the held
publication and one latest-wins publication.

This decision accepts snapshot consistency, not latency hiding. During a fast teleport,
the complete old active window may be outside the new camera view until the replacement
publishes. There is no predictive prefetch, overlap policy, loading presentation,
velocity model, or service-time budget. Exact half-open ownership also has no hysteresis,
so repeated idle crossings around one boundary may still produce repeated valid pairs.

The existing 50-slot terrain and instance caches, fixed GPU submission, arbitrary-Q8
grounding, terrain LOD, and contact gate remain unchanged. The optional terrain-cooker
center list only creates deterministic experiment packs; format V1 and the default pack
remain compatible.

This decision does not accept multiple in-flight pairs, cancellation after I/O begins,
predictive streaming, a general streaming graph, floating origins, authored world
partition, collision, navigation, or server interest management.

## Evidence

- [Experiment 0018](../../experiments/0018-camera-driven-region-traversal/README.md)
  records exact boundaries, world clamps, an eight-step corridor, held-I/O coalescing,
  synchronous failure, disable/catch-up, teleport/revisit, restart, compatibility, and
  release timing evidence.
- While center 65 was held, camera centers 66, 67, and 68 produced two queued
  replacements, maximum queue depth one, and exactly two schedules/publications after
  release: the held center and final center 68.
- Missing terrain produced one automatic attempt, zero accepted schedules, and zero
  publications. The complete center-64 pair remained published without retry.
- Exact baseline ground and position hashes, logical revisit attachments, 25/25
  physical slot divergence, terrain LOD validity, and the 0.125 meter contact gate pass.

## Reproduction

```powershell
runseal :region-traversal
```

The command writes the ignored report to
`out/captures/0018-camera-driven-region-traversal/acceptance.json`.
