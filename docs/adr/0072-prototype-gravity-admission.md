# ADR 0072: Prototype Gravity Admission

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0069 Prototype Gravity Admission

## Context

The prototype already owns a grounded capacity-one runtime actor, admits bounded host elapsed only
on Ready samples, and submits one transactional schedule/actor advance before each frame. That
advance still supplied zero vertical acceleration. Connecting horizontal input first would expose
the accepted downhill-separation behavior without any force returning the actor to terrain, or
would require input, gravity, and locomotion policy to be accepted together.

The engine already proves fixed vertical integration, exact contact, planar-first ordering,
bounded batches, and complete actor/schedule rollback. The missing boundary is application policy,
not another motion implementation.

## Decision

- The non-diagnostic prototype owns one fixed gravity step acceleration of `-179` Q16 units at the
  accepted 60 Hz simulation rate. This is the nearest integer per-step encoding of `-9.81 m/s²`.
- Every Ready-admitted schedule/actor transaction supplies that value. Reset, Suspended, and
  Stalled host outcomes continue to admit no simulation work.
- Planar deltas and step-up limit remain zero. The prototype adds no horizontal input mapping,
  jump, camera, or animation policy.
- `engine-runtime` remains the sole owner of integration, terrain queries/contact, batching,
  rollback, actor identity, presentation, and renderer admission. No gravity configuration or
  application policy is added to its public surface.
- `runseal :canonical-prototype` is the focused maintained gate for prototype tests, minimal fresh
  sources, bootstrap failures, direct restart equality, and Sidecar lifecycle. It complements the
  end-to-end canonical workflow and does not invoke it recursively.

## Consequences

- A touching prototype actor executes real downward acceleration on every due step while exact
  terrain contact returns it to the same grounded body with zero retained vertical velocity.
- Gravity is now a concrete Prototype v0 policy and a prerequisite for later horizontal movement;
  its value is intentionally not a general engine setting.
- Input mapping, step-up tuning, downhill locomotion evidence, camera following, jumping, and
  multiple actors remain independent future decisions.
- This change adds no engine API, GPU resource, renderer pass, copy, barrier, fence, signal, wait,
  compatibility path, or diagnostic prototype surface.

## Evidence

Experiment 0069 records the exact constant, focused tests, fresh-source process evidence, startup
rejections, normalized restart equality, and complete Sidecar cleanup. The focused workflow passed
in 32.032 seconds.
