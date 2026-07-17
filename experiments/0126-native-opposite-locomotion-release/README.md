# Experiment 0126: Native Opposite Locomotion Release

Status: Accepted

## Hypothesis

One atomic native Shift/W/S startup batch should produce stationary Survey readiness because the
opposed W/S axis cancels before locomotion and presentation selection. Releasing only S afterward
should preserve held Shift/W and commit exact negative-Z Run movement without product changes.

## Scope

- Add one exact visible-window atomic Shift/W/S startup action.
- Require readiness at local `(0,0)` with Survey clip 0, yaw 0, and default camera orbit.
- After readiness, post only S-up and a bounded delayed Escape.
- Require retained Shift/W to produce 64-Q9 negative-Z Run steps, clip 2, yaw 49,152, and a later
  animation epoch.
- Diagnose and remove acceptance-only startup-input and delayed-Escape timing races exposed by the
  expanded real-process matrix.

Product `HostInput`, locomotion, presentation, clock, session schema, Runtime, renderer/GPU,
resource, synchronization, and object policy are out of scope.

## Workload

1. Inventory unit-only opposing-axis branches and native session coverage.
2. Queue Shift/W/S against one suspended exact window thread and resume before product execution.
3. Capture stationary readiness, release S, and capture graceful completion from the same process.
4. Preserve all previous sessions while requiring exact actor, camera, clock, object-idle, and
   two-value completion invariants.
5. Run `runseal :guard`, `runseal :canonical-prototype`, and `runseal :init`.

## Controlled Variables

- Startup input is exactly Shift-down, W-down, and S-down; it contains no camera, action, or exit
  key.
- Post-readiness input is exactly S-up followed by Escape at least 200 ms later.
- Orbit stays zero, so retained W maps only to negative world Z.
- Stationary readiness must remain at the initial local origin and retain Survey/yaw zero.
- Completion displacement must be a nonzero integer multiple of 64 Q9 with zero X movement.
- No discontinuity, render block, object intent, action, target, consumption, or copied state is
  admitted.

## Metrics

- Exact PID/window/thread, message order, atomic span and per-key intervals; delayed-Escape actual
  interval; readiness/final actor position, clip, yaw, epoch, clock, frame count, render blocks,
  object state, output count, workflow duration, Flavor findings, and generated report inventory.

## Acceptance Criteria

- The startup helper is prepared before child launch, selects the next exact class/title window,
  and returns the same PID as the spawned child.
- Shift/W/S are queued on one suspended window thread within a finite span no greater than 50 ms.
- Readiness is exactly local `(0,0)`, Survey clip 0, yaw 0, with no horizontal movement.
- S-up preserves Shift/W and completion is exact negative-Z Run, clip 2, yaw 49,152, with a later
  epoch.
- The monotonic delayed-Escape measurement is at least the requested 200 ms without threshold
  relaxation.
- Every previous Prototype gate, `runseal :guard`, and `runseal :init` passes.

## Results

The initial full run exposed a shared delayed-Escape transport defect before the product oracle:
PowerShell requested a 200 ms sleep but Stopwatch measured only 197.0125 ms. The acceptance helper
now waits to a monotonic Stopwatch deadline and posts Escape only after the complete requested
interval; the 200 ms lower bound was not changed.

Two later full runs failed at different existing startup-input gates: held-camera readiness lost
orbit one once, object action missed completion once, and a later Jump readiness expected impulse
4,369 but received the zero-input Survey command. This established a common warm-process race:
each Prototype could publish readiness while a newly spawned PowerShell helper was still compiling.
The final transport starts and prepares the helper before child launch, waits for the next unique
class/title window, returns its actual PID, and requires that PID to equal the child. It adds no
retry, product delay, or relaxed oracle. Camera and object failures now also report exact expected
and actual state.

The first full run after that fix passed as `canonical-prototype-v41` in 147.265 seconds. The new
session used PID 6140; both startup and release helpers returned PID 6140. Window thread 13,656
queued `WM_SETFOCUS`, Shift-down, W-down, and S-down atomically in 0.0022 ms, with key intervals
0.0012 and 0.0010 ms.

Readiness stayed at local `(0,0)` with Survey clip 0, yaw 0, and epoch 1. The post-readiness helper
queued S-up and posted Escape 209.9403 ms later. Completion retained the actor/region at local
`(0,-832)`, exactly 13 Run steps, clip 2, yaw 49,152, and epoch 27. Clock ready/sample advanced
`1/2 -> 32/33`; reset/suspend/resume/stall remained `1/0/0/0`, render blocks stayed zero, object
observation/interaction remained idle, and output contained exactly readiness plus completion.

All previous startup, session, object, boundary, restart, failure, Sidecar, Rust, and Deno gates
passed. Flavor reported zero denies and five existing warnings. The generated acceptance inventory
is one JSON file / 479,873 bytes.

## Conclusion

Accepted. Real native evidence now proves opposed locomotion cancellation and release
readmission. Acceptance startup and delayed-exit transport enforce their declared ordering instead
of racing warm product readiness or weakening time bounds; product and engine behavior remain
unchanged.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
