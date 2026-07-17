# Experiment 0148: Native Diagonal Walk Stop

Status: Accepted

## Hypothesis

The existing diagonal-Walk process can prove a complete multi-key locomotion lifetime without
another child or product output. Atomic W/A should produce exact diagonal Walk, delayed W release
should retain exact left Walk, and delayed A release should transition to Survey while retaining
the last admitted left yaw. Because the final phase is stationary, the accepted two-phase movement
must remain exactly recoverable from final position.

## Scope

- Extend the existing post-readiness diagonal-Walk sequence with delayed A-up after its accepted
  delayed W-up.
- Hold the no-locomotion state for another bounded interval before the existing Escape completion.
- Preserve the atomic W/A prefix and exact diagonal/left displacement decomposition.
- Require final Survey clip 0/yaw 32,768, clock continuity, idle object policy, zero render blocks,
  and the existing two-value graceful completion.
- Advance the complete Prototype workflow revision from v62 to v63.

Product input/locomotion/presentation policy, session schema/output, process count, bootstrap,
Runtime, renderer/GPU resources, source formats, synchronization, and resource cleanup are out of
scope.

## Workload

1. Launch the existing diagonal-Walk child to grounded stationary readiness.
2. Atomically queue W-down and A-down on its exact visible window thread.
3. Hold diagonal Walk for at least 250 ms, then post W-up while retaining A.
4. Hold left Walk for at least 250 ms, then post A-up.
5. Hold the resulting stationary state for at least 250 ms, post Escape, and consume the existing
   completion value.
6. Derive both exact positive movement step counts from final X/Z and validate final Survey,
   retained yaw, actor/clock/frame/object state, transport timing, and cleanup.

## Controlled Variables

- Camera orbit remains zero; local W/A maps directly to negative world Z/X.
- Diagonal and cardinal Walk components remain fixed at 23 and 32 Q9.
- Initial W/A is one atomic prefix; W-up, A-up, and Escape use the maintained monotonic delayed
  transport.
- No locomotion key remains held after A-up. Stationary policy retains the final admitted yaw and
  contributes no displacement.
- No intermediate product output, position polling, inspect path, retry, threshold relaxation,
  telemetry, extra child, event history, or copied state is added.

## Metrics

- PID/window/thread identity; W/A atomic span and interval; diagonal, retained-left, and stationary
  hold intervals; final X/Z; derived diagonal/left step counts; actor identity/region/body/
  presentation/epoch; clock/frame/object state; render blocks; exit code/output count; workflow
  duration/report bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- W/A must be posted atomically after readiness on one exact visible process window thread with a
  0..=50 ms interval/span.
- W-up must follow A-down by 250..=750 ms, A-up must follow W-up by 250..=750 ms, and Escape must
  follow A-up by 250..=750 ms.
- Final Z must be a negative multiple of 23. `deltaZ-deltaX` must be a positive multiple of 32.
  Both derived movement step counts must be in `1..=512`.
- Actor handle, region, half-height, grounding, and zero vertical velocity must remain valid.
- Final presentation must be Survey clip 0/yaw 32,768 with a later epoch than readiness.
- Clock Ready/sample counts must advance without reset/suspend/resume/stall changes; render blocks
  and object activity must remain zero.
- Exit code must be zero with exactly readiness plus completion, and no product/Runtime/GPU/source/
  resource/process-count diff may exist.

## Results

`canonical-prototype-v63` passed on its first run in 163.081 seconds with a 455,824-byte report.
PID 16472 queued atomic W/A on window thread 23708 with a 0.0013 ms key interval and atomic span.
W-up followed 265.2565 ms later, A-up followed after another 261.3792 ms, and Escape followed A-up
after 259.2750 ms.

The actor retained its generation/region/shape, remained grounded with zero vertical velocity, and
finished at local `(-857,-345)` Q9. The final displacement decomposes uniquely into 15
`(-23,-23)` diagonal Walk steps plus 16 `(-32,0)` retained-left Walk steps. The stationary phase
added no movement. Final presentation was Survey clip 0/yaw 32,768; animation epoch advanced from
1 to 79.

Clock Ready/sample advanced from `2/3` to `93/94` with unchanged reset/suspend/resume/stall counts.
Render blocks remained zero, object observation/interaction remained idle, and exit code zero
contained exactly two values with empty stderr. All 103 engine-runtime, 48 Prototype, and 20
reference-host tests passed; Flavor retained zero denies and five existing warnings. No product,
Runtime, renderer/GPU, source, synchronization, resource, or process-count owner changed.

## Conclusion

Accepted. One existing real process now proves a complete native diagonal-Walk key lifetime:
diagonal movement transitions to retained-left movement and then to stationary Survey after the
last direction release, while exact movement, retained facing, and cleanup remain observable from
the canonical final state without a new acceptance or product surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
