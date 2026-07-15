# Experiment 0063: Live Prototype Time Driver

Status: Accepted

## Hypothesis

The non-diagnostic prototype can consume the accepted activation-aware HostClock and drive the sole
Runtime schedule/body transaction in one explicit loop order, emitting readiness only after a
nonzero simulation commit and the following successful frame, without introducing gravity,
locomotion, camera, actor, workbench, inspect, or alternate runtime policy.

## Scope

After each message pump, ingest normalized input and honor Escape first. Then drain one complete
activation batch, sample HostClock once, and invoke `Runtime::advance_simulation_body` only for a
`Ready` sample. Use the retained prototype handle with zero planar displacement, zero step-up limit,
and zero acceleration. Render one frame after the optional successful simulation commit.

Treat `Reset`, `Suspended`, and `Stalled` as explicit no-advance outcomes. Retain HostClock's
post-stall baseline recovery and make Runtime advance/frame failure terminal. Move the single
prototype readiness line to the first nonzero committed tick after its following frame, including
the exact sample, clock status, dual result, zero command, initial body, and frame count.

Do not add gravity, horizontal motion, input mapping, camera behavior, actor/presentation binding,
an inspect endpoint, workbench driving, backlog/catch-up, clamping, or a second time authority.

## Workload

1. Prove the prototype time policy admits exact elapsed only for `Ready`; reset, suspended, and
   stalled outcomes never produce a Runtime request.
2. Start a real prototype over fresh cooked sources and require no readiness until canonical
   publication, grounded body spawn, at least one emitted simulation tick, and the next frame.
3. Require the first live advance to use exact bounded elapsed, emit 1..=8 steps from tick zero,
   perform one terrain query per step, and preserve the generation-one touching body byte-exactly.
4. Require clock status to contain a baseline, at least one reset and ready sample, no suspension at
   readiness, and counters consistent with the published sample.
5. Restart the direct process and require distinct PIDs plus equal structural body/command/driver
   invariants; elapsed and rational remainder may vary with real wall time.
6. Preserve invalid config, missing source, corrupt payload, Sidecar start/restart/stop, terminal
   no-readiness failures, and final empty PID state in the existing targeted prototype gate.
7. Run focused prototype/reference-host/runtime tests, the targeted prototype process gate,
   `runseal :init`, and `runseal :guard`. Do not run the full canonical workflow because no
   workbench renderer/resource/synchronization authority changes.

## Controlled Variables

- Existing schema-1 content target, initial retained body, 60 Hz rational schedule, 125 ms elapsed
  bound, and 0..=8 batch bound remain unchanged.
- Every live command is exactly `(deltaXQ9=0, deltaZQ9=0, stepUpLimitQ16=0,
  stepAccelerationQ16=0)`.
- Simulation precedes its frame; presentation remains successful-frame-owned and independent.
- No sample is clamped, split, queued, replayed, or converted from reset/suspended/stalled to ready.

## Metrics

- Host sample kind/value and complete clock counters.
- Simulation start/end tick, step count, exact remainder, and elapsed nanoseconds.
- Retained generation, input/output motion equality, terrain query count, and zero command.
- Startup/live frame counts, readiness ordering, process identity, and lifecycle cleanup.
- Focused test count, targeted gate duration/result, Flavor denies, and guard result.

## Acceptance Criteria

- Only exact bounded Ready samples reach Runtime; all other typed clock outcomes do no simulation
  work and retain their explicit classification.
- Readiness proves a nonzero dual commit followed by a successful frame, never merely bootstrap.
- The zero command advances schedule while retaining body motion and one-query-per-step behavior.
- Direct restarts preserve structural invariants with fresh process identity and no time-value
  equality assumption.
- Existing startup failure and Sidecar lifecycle gates remain exact with no residual process.
- Focused tests, targeted process gate, `runseal :init`, and `runseal :guard` pass without the full
  canonical workflow.

## Reference Environment

The experiment uses the pinned Windows reference platform, fresh canonical cooked fixture sources,
the non-diagnostic prototype Sidecar, composed HostClock/activation transport, and Runtime's sole
retained schedule/body transaction.

## Evidence

Three focused prototype integration tests pass: two retain exact body grounding/overflow evidence,
and one proves only `Ready` admits exact zero and maximum bounded elapsed while reset, suspended,
and stalled outcomes admit nothing. Prototype clippy, unchanged workbench compilation, and the
TypeScript gate check pass.

A 50.75-second fresh-cook targeted prototype gate covered invalid configuration, missing source,
corrupt payload, two direct launches, and Sidecar start/restart/stop. Both direct processes reached
readiness only with a bounded Ready sample, an active clock with reset/ready history, a nonzero 1..=8
step advance from tick zero, one terrain query per step, byte-identical generation-one body
input/output, an all-zero command, and at least one following live frame. Real elapsed/remainder
values were deliberately not required to match; normalized structural driver invariants did.

All three failures emitted no readiness, direct PIDs differed, and final Sidecar status retained no
PID. `runseal :init` and `runseal :guard` passed with zero Flavor denies. The full canonical workflow
was not run because the targeted gate directly exercises the changed prototype frame/lifecycle order
while workbench renderer, GPU resources, traversal, synchronization, and canonical evidence remain
unchanged.
