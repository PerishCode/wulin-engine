# Experiment 0141: Native Focus Locomotion Readmission

Status: Accepted

## Hypothesis

The existing focus-discontinuity process can prove both sides of the locomotion lifetime boundary
without another child or product output. Space/F/Enter/W posted immediately before focus loss must
not reach resumed simulation, while a later fresh A press after focus recovery must commit exact
Walk movement and release back to Survey.

## Scope

- Retain the exact atomic Space/F/Enter/W/`WM_KILLFOCUS` batch on the visible Prototype window
  thread.
- After the existing suspended dwell and explicit focus resume, atomically post A-down, hold it
  for at least 250 ms, post A-up, then wait at least 250 ms before Escape.
- Replace the old unchanged-actor focus oracle with exact negative-X Walk displacement, zero Z
  displacement, stable identity/region/body, and final Survey yaw 32,768.
- Advance the complete Prototype workflow revision from v55 to v56.

Product HostInput, locomotion/presentation policy, main-loop ordering, session schema/process count,
Runtime, renderer/GPU resources, source formats, synchronization, and lifecycle ownership are out
of scope.

## Workload

1. Launch the existing post-readiness focus-discontinuity session from grounded idle Survey.
2. Atomically post Space/F/Enter/W and focus loss while the exact window thread is suspended.
3. Observe suspended sampling, resume focus, and allow the existing clock reset/recovery boundary.
4. On the same PID/window, atomically post A-down, post A-up after 250 ms, and post Escape after a
   further 250 ms.
5. Require only exact 32-Q9 negative-X steps, zero Z movement, final Survey clip 0/yaw 32,768,
   idle object policies, exact clock recovery, zero blocks/stalls, and exactly two output values.

## Controlled Variables

- The same focus process, native helper schema, suspended/resumed dwell, camera orbit 0, exact Walk
  command, playable boundary, gravity, session completion, and Escape ownership remain unchanged.
- Same-batch Jump, observation, activation, and forward-locomotion suppression remain live.
- No intermediate product output, input history, event queue, retry, compatibility alias,
  telemetry, or product behavior is added.

## Metrics

- Exact PID/window/thread identity; ordered native messages; atomic spans; A-held and stationary
  intervals; actor handle/region/position/body/animation epoch/presentation; clock counters;
  frame/render-block counts; object state; output count; workflow duration/report bytes; test
  counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- The original batch must remain exact
  `Space-down/F-down/Enter-down/W-down/WM_KILLFOCUS` on one positive window thread.
- The later action must target the same PID/window and emit exact
  `A-down/A-up/Escape`, with an atomic single-key prefix and held/stationary intervals each in
  `[250,750]` ms.
- Completion must move negative X by 1..=512 exact 32-Q9 Walk steps and move Z by exactly zero,
  proving the stale W does not reach resumed simulation and the fresh A does.
- Actor handle and region must remain stable, vertical velocity must be zero, animation epoch must
  advance, and final presentation must be Survey clip 0/yaw 32,768.
- The clock must record exactly one suspend/resume pair and one post-resume reset with later Ready
  progress, zero stalls/render blocks, idle object state, and exactly two process values.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count
  diffs must remain empty.

## Results

`canonical-prototype-v56` passed on its first run in 180.909 seconds. The 442,700-byte report
recorded PID 9148 and window thread 24044 for both atomic actions. The four-key pre-loss batch
spanned 0.0034 ms. After recovery, A remained held for 272.3119 ms and the process remained
stationary for 261.9928 ms after A-up before Escape.

The actor retained generation 1 and signed region
`(1099511627776,-1099511627776)`, moved from local `(0,0)` to `(-512,0)` Q9, and therefore
committed exactly 16 left Walk steps with no forward displacement. It finished with zero vertical
velocity as Survey clip 0/yaw 32,768; animation epoch advanced from 1 to 234.

Clock suspend/resume/reset counts changed by exactly `+1/+1/+1`. Suspended samples advanced
`0 -> 76`, Ready/sample counts advanced `2/3 -> 171/249`, stall count remained zero, and all 249
live frames had zero render blocks. Jump and object policies remained idle, exit code was zero,
and stdout contained exactly readiness plus completion.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard`
retained zero Flavor denies and five existing warnings. The first guard after Experiment 0140's
scheduled workspace cleanup correctly required regeneration of the maintained Agility SDK through
`runseal :gpu-lab correctness`; the restored guard and complete workflow then passed without an
acceptance retry or product change.

## Conclusion

Accepted. Focus loss is now proven as a complete locomotion lifetime boundary in the maintained
real process: stale pre-loss W cannot reach resumed simulation, while a fresh post-recovery A is
admitted, committed, released, and retained as the final facing without new state or surfaces.

## Reproduction

```powershell
runseal :gpu-lab correctness # only when workspace-local SDK output is absent
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
