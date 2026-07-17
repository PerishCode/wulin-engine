# Experiment 0146: Native Diagonal Walk Release

Status: Accepted

## Hypothesis

The existing native diagonal-Walk process can prove a second locomotion phase without another
child or product output. Atomic W/A should first produce exact `(-23,-23)` Q9 Walk steps; after a
bounded delayed W release, retained A should produce exact `(-32,0)` Q9 left-Walk steps and final
yaw 32,768. The two phases should be uniquely recoverable from the final displacement.

## Scope

- Extend the existing post-readiness diagonal-Walk native sequence with delayed W-up.
- Hold retained A for a second bounded interval before the existing Escape completion.
- Require the initial W/A prefix to remain atomic on the exact visible PID/window thread.
- Decompose final displacement exactly as
  `deltaZ=-23*diagonalSteps`, `deltaX=-23*diagonalSteps-32*leftSteps`.
- Require at least one step in both phases, final Walk clip 1/yaw 32,768, clock continuity, idle
  object policy, zero render blocks, and the existing two-value graceful completion.
- Advance the complete Prototype workflow revision from v60 to v61.

Product input/locomotion/presentation policy, session schema/output, process count, bootstrap,
Runtime, renderer/GPU resources, source formats, synchronization, and resource cleanup are out of
scope.

## Workload

1. Launch the existing diagonal-Walk child to grounded stationary readiness.
2. Atomically queue W-down and A-down on its exact visible window thread.
3. Hold diagonal Walk for at least 250 ms, then post W-up while retaining A.
4. Hold left Walk for at least 250 ms, post Escape, and consume the existing completion value.
5. Derive both exact positive step counts from final X/Z, then validate final presentation,
   actor/clock/frame/object state, transport timing, and process cleanup.

## Controlled Variables

- Camera orbit remains zero; local W/A maps directly to negative world Z/X.
- Diagonal and cardinal Walk components remain fixed at 23 and 32 Q9.
- Initial W/A is one atomic prefix; W-up and Escape use the maintained monotonic delayed transport.
- A remains held through completion; Escape owns normal process exit.
- No intermediate product output, inspect path, position polling, retry, threshold relaxation,
  telemetry, extra child, event history, or copied state is added.

## Metrics

- PID/window/thread identity; W/A atomic span and interval; diagonal and retained-left hold
  intervals; final X/Z; derived diagonal/left step counts; actor identity/region/body/presentation/
  epoch; clock/frame/object state; render blocks; exit code/output count; workflow duration/report
  bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- W/A must be posted atomically after readiness on one exact visible process window thread with
  0..=50 ms interval/span.
- W-up must follow A-down by 250..=750 ms; Escape must follow W-up by 250..=750 ms.
- Final Z must be a negative multiple of 23. `deltaZ-deltaX` must be a positive multiple of 32.
  Both derived step counts must be in `1..=512`.
- Actor handle, region, half-height, grounding, and zero vertical velocity must remain valid.
- Final presentation must be Walk clip 1/yaw 32,768 with a later epoch than stationary readiness.
- Clock Ready/sample counts must advance without reset/suspend/resume/stall changes; render blocks
  and object activity must remain zero.
- Exit code must be zero with exactly readiness plus completion, and no product/Runtime/GPU/source/
  resource/process-count diff may exist.

## Results

`canonical-prototype-v61` passed on its first run in 169.600 seconds with a 454,638-byte report.
PID 26360 queued atomic W/A on window thread 16028 with a 0.0015 ms interval/span. W-up followed
264.2536 ms later; Escape followed W-up after 260.9478 ms.

The actor retained its generation/region/shape, remained grounded with zero vertical velocity, and
finished at local `(-848,-368)` Q9. The final displacement decomposes uniquely into 16
`(-23,-23)` diagonal Walk steps plus 15 `(-32,0)` retained-left Walk steps. Final presentation was
Walk clip 1/yaw 32,768; animation epoch advanced from 1 to 63.

Clock Ready/sample advanced from `2/3` to `92/93` with unchanged reset/suspend/resume/stall counts.
Render blocks remained zero, object observation/interaction remained idle, and exit code zero
contained exactly two values. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests
passed; Flavor retained zero denies and five existing warnings. No product, Runtime,
renderer/GPU, source, synchronization, resource, or process-count owner changed.

## Conclusion

Accepted. One existing real process now proves a bounded native locomotion turn: exact diagonal
Walk transitions to exact retained-left Walk after W release, with both phases recoverable from
the final canonical actor state and no new product or acceptance surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
