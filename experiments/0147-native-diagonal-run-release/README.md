# Experiment 0147: Native Diagonal Run Release

Status: Accepted

## Hypothesis

The existing native diagonal-Run process can prove a second Run direction without another child or
product output. Atomic Shift/W/A should first produce exact `(-45,-45)` Q9 diagonal Run steps;
after a bounded delayed W release, retained Shift+A should produce exact `(-64,0)` Q9 left-Run
steps and final yaw 32,768. Both phases should be uniquely recoverable from final displacement.

## Scope

- Extend the existing post-readiness diagonal-Run native sequence with delayed W-up.
- Hold retained Shift+A for a second bounded interval before the existing Escape completion.
- Require the initial Shift/W/A prefix to remain atomic on the exact visible PID/window thread.
- Decompose final displacement exactly as
  `deltaZ=-45*diagonalSteps`, `deltaX=-45*diagonalSteps-64*leftSteps`.
- Require at least one step in both phases, final Run clip 2/yaw 32,768, clock continuity, idle
  object policy, zero render blocks, and the existing two-value graceful completion.
- Advance the complete Prototype workflow revision from v61 to v62.

Product input/locomotion/presentation policy, session schema/output, process count, bootstrap,
Runtime, renderer/GPU resources, source formats, synchronization, and resource cleanup are out of
scope.

## Workload

1. Launch the existing diagonal-Run child to grounded stationary readiness.
2. Atomically queue Shift-down, W-down, and A-down on its exact visible window thread.
3. Hold diagonal Run for at least 250 ms, then post W-up while retaining Shift+A.
4. Hold left Run for at least 250 ms, post Escape, and consume the existing completion value.
5. Derive both exact positive step counts from final X/Z, then validate final presentation,
   actor/clock/frame/object state, transport timing, and process cleanup.

## Controlled Variables

- Camera orbit remains zero; local W/A maps directly to negative world Z/X.
- Diagonal and cardinal Run components remain fixed at 45 and 64 Q9.
- Initial Shift/W/A is one atomic prefix; W-up and Escape use the maintained monotonic delayed
  transport.
- Shift and A remain held through completion; Escape owns normal process exit.
- No intermediate product output, inspect path, position polling, retry, threshold relaxation,
  telemetry, extra child, event history, or copied state is added.

## Metrics

- PID/window/thread identity; Shift/W/A atomic span and intervals; diagonal and retained-left hold
  intervals; final X/Z; derived diagonal/left step counts; actor identity/region/body/presentation/
  epoch; clock/frame/object state; render blocks; exit code/output count; workflow duration/report
  bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- Shift/W/A must be posted atomically after readiness on one exact visible process window thread
  with 0..=50 ms intervals/span.
- W-up must follow A-down by 250..=750 ms; Escape must follow W-up by 250..=750 ms.
- Final Z must be a negative multiple of 45. `deltaZ-deltaX` must be a positive multiple of 64.
  Both derived step counts must be in `1..=512`.
- Actor handle, region, half-height, grounding, and zero vertical velocity must remain valid.
- Final presentation must be Run clip 2/yaw 32,768 with a later epoch than stationary readiness.
- Clock Ready/sample counts must advance without reset/suspend/resume/stall changes; render blocks
  and object activity must remain zero.
- Exit code must be zero with exactly readiness plus completion, and no product/Runtime/GPU/source/
  resource/process-count diff may exist.

## Results

`canonical-prototype-v62` passed on its first run in 169.946 seconds with a 455,214-byte report.
PID 15960 queued atomic Shift/W/A on window thread 30396 with 0.0015/0.0018 ms key intervals and a
0.0033 ms total span. W-up followed 268.9852 ms later; Escape followed W-up after 262.2319 ms.

The actor retained its generation/region/shape, remained grounded with zero vertical velocity, and
finished at local `(-1744,-720)` Q9. The final displacement decomposes uniquely into 16
`(-45,-45)` diagonal Run steps plus 16 `(-64,0)` retained-left Run steps. Final presentation was
Run clip 2/yaw 32,768; animation epoch advanced from 1 to 48.

Clock Ready/sample advanced from `2/3` to `78/79` with unchanged reset/suspend/resume/stall counts.
Render blocks remained zero, object observation/interaction remained idle, and exit code zero
contained exactly two values. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests
passed; Flavor retained zero denies and five existing warnings. No product, Runtime,
renderer/GPU, source, synchronization, resource, or process-count owner changed.

## Conclusion

Accepted. One existing real process now proves a bounded native Run turn: exact diagonal Run
transitions to exact retained-left Run after W release, with both phases recoverable from the final
canonical actor state and no new product or acceptance surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
