# Experiment 0136: Native Focus Object-Intent Suppression

Status: Accepted

## Hypothesis

The existing focus-discontinuity process can prove that both Prototype object intents obey the same
activation-before-action boundary already proven for Jump. If F and Enter are added to the exact
window-thread Space/W/focus-loss batch, neither intent may reach resumed nonzero simulation after
Suspended/Reset, and the final object policies must remain completely idle.

## Scope

- Extend the maintained atomic focus batch from Space/W to Space/F/Enter/W.
- Retain the same exact PID/window/thread, `WM_KILLFOCUS`, bounded resume/exit dwell, and two-value
  completion.
- Require idle observation and interaction policies at readiness and completion.
- Preserve the existing Jump, locomotion, actor, clock recovery, and zero-block oracles.
- Advance the full Prototype workflow revision from v50 to v51.

Product HostInput, observation/interaction policy, activation/clock ordering, session schema,
Runtime, renderer/GPU resources, source formats, synchronization, and process count are out of
scope.

## Workload

1. Re-audit the product loop from input ingestion through activation-aware sampling, policy sample
   observation, edge admission, Ready-only actor advance, and post-advance object completion.
2. Begin from grounded actor readiness with no observation target/pending state and no interaction
   pending/acknowledgement/count/consumption state.
3. Suspend the exact visible window thread, atomically post Space, F, Enter, and W down, then post
   `WM_KILLFOCUS` before releasing the thread.
4. Wait at least 250 ms, post `WM_SETFOCUS`, wait at least 250 ms, and exit normally through Escape.
5. Require one suspend/resume pair, one additional reset, suspended samples followed by Ready work,
   exact unchanged actor state, idle final object policies, zero action counts, and zero stalls,
   render blocks, or elapsed backlog.
6. Run every existing Prototype session, Sidecar lifecycle, static guard, and initialization gate.

## Controlled Variables

- Space, W, focus loss/resume, actor state, clock recovery, process count, and product output remain
  the accepted Experiment 0133 contracts.
- Only F and Enter are added to the same atomic batch.
- The object-action radius, nearest query, target ownership, eligibility, acknowledgement,
  consumption, suppression, and feedback paths remain unchanged.
- No intermediate product report, event history, retry, relaxed threshold, fallback, compatibility
  alias, or new action state is admitted.

## Metrics

- Exact process/window/thread identity, ordered messages, three key intervals, total batch span,
  readiness and final object-policy state, actor equality, clock suspend/resume/reset/Ready/
  suspended counters, frame/render-block counts, output count, workflow duration, report bytes,
  test totals, Flavor findings, and process cleanup.

## Acceptance Criteria

- Native evidence must be exactly focus, Space-down, F-down, Enter-down, W-down, and focus loss in
  one exact-window-thread atomic batch after readiness.
- All three inter-key intervals and the complete batch span must be in `[0,50]` ms.
- Readiness and completion must have no pending observation/interaction, no target,
  acknowledgement, consumption, or exclusion, and zero committed/ineligible action counts.
- Completion must add exactly one suspend/resume pair and one reset, include suspended samples and
  later Ready/live-frame progress, and retain actor state exactly equal to readiness.
- Stalls, elapsed backlog, render blocks, extra processes/product values, and product/Runtime/GPU/
  source/resource/synchronization changes must remain absent.

## Results

The loop audit confirmed the same boundary as Experiment 0133. A press may be exposed in the first
ingest containing focus loss, but the activation-aware sample admits no elapsed simulation.
Subsequent Suspended or Reset observation clears the capacity-one object intents before any resumed
Ready actor transaction. The evidence therefore proves suppression across the discontinuity, not
immediate deletion in the first ingest.

The first v51 workflow reached the new focus gate but later failed in the independent invalid-key
process at its unchanged clock-continuity oracle. No focus or product invariant failed, no report
was retained, and no code, threshold, or helper retry was changed. An immediate full operator rerun
was used to distinguish an external host-time discontinuity from repeatable experiment behavior.

Final `canonical-prototype-v51` passed unchanged in 172.935 seconds. PID 5004 used window thread
25564 to post Space/F/Enter/W with intervals 0.0014, 0.0013, and 0.0012 ms; the complete atomic
batch span was 0.0039 ms. Completion recorded exactly one suspend/resume pair, one additional reset,
88 suspended samples, 156 Ready samples, 246 live frames, zero stalls, and zero render blocks.

The final actor remained exactly readiness. Observation had no pending intent or target;
interaction had no pending intent, acknowledgement, consumption, or exclusion and retained
committed/ineligible counts at zero. Output remained exactly two values. The ignored report was
447,315 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU, source,
resource, or synchronization code changed.

## Conclusion

Accepted. The existing focus-discontinuity process now proves that same-batch F observation and
Enter activation intents, beside Jump and held locomotion, cannot survive Suspended/Reset into
resumed nonzero simulation. No process or product surface was added.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
