# Experiment 0134: Bounded Object Suppression Dwell

Status: Accepted

## Hypothesis

The maintained post-readiness Activated session cannot reliably prove suppression when Escape is
requested exactly 200 ms after F/Enter. Twelve successful acknowledgement frames already consume
that nominal interval at 60 Hz. Raising only the acceptance helper's requested dwell to 250 ms
should preserve the exact action contract while guaranteeing at least one following suppression
frame in the existing process.

## Scope

- Diagnose the repeated pre-focus Activated completion failure with temporary exact metrics.
- Change the existing exact-PID F/Enter/Escape helper from a 200 ms to a 250 ms requested exit
  delay.
- Validate the observed F/Enter-to-Escape interval in `[250,750]` ms.
- Retain exactly 12 Activated acknowledgement frames and require at least one suppression frame.
- Add a static guard against restoring the 200 ms lower bound.

Product object action/suppression policy, acknowledgement count, session schema, Runtime,
renderer/GPU resources, source formats, synchronization, and process count are out of scope.

## Workload

1. Attempt the proposed focus-action extension through the full Prototype workflow.
2. On repeated earlier Activated failures, temporarily include all six relevant completion metrics
   in the existing failure and rerun without changing thresholds.
3. Remove the temporary diagnostic and defer the unrelated focus extension.
4. Request Escape 250 ms after the existing atomic post-readiness F/Enter batch.
5. Require exact committed identity/exclusion, exactly 12 Activated frames, at least one later
   suppression frame, idle final action state, a stationary actor, zero render blocks, and
   two-value completion.
6. Run all existing Prototype sessions, Sidecar lifecycle, static guard, and initialization gates.

## Controlled Variables

- The same process, window, native F/Enter atomic batch, object source, actor origin/facing,
  acknowledgement count, consumption/exclusion, suppression policy, and Escape completion remain
  unchanged.
- The helper still uses one monotonic delayed exit and schema 4 evidence.
- No product delay, retry, relaxed semantic threshold, intermediate report, event history,
  fallback, or compatibility alias is admitted.

## Metrics

- Committed/ineligible counts, observation target, Activated/Rejected/suppression frame counts,
  exact process/window/thread identity, atomic interval/span, requested and observed exit delay,
  consumed/excluded identity, actor state, render blocks, output count, workflow duration, report
  bytes, Flavor findings, and process cleanup.

## Acceptance Criteria

- The diagnosed failure must identify one specific unmet invariant without changing the product or
  semantic oracle.
- F/Enter begins only after readiness and remains one exact-PID, exact-window-thread atomic batch.
- Escape is requested after exactly 250 ms and observed in `[250,750]` ms.
- Completion has committed count 1, ineligible count 0, exactly 12 Activated frames, zero Rejected
  frames, at least one suppression frame, exact consumed/excluded identity, and no pending target or
  acknowledgement.
- Actor state remains readiness, render blocks remain zero, output remains exactly two values, and
  every previous session/lifecycle gate passes.

## Results

Three initial `canonical-prototype-v49` runs failed before any focus session at the unchanged
post-ready Activated completion oracle. The third run temporarily reported the complete metric set:
`committed=1`, `ineligible=0`, `observationTarget=null`, `activated=12`, `rejected=0`, and
`suppression=0`. The action and acknowledgement were exact; only the next suppression frame was
missing. The original 200 ms delayed Escape matched the nominal duration of twelve 60 Hz frames and
therefore relied on scheduler overshoot to retain a thirteenth frame.

The temporary diagnostic and the unproven focus F/Enter extension were removed. The maintained
object helper now requests 250 ms, its oracle accepts only observed intervals in `[250,750]` ms,
and the static guard requires the new lower bound. No semantic condition was relaxed.

Final `canonical-prototype-v49` passed in 174.564 seconds. PID 28412 used window thread 31632 to
post F and Enter 0.0016 ms apart in one atomic batch, then observed Escape 270.6458 ms later. The
action consumed and excluded exact authored ID 496, completed exactly 12 Activated frames, zero
Rejected frames, and two suppression frames. Final observation target, pending action,
acknowledgement, and render-block count were all empty/zero; actor state remained readiness and
stdout remained exactly two values. The ignored report was 446,569 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU, source,
resource, or synchronization code changed.

## Conclusion

Accepted. The post-readiness object gate now observes suppression beyond the exact 12-frame
acknowledgement without depending on scheduler overshoot. The fix changes only bounded acceptance
timing and preserves the complete product/action contract.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
