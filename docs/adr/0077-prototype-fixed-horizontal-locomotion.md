# ADR 0077: Prototype Fixed Horizontal Locomotion

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0074 Prototype Horizontal Locomotion

## Context

The prototype already consumes normalized host input for Escape, drives one authoritative actor
from admitted host time, handles typed render backpressure without retry, and anchors the camera to
the retained actor before each frame. Horizontal command fields remained zero, so enabling
camera-driven composition traversal would not yet prove a meaningful application target change.

The first locomotion policy must establish an input-to-command boundary without introducing
frame-rate scaling, accumulated velocity, traversal publication, configurable bindings, or a
second engine movement path.

## Decision

- The prototype owns a fixed W/A/S/D held-state mapping over the reference host's existing
  normalized input. W/S produce negative/positive Z and A/D produce negative/positive X; opposing
  directions cancel per axis.
- One cardinal fixed step is 32 Q9 units. A diagonal uses the fixed nearest normalized component
  23 Q9 on each active axis. Every command carries a 32,768 Q16 step-up limit and the existing -179
  Q16 gravity acceleration.
- The prototype reduces input once after native message ingestion and before host-time sampling.
  Every fixed step in one admitted batch consumes that immutable command through the sole atomic
  simulation/actor transaction.
- Composition traversal remains disabled. Actor-relative camera anchoring consumes the committed
  actor exactly as before, so horizontal and terrain-following displacement move the camera without
  a parallel camera policy.
- `runseal :canonical-prototype` owns a maintained Windows input support module that locates the
  exact prototype class/title, verifies the process identity, and posts a native W key-down. The
  prototype gains no inspect route, test mode, command-line override, or support-module dependency.

## Consequences

- Prototype v0 now has a small deterministic input-command-actor-camera loop, but no horizontal
  velocity, acceleration, turn/yaw policy, animation selection, jumping, configurable controls, or
  cross-window traversal policy.
- A multi-step elapsed batch repeats one sampled displacement. This is deliberate fixed-step
  policy, not frame-time-scaled movement.
- The nonzero step-up limit permits bounded uphill motion through the existing exact terrain
  transaction; terrain contact, rollback, actor identity, and GPU presentation remain engine-owned.
- Traversal can be evaluated separately once movement can produce a real publication target.
- No engine API, renderer algorithm, shader, GPU resource, copy, barrier, fence, wait, source
  format, or compatibility path changes.

## Evidence

Experiment 0074 records complete policy tests and a real-process native W workload. The focused
workflow passed in 38.742 seconds: one fixed step produced command `(0, -32)`, actor Z `0 -> -32`
Q9, one terrain query, zero render blocks, and camera Z `12 -> 11.9375` / `-3 -> -3.0625`; ordinary
processes remained stationary and restart-identical. Failure and Sidecar cleanup gates also passed.
