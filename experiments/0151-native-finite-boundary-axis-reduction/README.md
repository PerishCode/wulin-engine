# Experiment 0151: Native Finite-Boundary Axis Reduction

Status: Accepted

## Hypothesis

The sole maintained finite-boundary process can prove the product's independent per-axis admission
without another child or output. After held Shift/W reaches the negative-Z edge, adding held A must
retain the exact 45-Q9 diagonal Run X component after the maximum-eight-step Z candidate becomes
unsafe, keep Z inside the one-region boundary band, and finish as retained tangential-facing
Survey.

## Scope

- Reuse the existing exact-PID boundary process, atomic Shift/W start, 15,000 ms hold, and standard
  sequence-2 completion.
- After the boundary hold, post A-down, hold it for 500 ms, release A/W/Shift, wait 250 ms, and post
  Escape through the existing schema-4 native helper.
- Require exact 45-Q9 negative-X displacement, bounded negative-Z position, at least one provable
  tangential-only Run phase, final Survey/yaw 32,768, continuous clock/frame progress, idle object
  state, and clean process teardown.
- Advance canonical Prototype acceptance from v65 to v66.

Product boundary/input/locomotion/presentation code, bootstrap bounds, Runtime, renderer/GPU
resources, source formats, synchronization, session schema, and process count are out of scope.

## Workload

1. Launch the existing one-region boundary child to grounded idle readiness.
2. Atomically post Shift-down/W-down to the exact visible PID/window thread and hold for at least
   15,000 ms.
3. On the same PID/window, post A-down, delay 500 ms, then release A/W/Shift and delay Escape by
   250 ms.
4. Consume the existing graceful completion and require final local X to encode 16..=48 exact
   45-Q9 steps while local Z remains in inclusive `[-4096,-3648]`.
5. Require final Survey/yaw 32,768, stable actor identity/region/shape, zero vertical velocity,
   continuous Ready/frame progress, idle object policy, zero render blocks, and complete cleanup.

## Controlled Variables

- The boundary child, cooked sources, one-region playable rectangle, readiness, actor, camera,
  traversal, long hold, and completion schema remain unchanged.
- The existing five product boundary tests remain the pure admission/overflow authority.
- The native helper retains its 1,000 ms per-delay bound; the 15-second hold remains outside it.
- No intermediate product state, position polling, retry, second process, relaxed boundary,
  compatibility alias, or replacement report is added.

## Derivation

The pre-slide endpoint is one of the prior accepted 64-Q9 points in `[-4096,-3648]`. From the most
inward endpoint, a `(-45,-45)` Run request can move both axes for at most nine total steps across
any 1..=8 simulation-batch partition before the next eight-step Z candidate leaves the one-region
rectangle. Therefore a final negative-X displacement of at least 16 exact 45-Q9 steps proves at
least seven committed steps for which Z was rejected while X remained admitted.

## Metrics

- Exact PID/window/thread identity; native message order and intervals; boundary and slide holds;
  final actor position, motion, presentation, and epoch; total/coupled/tangential-only step bounds;
  clock/frame/object state; output/exit/stderr/trailing-output shape; workflow duration/report
  bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- Initial Shift/W must remain one exact-window atomic batch after readiness with a 0..=50 ms
  interval/span and a hold of at least 15,000 ms.
- The second action must target the same PID/window and record
  `A-down, A-up, W-up, Shift-up, Escape`; A must remain held for 500..=1,000 ms, both adjacent
  release intervals must be 0..=50 ms, and Escape must follow after 250..=750 ms.
- Final X must be a negative 45-Q9 multiple encoding 16..=48 Run steps. Because no more than nine
  can remain coupled to Z, the derived tangential-only count must be at least seven.
- Final Z must remain in inclusive `[-4096,-3648]`; actor identity/region/shape must remain stable,
  vertical velocity must be zero, and final presentation must be Survey clip 0/yaw 32,768 with a
  later epoch.
- Ready/sample/live-frame counts must advance without new reset/suspend/resume/stall counts, render
  blocks, or object state.
- Completion must report Escape, exit zero, exactly two values, empty stderr/trailing output, and
  no lingering process.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count
  diffs must remain empty.

## Results

`canonical-prototype-v66` passed in 170.657 seconds with a 457,402-byte report. PID 2188 received
atomic Shift/W on window thread 27276 and window `31526174`; the interval and total batch span were
both 0.0013 ms. The boundary hold lasted 15,012.6987 ms. The same window then received A-down,
A-up, W-up, and Shift-up with intervals `506.4753/2.3234/0.0528` ms, followed by Escape after
263.6341 ms.

The actor retained generation 1, its signed region, 65,536 half-height numerator, and zero vertical
velocity. It finished at local `(-1395,-3738)` Q9: X proves 31 exact 45-Q9 Run steps, while the
nine-step maximum coupled bound proves at least 22 committed tangential-only steps after Z was
reduced. Completion retained Survey clip 0/yaw 32,768 and advanced the presentation epoch from 1
to 1,065.

The clock reached Ready/sample `1079/1080`; all 1,080 live frames and camera anchors completed with
zero stalls, suspends, resumes, render blocks, or object feedback/action/suppression. The process
returned Escape, exit zero, exactly two values, empty stderr, and empty trailing output. All other
16 normal sessions retained the same two-value/exit-zero/empty-output contract, and no process
remained.

The first v66 run rejected the new completion after all focused tests and prior sessions passed.
Its oracle incorrectly required final Z to remain divisible by the original 64-Q9 cardinal Run
component. The real boundary policy can admit up to nine initial `(-45,-45)` steps before reducing
Z, so final `-3738` is valid. The corrected gate preserves the exact boundary band and derives the
stronger minimum tangential-only count instead of imposing a false lattice.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard` passed
with zero Flavor denies and five existing warnings. No product Rust, Runtime, renderer/GPU, source,
resource, synchronization, schema, or process-count change was made.

## Conclusion

Accepted. The existing real boundary process now proves independent maximum-eight-step per-axis
admission: after the forward axis becomes unsafe, the exact tangential Run component remains live
for at least seven and observed 22 committed steps, then releases to retained tangential-facing
Survey without another process or product surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
