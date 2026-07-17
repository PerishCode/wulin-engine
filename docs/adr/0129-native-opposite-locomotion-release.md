# ADR 0129: Native Opposite Locomotion Release

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0126 Native Opposite Locomotion Release

## Context

Prototype unit evidence covered W/S cancellation, but no real native session proved both keys
entered one ingest or that cancellation preserved held state for later release readmission. Adding
the session also exposed two acceptance transport races: Windows sleep could return below its
declared lower bound, and a warm Prototype could publish readiness while a newly spawned
PowerShell startup helper was still compiling.

## Decision

- Add one atomic exact-window Shift/W/S startup batch, stationary Survey readiness, post-readiness
  S release, and exact retained Shift/W Run completion.
- Require zero-origin Survey/yaw-zero readiness and nonzero 64-Q9 negative-Z Run/clip-2/yaw-49,152
  completion with a later epoch.
- Implement delayed Escape against a monotonic Stopwatch deadline rather than weakening the
  requested interval.
- Start and prepare startup-input helpers before spawning their Prototype child. Let each helper
  select the next unique class/title window, return the actual PID, and require it to match the
  child.
- Keep post-readiness input process-qualified and retain every existing actor, camera, clock,
  object-idle, and two-value session invariant.

## Consequences

- W/S cancellation and retained-input release now have exact real-process evidence.
- Warm startup input no longer races readiness, and delayed native sequences cannot report less
  than their requested lower bound.
- Acceptance gains no retry, dynamic result set, product delay, threshold relaxation, journal,
  event history, or new output value.
- Product input/locomotion/presentation, Runtime, renderer/GPU resources, synchronization, and
  object policy remain unchanged.

## Evidence

`canonical-prototype-v41` passes in 147.265 seconds after the transport fixes. PID 6140 queues
Shift/W/S on window thread 13,656 in a 0.0022 ms atomic span. Readiness is local `(0,0)`,
Survey/yaw 0/epoch 1. S-up followed by Escape at 209.9403 ms produces local `(0,-832)`, 13 exact
Run steps, clip 2/yaw 49,152/epoch 27, clock `1/2 -> 32/33`, zero render blocks, idle object state,
and exactly two output values. `runseal :guard` passes with zero Flavor denies; all previous
Prototype gates pass.
