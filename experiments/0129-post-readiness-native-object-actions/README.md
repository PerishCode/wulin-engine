# Experiment 0129: Post-Readiness Native Object Actions

Status: Accepted

## Hypothesis

Native edge-triggered object acceptance becomes deterministic when readiness first establishes the
exact product process and idle policy state, then F/Enter is posted to that PID and the existing
single completion value proves the resulting action. Suspending a newly visible window thread and
queueing startup messages before resuming it cannot prove that those messages preceded the current
frame's already-completed message pump.

## Scope

- Replace the startup `"object-action"` request with exact-PID post-readiness F/Enter actions.
- Make Activated and Rejected use stationary readiness plus one delayed-Escape completion.
- Make the sustained capacity session commit its first consumption after readiness before the
  existing D motion and exclusion-aware second action.
- Derive exact target identities from the existing source oracle at the committed final actor
  position.
- Delete action-specific readiness observation/interaction oracles that no longer have a caller.

Product input, object policy, session schema/output count, Runtime, renderer/GPU, source formats,
fixtures, synchronization, and gameplay semantics are out of scope.

## Workload

1. Reproduce the full-workflow object-action variance after schema-4 atomic startup prefixes.
2. Compare window creation/show, bootstrap pumping, live pumping, input ingest, and readiness
   publication ordering.
3. Replace all three maintained object-action gates with post-readiness exact-PID input.
4. Require idle readiness, exact source-qualified identity, 12-frame feedback, final
   consumption/rejection/exclusion state, clock/frame continuity, and exactly two output values.
5. Delete the retired startup request and action-specific readiness oracle branches.
6. Run focused real-process probes, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :init`.

## Controlled Variables

- The product still pumps messages, ingests input, advances simulation, resolves object actions,
  renders feedback, and publishes readiness/completion in the same order.
- F and Enter remain one schema-4 atomic exact-window-thread batch.
- Activated and Rejected retain the existing base and `base + 4` stationary fixtures.
- The primary action sessions retain a 200 ms delayed Escape.
- The sustained session retains 250 ms before D motion, the existing capacity action, and 250 ms
  before Escape.
- Acknowledgement capacity remains 12 successful projected frames.
- No event stream, retry, dynamic outcome acceptance, product delay, threshold relaxation, copied
  object state, or additional product output is admitted.

## Metrics

- Exact PID/window/thread, F/Enter atomic span, message order, delayed-Escape interval, idle
  readiness, final actor identity/position, source-qualified consumed/target identity, committed
  and ineligible counts, Activated/Rejected/suppression frames, render blocks, output count,
  workflow duration, report bytes, Flavor findings, and process cleanup.

## Acceptance Criteria

- Readiness precedes every maintained native object action and contains idle observation and
  interaction state.
- Activated consumes the exact source-oracle identity, projects exactly 12 Activated frames,
  clears its retained target through existing suppression, and commits no ineligible action.
- Rejected retains the exact resolved source-oracle target, projects exactly 12 Rejected frames,
  commits no consumption, and records one ineligible action.
- Sustained capacity commits its first consumed identity after readiness, moves only after that
  consumption, then retains committed/ineligible `1/1`, exact exclusion-aware second target, 12
  Activated, 12 Rejected, and nonzero suppression frames.
- The startup `"object-action"` request and action-specific readiness oracles have no live branch.
- Every prior Prototype gate, `runseal :guard`, and `runseal :init` pass without product, Runtime,
  renderer/GPU, source-format, resource, or synchronization changes.

## Results

The live order explains the race. `window::show` is immediately followed by the live loop's
`pump_messages`. The helper may observe visibility and suspend the window thread only after that
pump has returned. Its queued F/Enter then waits for the next pump while the resumed current frame
can already advance and publish readiness. Queue-before-resume is exact transport evidence but is
not a message-pump-before-frame transaction.

Two expanded pre-fix runs failed in different existing gates. One sustained session reached
readiness with `consumed=null`; a later Rejected fixture reached readiness with
`object_observation_driver.completed=false`. An isolated sustained run and a fixed three-run
diagnostic sample all passed, demonstrating timing variance rather than a fixture, object policy,
or source-oracle mismatch. No retry or timing threshold was added.

The direct replacement reads idle readiness first and then targets the known child PID. Focused
real-process acceptance passed. Activated PID 15,312 used a 0.0013 ms F/Enter span, exited after
210.5996 ms, consumed authored ID 496, projected 12 Activated and 71 suppression frames. Rejected
PID 452 used a 0.0015 ms span, exited after 208.2418 ms, retained authored ID 495, and projected 12
Rejected frames. Sustained PID 9,480 used a 0.0012 ms first-consumption span and finished with
committed/ineligible `1/1`, 12 Activated, 12 Rejected, and 2,096 suppression frames.

Final `canonical-prototype-v44` passed in 163.431 seconds with every prior gate. Activated PID
8,908 used a 0.0012 ms span and 206.5016 ms delayed exit, consumed ID 496, and projected 12
Activated plus 64 suppression frames. Rejected PID 3,764 used a 0.0270 ms span and 215.1196 ms
exit, retained ID 495, and projected 12 Rejected frames. Sustained PID 1,596 used a 0.0012 ms first
span, held before capacity work for 262.4291 ms, moved to local X 2,400, retained consumed ID 496
and exclusion-aware target ID 505, and projected 12 Activated, 12 Rejected, and 2,133 suppression
frames. The existing finite-boundary process remained live for 15,014.427 ms.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. Flavor reported zero
denies and five existing warnings. The report is one ignored JSON file of 519,872 bytes. No Rust
product, Runtime, renderer/GPU, source, resource, or synchronization code changed.

## Conclusion

Accepted. Native object-action evidence now begins at an explicit product readiness boundary and
targets the established PID. Window-thread atomicity remains transport evidence; it is no longer
misstated as proof that startup edge messages preceded an already-running frame.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
