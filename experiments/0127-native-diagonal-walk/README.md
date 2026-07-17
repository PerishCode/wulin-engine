# Experiment 0127: Native Diagonal Walk

Status: Accepted

## Hypothesis

One atomic native W/A startup batch should drive the exact local forward-left Walk command through
the real Windows input boundary: `(-23,-23)` Q9 per fixed step, clip 1, and yaw 40,960. A helper
readiness handshake before Prototype launch should also eliminate the remaining warm-process
startup race without retries or product delay.

## Scope

- Add native A to the acceptance-only key vocabulary.
- Add one exact visible-window atomic W/A startup sequence with delayed Escape.
- Require readiness and completion to retain equal negative X/Z displacement in exact 23-Q9
  components, Walk clip 1, yaw 40,960, and one unchanged animation epoch.
- Require each startup helper to complete native type preparation and emit one readiness marker
  before its Prototype child is spawned.
- Centralize startup request definitions and delete the superseded start-only dispatch helpers.

Product `HostInput`, locomotion, presentation, clock, session schema, Runtime, renderer/GPU,
resource, synchronization, and object policy are out of scope.

## Workload

1. Compare unit-only W/A diagonal reduction with the current real-process session matrix.
2. Prepare a PowerShell helper, wait for its post-`Add-Type` readiness marker, then launch the
   exact Prototype child it will select.
3. Queue W-down and A-down atomically on the selected window thread and post Escape after the
   existing 200 ms lower bound.
4. Validate exact native messages, actor motion, presentation, epoch, camera, clock, object-idle,
   and two-value completion invariants.
5. Preserve all previous Prototype gates and run `runseal :guard` plus `runseal :init`.

## Controlled Variables

- Startup input is exactly W-down and A-down in one suspended-window-thread batch.
- Camera orbit stays zero, so local W/A maps directly to negative world X/Z.
- Walk components remain the fixed nearest-normalized value 23 Q9; Shift is absent.
- Escape is posted only after the existing monotonic 200 ms lower bound.
- The helper marker is acceptance transport framing only and is not product output.
- No discontinuity, render block, object intent, target, action, consumption, copied state, retry,
  product delay, or relaxed threshold is admitted.

## Metrics

- Helper preparation order; exact PID/window/thread; message order; atomic span and key interval;
  delayed-Escape interval; readiness/final position, clip, yaw, epoch, frame and clock counts;
  render blocks; object state; output count; workflow duration; Flavor findings; and report
  inventory.

## Acceptance Criteria

- The helper emits `prototype-native-helper-ready-v1` only after native type preparation and before
  the Prototype child is spawned.
- Its final window evidence PID equals the spawned child PID.
- W/A are queued on one suspended exact window thread within a finite span no greater than 50 ms.
- Readiness is a nonzero exact `(-23,-23)` multiple with Walk clip 1 and yaw 40,960.
- Completion adds at least one more equal negative 23-Q9 step, retains clip/yaw, and does not reset
  the animation epoch.
- Clock continuity, zero render blocks, idle object state, exactly two output values, every previous
  Prototype gate, `runseal :guard`, and `runseal :init` pass.

## Results

The first full `canonical-prototype-v42` run failed before the new gate at the existing
`run-forward` readiness oracle: expected Shift/W Run but received stationary Survey. Experiment
0126 had launched the PowerShell helper before the child, but process launch alone did not prove
that `Add-Type` compilation and helper initialization had completed before the warm Prototype
published readiness.

The transport now emits one fixed helper-ready marker after `Add-Type` and before window search.
Both captured-readiness and graceful-session owners await that marker before spawning the child,
then await the helper's final evidence and still require its actual window PID to equal the child
PID. Startup definitions are centralized in one request owner; superseded duplicate start-only
action and Run-sequence helpers were deleted. No retry, product sleep, or threshold change was
added.

The first full run after the explicit handshake passed as `canonical-prototype-v42` in 156.480
seconds, including every previous gate. The new session used PID 3964 and window thread 8,896.
`WM_SETFOCUS`, W-down, and A-down were queued atomically with a 0.0012 ms key interval and 0.0012 ms
batch span; Escape followed 214.8289 ms later.

Readiness was local `(-23,-23)`, one exact diagonal Walk step, clip 1, yaw 40,960, and epoch 5.
Completion was local `(-299,-299)`: an additional `(-276,-276)`, exactly 12 more 23-Q9 diagonal
steps. Clip/yaw stayed exact and epoch remained 5. Clock ready/sample advanced `4/5 -> 43/44`;
reset/suspend/resume/stall stayed `1/0/0/0`, render blocks stayed zero, object
observation/interaction remained idle, and output contained exactly readiness plus completion.

Flavor then required the 594-line transport owner to split at its helper process/evidence boundary.
The final owners are 457 and 149 lines. The final-worktree full run passed again in 157.404 seconds:
PID 1852 / thread 2,472 used a 0.0013 ms interval/span and a 213.7342 ms Escape interval; positions,
1+12 steps, clip/yaw, and epoch `5 -> 5` were identical, while clock ready/sample advanced
`4/5 -> 70/71`. All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed with the
complete existing real-process matrix. Final Flavor reported zero denies and five existing
warnings, `runseal :init` passed, and the generated acceptance inventory is one JSON file / 502,641
bytes.

## Conclusion

Accepted. Native W/A now proves the exact diagonal Walk normalization and forward-left facing
through the real host boundary. Startup helpers have an explicit preparation-before-child
contract rather than a scheduling assumption; product and engine behavior remain unchanged.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
