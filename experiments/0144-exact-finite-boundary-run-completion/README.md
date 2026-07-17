# Experiment 0144: Exact Finite-Boundary Run Completion

Status: Accepted

## Hypothesis

The sole 15-second finite-boundary process can expose an exact product endpoint without another
child or output type. Replacing evidence-owned termination with the existing graceful session
completion should prove that held native Shift/W advances only inside the one-region rectangle,
stops in the exact conservative maximum-batch band, returns to Survey with retained facing, and
exits cleanly through Escape.

## Scope

- Retain the existing exact-PID atomic Shift/W input and 15,000 ms monotonic hold.
- Post the standard Escape action to the same PID/window after the hold and consume the existing
  sequence-2 completion value.
- Replace the custom boundary spawn/read/kill path with the maintained graceful session owner.
- Require stable actor identity/region/shape, exact 64-Q9 Run quantization, the complete
  `[-4096,-3648]` final local-Z band, final Survey/yaw 49,152, continuous clock/frame progress,
  idle object policies, and zero stalls/render blocks.
- Delete `boundarySurvival` and `holdPrototypeBoundaryRun` without aliases.
- Advance the complete Prototype workflow revision from v58 to v59.

Product boundary/input/locomotion/presentation policy, bootstrap bounds, Runtime, renderer/GPU
resources, source formats, synchronization, session schema, and process count are out of scope.

## Workload

1. Launch the existing one-region boundary process to grounded idle readiness.
2. Atomically post Shift-down and W-down to the exact visible PID/window thread.
3. Hold both keys for at least 15,000 ms under the existing monotonic acceptance timer.
4. Post Escape to the same PID/window and read the standard graceful completion.
5. Require 57..=64 exact 64-Q9 Run steps from local Z zero, a final same-region local Z in
   `[-4096,-3648]`, zero local X, grounded stationary Survey with retained negative-Z yaw, and
   complete two-value process cleanup.

## Controlled Variables

- One-region playable rectangle, actor/source/camera/traversal state, Shift/W order, hold duration,
  boundary maximum-eight-step reduction, Run component, Escape behavior, and all other Prototype
  processes remain unchanged.
- The existing five product boundary tests remain the pure per-axis/overflow authority.
- The shared native helper retains its 1,000 ms maximum delayed-exit bound; the 15-second boundary
  hold remains externally timed and does not broaden that permission.
- No retry, intermediate product output, telemetry, position polling, relaxed endpoint, fallback,
  compatibility alias, or additional child is added.

## Metrics

- Exact PID/window/thread identity; native key/message order; atomic span; held duration; Escape
  window; actor handle/region/local position/body/velocity/presentation/epoch; committed Run-step
  count; clock/frame/object state; exit code/output count; workflow duration/report bytes; test
  counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- Shift/W must occur strictly after readiness on one exact visible PID/window thread with atomic
  prefix length two and a 0..=50 ms interval/span.
- Held duration must be at least 15,000 ms; Escape must target the same PID/window through the
  current schema-4 action and produce reason `escape`, exit code zero, and exactly two values.
- Final actor identity and region must match readiness. Local X must remain zero; local Z must be a
  64-Q9 multiple in inclusive `[-4096,-3648]`, proving exactly 57..=64 Run steps before the
  maximum-eight-step candidate is reduced.
- Final motion must be grounded/stationary with unchanged shape; presentation must be Survey clip
  0/yaw 49,152 with a later animation epoch.
- Ready/sample/live-frame counts must advance with unchanged reset/suspend/resume/stall counts,
  zero render blocks, zero object feedback/action state, and clean process teardown.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count diffs
  must remain empty.

## Results

`canonical-prototype-v59` passed in 175.188 seconds with a 454,446-byte report. PID 30632 received
atomic Shift/W on window thread 21468 with a 0.0019 ms interval/span. The monotonic hold lasted
15,014.8593 ms; Escape targeted the same window `43846842`, returned exit code zero, and emitted
exactly readiness plus completion.

The actor retained generation 1 and its signed region, committed exactly 57 Run steps, and stopped
at local `(0,-3648)` Q9 inside the accepted conservative boundary band. It was grounded with zero
vertical velocity and unchanged half-height; final presentation was Survey clip 0/yaw 49,152 with
animation epoch 110. The final clock recorded Ready/sample `1007/1008`, all 1,008 live frames had
zero render blocks, stall/suspend/resume counts remained zero, and object observation/interaction
remained idle.

The first implementation attempted to place the 15-second delay inside the shared native helper;
its maintained 1,000 ms permission rejected that design before the boundary process ran. The final
implementation preserved the limit. A later full run correctly exposed that the actor moves to the
finite edge rather than remaining byte-identical, leading to the exact 57..=64-step endpoint
contract. One subsequent full run hit the unchanged camera re-press host-stall gate before reaching
the boundary; that strict gate was not relaxed, and the final complete run passed.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard`
retained zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU,
source, resource, or synchronization file changed.

## Conclusion

Accepted. The finite-boundary real process now proves its exact observable endpoint and standard
graceful lifetime: native Run advances in 64-Q9 quanta, stops inside the mathematically exact
maximum-batch band, returns to retained-facing Survey, and publishes the existing completion value
without another process or surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
