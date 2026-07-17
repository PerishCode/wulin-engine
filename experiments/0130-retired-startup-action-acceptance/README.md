# Experiment 0130: Retired Startup Action Acceptance

Status: Accepted

## Hypothesis

Prototype native-action acceptance no longer needs a pre-child helper or next-window request.
Every maintained action can first establish idle product readiness and the exact child PID, then
post input to that process and use the existing completion plus product tests as authority.
Removing readiness-only action checkpoints and startup action reports should reduce workflow cost
without weakening a distinct current behavior gate.

## Scope

- Delete the five readiness-only W, Shift/W, E/W, E, and Space process checkpoints.
- Delete `StartupInput`, `prepareStartupInput`, all startup request cases, next-window/PID-zero
  selection, and the `startupNativeInput` report field.
- Move camera repeat/re-press, Run release/re-press, opposite locomotion, and diagonal Walk/Run
  acceptance behind idle readiness and exact PID ownership.
- Replace `Start-Sleep` delayed-key admission with an exact monotonic lower-bound deadline.
- Delete the four action-only command expectations and the unused start-only camera helper.

Product input, camera, locomotion, Jump, object policy, Runtime, renderer/GPU, source formats,
fixtures, synchronization, and the two-value session contract are out of scope.

## Workload

1. Inventory every action-bearing `capturedReady` call and every `StartupInput` request/caller.
2. Map each old readiness-only behavior to its stronger maintained completion session or focused
   product test.
3. Remove the forced-readiness action processes and action capability from `capturedReady`.
4. Move every remaining acceptance action after readiness and address only the established child
   PID.
5. Require exact native messages, atomic prefixes/batches where authored, final locomotion or
   camera outcome, clock continuity, idle object policy, and exactly two output values.
6. Run the full Prototype workflow, static removal guard, and repository initialization gate.

## Controlled Variables

- Product message pumping, input normalization, simulation, camera, presentation, object policy,
  frame rendering, and completion publication are unchanged.
- Existing key order, 250/500 ms holds, 200 ms delayed Escape, and exact Walk/Run components remain.
- Schema 4 native evidence remains the sole transport schema.
- The current helper preparation marker remains, but every action now selects one explicit PID.
- No retry, fallback, event stream, product delay, dynamic outcome acceptance, relaxed threshold,
  compatibility decoder, or additional product output is admitted.

## Metrics

- Process/checkpoint count, workflow duration, report bytes, exact PID/window/thread, atomic span,
  delayed-key/exit intervals, readiness state, final actor position/presentation/camera result,
  frame/clock continuity, test counts, Flavor findings, and process cleanup.

## Acceptance Criteria

- No acceptance path contains `StartupInput`, a startup request switch, PID-zero/next-window
  selection, `startupNativeInput`, or a readiness-only action checkpoint/report field.
- `capturedReady` accepts no input and owns only plain readiness/failure/restart baselines.
- Every maintained native action starts after readiness and targets the exact child PID.
- Delayed keys and delayed Escape meet their declared lower bounds by monotonic deadline.
- Camera repeat/re-press, Run release/re-press, opposite locomotion, diagonal Walk/Run, Jump,
  object, focus, and exit sessions retain exact two-value completion evidence.
- All previous Prototype gates, `runseal :guard`, and `runseal :init` pass without product,
  Runtime, renderer/GPU, source-format, resource, or synchronization changes.

## Results

The audit found five redundant forced-readiness action processes. Current two-value sessions and
focused product tests already own Walk/Run, camera edge/locomotion, and Jump behavior. Removing
those processes exposed the same timing assumption documented by Experiment 0129: the first v45
run reached camera re-press readiness at orbit 0 instead of the startup E gate's expected orbit 1.
That failure confirmed the startup transport was still an acceptance compatibility surface.

The second v45 run exposed a separate delayed-key bug. A requested 200 ms midair-Jump interval was
measured as 199.2574 ms because the helper used one `Start-Sleep` call while the validator correctly
required the authored lower bound. The helper now computes a Stopwatch deadline and yields/sleeps
until that deadline; the threshold was not relaxed and the run was not accepted by retry.

The direct replacement removes the complete startup request/type/report chain and makes every
action exact-PID post-readiness work. Camera held-repeat used a 267.5133 ms initial hold and ended
at orbit 1 with `(-384,0)` Q9 movement. Camera re-press held 260.4763 ms, used a 0.0026 ms atomic
release/re-press/W span, and ended at orbit 2 with `(0,416)` Q9 movement. Run release and re-press
met 512.4148/514.1356 ms holds and ended as exact Walk/Run. Opposed Shift/W/S held 259.2241 ms
before S release and completed 12 exact Run steps.

Post-readiness diagonal Walk used a 0.0012 ms W/A span and completed 13 steps at
`(-299,-299)` Q9. Diagonal Run used a 0.0023 ms Shift/W/A span and completed 13 steps at
`(-585,-585)` Q9. Both began from idle Survey readiness and committed the expected
Walk/Run presentation transition. The maintained midair-Jump sequence measured 213.4783 ms before
the second press and 207.8540 ms before exit. The finite-boundary process received W only after
readiness and remained live for 15,016.9422 ms.

Final `canonical-prototype-v45` passed in 144.642 seconds, 18.789 seconds faster than v44. The
ignored report fell from 519,872 to 442,038 bytes, a 77,834-byte reduction. All 103 engine-runtime,
45 Prototype, and 20 reference-host tests passed, as did every prior process/session/Sidecar gate.
No Rust product, Runtime, renderer/GPU, source, resource, or synchronization code changed.

## Conclusion

Accepted. Acceptance startup actions, next-window selection, and their report surface are retired.
Product readiness and exact PID ownership now precede every maintained native action. Authored key
delays and exits share monotonic lower-bound timing rather than scheduler sleep assumptions.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
