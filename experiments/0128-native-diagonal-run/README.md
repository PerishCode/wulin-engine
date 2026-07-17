# Experiment 0128: Native Diagonal Run

Status: Accepted

## Hypothesis

One atomic native Shift/W/A startup batch should drive the exact local
forward-left Run command through the real Windows input boundary: `(-45,-45)` Q9
per fixed step, imported Run clip 2, and yaw 40,960. Every zero-delay startup
input must already be queued when its selected window thread resumes so a first
live readiness frame cannot overtake a press edge or held command.

## Scope

- Add one exact visible-window atomic Shift/W/A startup sequence with delayed
  Escape.
- Require readiness and completion to retain equal negative X/Z displacement in
  exact 45-Q9 components, Run clip 2, yaw 40,960, and one unchanged animation
  epoch.
- Permit one-key exact-window atomic batches and atomically queue each startup
  request's zero-delay prefix before the window thread resumes.
- Preserve the 500 ms transition semantics of Run release and re-press after
  their atomic startup prefixes.
- Replace native window-action evidence schema 3 directly with schema 4; retain
  no compatibility decoder or fallback.

Product `HostInput`, locomotion, presentation, clock, session schema, Runtime,
renderer/GPU, resource, synchronization, and object policy are out of scope.

## Workload

1. Compare unit-only Shift/W/A diagonal Run reduction with the current
   real-process matrix.
2. Prepare the existing helper before child launch, select the exact visible
   Prototype window, and suspend its window thread.
3. Queue the request's zero-delay startup prefix before resuming the thread;
   retain later authored delays and monotonic delayed Escape.
4. Validate exact native messages, actor motion, presentation, epoch, camera,
   clock, object-idle, and two-value completion invariants.
5. Preserve all previous Prototype gates and run `runseal :guard` plus
   `runseal :init`.

## Controlled Variables

- Startup input is exactly Shift-down, W-down, and A-down in one
  suspended-window-thread batch.
- Camera orbit stays zero, so local W/A maps directly to negative world X/Z.
- Run components remain the fixed nearest-normalized value 45 Q9.
- Escape is posted only after the existing monotonic 200 ms lower bound.
- Run release atomically queues Shift/W before its delayed Shift-up; Run
  re-press atomically queues W before its delayed Shift-down.
- No discontinuity, render block, object intent, target, action, consumption,
  copied state, retry, product delay, or relaxed threshold is admitted.

## Metrics

- Exact PID/window/thread; schema; atomic-prefix length, span, and key
  intervals; message order; delayed-Escape interval; readiness/final position,
  clip, yaw, epoch, frame and clock counts; render blocks; object state; output
  count; workflow duration; Flavor findings; and report inventory.

## Acceptance Criteria

- Shift/W/A are queued on one suspended exact window thread within a finite span
  no greater than 50 ms, and evidence reports an atomic prefix of three.
- Readiness is a nonzero exact `(-45,-45)` multiple with Run clip 2 and yaw
  40,960.
- Completion adds at least one more equal negative 45-Q9 step, retains clip/yaw,
  and does not reset the animation epoch.
- Single-key camera and Jump startup proofs retain their exact edge outcomes.
- Run release and re-press retain their 500..=1,000 ms transition intervals
  after atomic startup prefixes.
- Clock continuity, zero render blocks, idle object state, exactly two output
  values, every previous Prototype gate, `runseal :guard`, and `runseal :init`
  pass.

## Results

The first full `canonical-prototype-v43` run failed before the new gate at the
existing camera-relative E/W readiness oracle: the first live frame committed
stationary orbit-zero Survey before the helper's startup messages were consumed.
A synchronous window-response probe was tested, but the next full run failed
later at the existing camera re-press session because the live frame could still
commit between the probe response and E-down. The probe was deleted rather than
retained as a fallback.

The transport now treats the zero-delay startup prefix as the actual
synchronization boundary. It permits one-key batches, suspends the selected
window thread, queues the complete prefix, and resumes only afterward. Delayed
Run transitions atomically queue their initial Shift/W or W prefix while
retaining their later 500 ms message. Native evidence moved directly to
`prototype-native-window-action-v4` with exact `atomicPrefixLength`; schema 3
has no live decoder or branch.

Three focused real-process probes passed before the full rerun. Single E
committed orbit 1 with a zero-span batch, E/W committed orbit 1 with a 0.0017 ms
span, and single Space committed one Jump with a zero-span batch. Run release
used a 0.0123 ms Shift/W prefix, released Shift after 526.1612 ms, and
transitioned Run to Walk. Run re-press used a zero-span W prefix, pressed Shift
after 525.846 ms, and transitioned Walk to Run. A focused diagonal Run used a
0.0022 ms Shift/W/A span, committed one ready step plus 12 completion steps, and
retained clip/yaw/epoch.

The final `canonical-prototype-v43` passed in 161.489 seconds with every
previous gate. The new session used PID 8,092 and window thread 7,344. Shift/W/A
intervals were 0.0016/0.0010 ms with a 0.0026 ms batch span; Escape followed
207.613 ms later. Readiness was local `(-45,-45)`, one exact diagonal Run step,
clip 2, yaw 40,960, and epoch 5. Completion was local `(-585,-585)`: an
additional `(-540,-540)`, exactly 12 more 45-Q9 diagonal steps. Clip/yaw stayed
exact and epoch remained 5. Clock ready/sample advanced `4/5 -> 61/62`;
reset/suspend/resume/stall stayed `1/0/0/0`, render blocks stayed zero, object
state remained idle, and stdout contained exactly readiness plus completion.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. Final
Flavor reported zero denies and five existing warnings, `runseal :init` passed,
and the generated acceptance inventory is one JSON file / 524,119 bytes. No Rust
product, Runtime, traversal, renderer, GPU resource, source, or synchronization
code changed.

## Conclusion

Accepted. Native Shift/W/A now proves exact diagonal Run normalization and
forward-left presentation through the real host boundary. Startup input ordering
is the queued atomic prefix itself, including single press edges, rather than a
response-before-post timing assumption.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
