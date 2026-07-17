# Experiment 0132: Native Forward Release

Status: Accepted

## Hypothesis

The redundant idle Escape process can become a stronger normal locomotion-release gate without
adding a process. After idle readiness, an exact-PID native W press held for at least 250 ms,
followed by W release and at least 250 ms of stationary work before Escape, should commit exact
Walk displacement and finish as Survey with the last admitted forward yaw.

## Scope

- Replace the plain idle Escape session/report field with one post-readiness forward-release
  session that still exits through Escape and publishes exactly two values.
- Atomically post the initial W-down to the exact visible process window thread, then post W-up
  after a monotonic 250 ms lower bound and Escape after another 250 ms lower bound.
- Validate exact native messages, movement, final presentation, actor identity, camera baseline,
  clock continuity, object-idle state, and zero render blocks.
- Add a focused session oracle and guard against restoration of the plain Escape report shape.

Product input normalization, locomotion/presentation policy, session schema, Runtime,
renderer/GPU resources, source formats, synchronization, and process count are out of scope.

## Workload

1. Audit every live locomotion session for normal direction-key release evidence.
2. Reuse the existing idle Escape process rather than launching another child.
3. Start from idle Survey readiness, atomically post W-down, hold it for 250 ms, post W-up, wait
   another 250 ms, and post Escape from the same helper to the same PID/window.
4. Require negative-Z displacement in exact 32-Q9 units, no X/region/handle divergence, final
   Survey clip 0 with retained W yaw 49,152, a later animation epoch, continuous clock counters,
   idle object state, zero render blocks, and exact two-value completion.
5. Run all existing Prototype sessions, Sidecar lifecycle, static guard, and initialization gates.

## Controlled Variables

- Initial actor/camera/object state, timing helper schema, fixed Walk command, gravity, boundary,
  session completion, and Escape ownership remain unchanged.
- The process count is unchanged; only the old idle Escape process receives current input work.
- Existing Run modifier release/re-press, focus cleanup, diagonal, camera, Jump, object, and
  finite-boundary sessions retain their exact current evidence.
- No intermediate product output, retry, dynamic expected presentation, relaxed delay, input
  history, event stream, fallback, or compatibility alias is admitted.

## Metrics

- Exact process/window/thread identity, ordered messages, atomic prefix, W hold and post-release
  stationary intervals, actor position/region/handle/body, animation epoch, presentation clip/yaw,
  camera orbit, clock counters, frame/render-block counts, object state, output count, workflow
  duration, report bytes, Flavor findings, and process cleanup.

## Acceptance Criteria

- The action begins only after readiness and targets that exact PID and visible window.
- Native evidence is exactly focus, W-down, W-up, Escape; W hold and stationary dwell are each in
  `[250,750]` ms, and the initial single-key atomic prefix uses one positive window thread.
- Completion moves only negative Z by 1..=512 exact 32-Q9 Walk steps, retains actor handle and
  region, has zero vertical velocity, and finishes as Survey clip 0 with yaw 49,152.
- Animation epoch advances, the clock has no suspend/resume/reset/stall discontinuity, render
  blocks remain zero, object state remains idle, and output remains exactly readiness plus
  completion.
- The old `escape`/`escapeInvariant` report shape, a new process, and product/Runtime/GPU/source/
  resource/synchronization changes do not return.

## Results

The audit found no normal locomotion-direction release in maintained process evidence. Run release
keeps W held, opposite-locomotion release keeps W held, and focus loss clears input through a
discontinuity rather than `WM_KEYUP:W`. The plain idle Escape process carried only session framing
already exercised by every graceful session, so it was replaced directly.

The first v47 full run rejected the initial final-presentation oracle. It expected Survey yaw 0,
but the accepted product contract retains the last facing only after a nonzero committed advance:
W owns yaw 49,152. The oracle was corrected from product tests and Experiment 0078; product code,
delays, and thresholds were not changed.

Final `canonical-prototype-v47` passed in 144.561 seconds. PID 20436 received W-down/W-up/Escape on
the exact visible window, with the initial press atomically posted on thread 10864. W remained held
for 255.9837 ms and the process ran for another 252.1398 ms after release before Escape. The actor
moved `(0,-480)` Q9, exactly 15 Walk steps, retained its region and generation, and finished with
zero vertical velocity as Survey clip 0/yaw 49,152. Its animation epoch advanced from 1 to 609.

Clock ready/sample counts advanced from 4/5 to 758/759 with reset count 1 unchanged, no
suspend/resume/stall, and zero render blocks across 759 live frames. Object state remained idle,
completion used the same PID/handle, and stdout remained exactly two values. The ignored report was
446,287 bytes. Runtime was 0.248 seconds slower and the report 2,468 bytes larger than v46, while
the child-process count remained unchanged.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU, source,
resource, or synchronization code changed.

## Conclusion

Accepted. Normal native W release now has an exact post-readiness product completion witness:
committed Walk movement transitions back to Survey while retaining the committed forward facing
and all actor/clock/object/session invariants. The gate replaces rather than supplements the old
idle Escape process.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
