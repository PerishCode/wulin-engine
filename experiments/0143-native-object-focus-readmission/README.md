# Experiment 0143: Native Object Focus Readmission

Status: Accepted

## Hypothesis

The existing Activated object-feedback process can prove the complete F/Enter focus lifetime
without another child or product output. An atomic F/Enter batch immediately before focus loss
must be cancelled by the activation/time discontinuity, while one later atomic F/Enter batch after
focus recovery must still observe, activate, acknowledge, and consume the exact canonical object
once.

## Scope

- Atomically post F-down, Enter-down, and `WM_KILLFOCUS` to the exact visible Activated process
  window thread after idle readiness.
- Resume focus, wait for the existing reset/recovery boundary, and retain the existing atomic
  F/Enter plus delayed-Escape sequence.
- Require one exact focus suspend/resume/reset boundary, zero stale object-policy effects, and the
  existing stationary Activated source oracle.
- Keep the separate Rejected object-feedback process and the main focus-discontinuity process
  unchanged.
- Advance the complete Prototype workflow revision from v57 to v58.

Object observation/interaction policy, HostInput normalization, main-loop ordering, session schema
and process count, Runtime, renderer/GPU resources, source formats, synchronization, and lifecycle
ownership are out of scope.

## Workload

1. Launch the existing Activated object-feedback session to grounded idle readiness.
2. Atomically post F-down, Enter-down, and focus loss on the same PID/window thread.
3. Observe suspended sampling, resume focus, and wait 250 ms for the existing reset/recovery
   boundary.
4. Atomically post fresh F-down and Enter-down, then post Escape after the existing 250 ms lower
   bound.
5. Require stationary exact-object observation, one Activated commit, 12 projected
   acknowledgement frames, exact consumption, and clean completion.

## Controlled Variables

- Object source, fixture, identity, radius, facing, observation and interaction capacities,
  acknowledgement length, actor state, camera, traversal, fresh F/Enter action, exit delay,
  two-value session framing, and Rejected process remain unchanged.
- The Experiment 0141 main focus process remains the stale Jump/observation/activation/W plus fresh
  A authority.
- No new process, product output, pending-action history, event queue, retry, compatibility alias,
  telemetry, or product behavior is added.

## Metrics

- Exact PID/window/thread identity; ordered native messages; stale and fresh atomic spans;
  fresh-action-to-exit timing; actor identity/body/presentation; object target, committed and
  ineligible counts; consumed identity; Activated/acknowledgement/suppression frame counts; clock
  counters; live/render-block counts; output count; workflow duration/report bytes; test counts;
  Flavor findings; and process cleanup.

## Acceptance Criteria

- The discontinuity batch must target one exact PID/window and atomically emit
  `WM_SETFOCUS/F-down/Enter-down/WM_KILLFOCUS` on one positive thread with a 0..=50 ms span.
- Recovery must emit exact `WM_SETFOCUS`; the later sequence must retain exact
  `WM_SETFOCUS/F-down/Enter-down/Escape` on the same PID/window with an atomic F/Enter prefix and
  delayed exit.
- Completion must retain the exact readiness actor and contain one committed action, zero
  ineligible actions, no pending observation/interaction, 12 Activated frames, and exact consumed
  source/owner-region/authored-local-ID identity.
- The clock must record exactly one suspend, one resume, and one post-resume reset with increased
  suspended/Ready samples, zero stalls/render blocks, and exactly two outputs.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count diffs
  must remain empty.

## Results

`canonical-prototype-v58` passed on its first run in 169.078 seconds. The 448,810-byte report
recorded PID 13320 and window thread 22076. The stale F/Enter/focus-loss batch and fresh F/Enter
batch each spanned 0.0013 ms; the fresh batch preceded Escape by 275.2869 ms.

The actor remained generation 1 at signed region
`(1099511627776,-1099511627776)`, local `(0,0)` Q9, with half-height 65,536,
zero vertical velocity, Survey clip 0/yaw 0, and animation epoch 1. The final object policy had
exactly one commit, zero ineligible attempts, no pending state, and 12 Activated frames. It consumed
authored local ID 496 in the actor's region under source namespace
`99d9511b8cea49a59d771d97874d56bb7790a79c880490353852bc75aa4fd94d`.

Clock suspend/resume/reset counts changed by exactly `+1/+1/+1`. Suspended samples advanced
`0 -> 83`, Ready/sample counts advanced `2/3 -> 179/264`, stall count remained zero, and all 264
live frames had zero render blocks. Exit code was zero and stdout contained exactly readiness plus
completion.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard`
retained zero Flavor denies and five existing warnings. No product, Runtime, renderer/GPU, source,
resource, or synchronization file changed.

## Conclusion

Accepted. Focus loss now has an exact real-process F/Enter lifetime witness: pre-loss observation
and activation intents cannot reach resumed simulation, while a later normalized F/Enter batch is
admitted as one exact Activated action without new state or surfaces.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
