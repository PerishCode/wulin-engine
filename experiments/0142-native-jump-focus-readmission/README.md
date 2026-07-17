# Experiment 0142: Native Jump Focus Readmission

Status: Accepted

## Hypothesis

The existing Jump-readmission process can prove the complete held-Space focus lifetime without
another child or product output. After the first Jump lands while Space remains held, a duplicate
Space-down immediately before focus loss must not create another action; focus cleanup must end
that held lifetime, and a later Space up/down after recovery must start one exact second flight.

## Scope

- Retain the first grounded native Space press and the bounded wait beyond its complete flight.
- Atomically post duplicate Space-down plus `WM_KILLFOCUS` on the exact visible process window
  thread while Space remains held.
- Resume focus, then retain the existing Space-up/Space-down/delayed-Escape sequence.
- Require one exact focus suspend/resume/reset boundary and the existing exact second-flight
  trajectory.
- Advance the complete Prototype workflow revision from v56 to v57.

Jump policy, HostInput normalization, main-loop ordering, session schema/process count, Runtime,
renderer/GPU resources, source formats, synchronization, and lifecycle ownership are out of scope.

## Workload

1. Launch the existing Jump-readmission session from grounded idle readiness and press Space.
2. Wait at least 1,250 ms so the exact first flight lands while native Space remains held.
3. Atomically post another Space-down followed by focus loss on the same PID/window.
4. Observe suspended sampling, resume focus, and wait for the existing reset/recovery boundary.
5. Post Space-up/Space-down and Escape through the existing helper; require an exact second
   airborne trajectory with stable actor identity, position, shape, and presentation.

## Controlled Variables

- The first and second Jump commands, fixed 4,369-Q16 velocity delta, 179-Q16 step acceleration,
  landing wait, second-to-exit delay, actor source, camera, object-idle state, and two-value session
  framing remain unchanged.
- The Experiment 0141 focus-locomotion gate remains unchanged and owns stale pre-loss action/intent
  suppression plus fresh A admission.
- No new process, intermediate product output, pending-action history, event queue, retry,
  compatibility alias, telemetry, or product behavior is added.

## Metrics

- Exact PID/window/thread identity; ordered native messages; atomic cleanup span; first-to-second
  and second-to-exit timing; actor handle/position/body/velocity/height/presentation/epoch; exact
  second-flight step count; clock counters; frame/render-block counts; object state; output count;
  workflow duration/report bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- The first action must remain exact focus plus Space-down and must complete its first flight before
  the discontinuity sequence.
- Cleanup must target the same PID/window and atomically emit exact
  `WM_SETFOCUS/Space-down/WM_KILLFOCUS` on one positive thread with zero single-key batch span.
- Recovery must emit exact `WM_SETFOCUS`; the later sequence must remain exact
  `Space-up/Space-down/Escape` on the same window.
- Completion must lie on exactly one valid second 4,369/-179 flight of 1..=43 steps, retain actor
  handle/horizontal position/body shape/Survey presentation/animation epoch, and remain airborne.
- The clock must record exactly one suspend, one resume, and one post-resume reset with increased
  suspended/Ready samples, zero stalls/render blocks, idle object state, and exactly two outputs.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count diffs
  must remain empty.

## Results

`canonical-prototype-v57` passed on its first run in 170.997 seconds. The 445,067-byte report
recorded PID 26908. The first-to-second posting lower bound was 4,053.3303 ms, including the
1,250 ms landing dwell, native focus helpers, suspended dwell, and recovery dwell. Held-Space
cleanup ran atomically on window thread 23692 with zero single-key batch span. The later
Space-up/Space-down preceded Escape by 118.1957 ms.

The second flight completed at step 7 with exact velocity 3,116 Q16 and exact rise 25,571 Q16.
The actor retained generation 1, signed region `(1099511627776,-1099511627776)`, local position
`(0,0)` Q9, half-height 65,536, Survey clip 0/yaw 0, and animation epoch 1.

Clock suspend/resume/reset counts changed by exactly `+1/+1/+1`. Suspended samples advanced
`0 -> 84`, Ready/sample counts advanced `2/3 -> 299/385`, stall count remained zero, and all 385
live frames had zero render blocks. Object policies remained idle, exit code was zero, and stdout
contained exactly readiness plus completion.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard`
retained zero Flavor denies and five existing warnings. No product, Runtime, renderer/GPU, source,
resource, or synchronization file changed.

## Conclusion

Accepted. Focus cleanup now has an exact real-process held-Space lifetime witness: a duplicate held
press cannot repeat Jump, focus loss ends that key lifetime, and the later normalized press is
admitted as one exact second flight without new state or surfaces.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
