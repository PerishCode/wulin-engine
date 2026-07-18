# Experiment 0152: Native Boundary Held-State Readmission

Status: Accepted

## Hypothesis

The finite-boundary tangential phase must declare every held key it depends on at its own native
helper boundary. Atomically reasserting Shift/W with A on the exact window thread should eliminate
the implicit cross-helper held-state dependency while preserving the accepted 45-Q9 tangential
Run result, process count, product behavior, and completion contract.

## Scope

- Retain the existing exact-PID boundary child, initial atomic Shift/W, 15,000 ms hold, 500 ms
  tangential phase, 250 ms stationary phase, and sequence-2 Escape completion.
- Change the second native action from A-down alone to one atomic Shift-down/W-down/A-down prefix.
- Require the second prefix to target the same PID/window/thread with a 0..=50 ms span before its
  delayed A/W/Shift releases.
- Preserve the existing endpoint, tangential-step, presentation, clock/frame, object-idle, and
  process-cleanup gates.
- Advance canonical Prototype acceptance from v66 to v67.

Product input normalization, locomotion, boundary policy, presentation, Runtime, renderer/GPU
resources, source formats, synchronization, output schema, process count, compatibility, retry,
and telemetry are out of scope.

## Workload

1. Launch the existing one-region boundary child and consume idle readiness.
2. Atomically post Shift-down/W-down and hold for at least 15 seconds.
3. On the same PID/window, atomically post Shift-down/W-down/A-down while the window thread is
   suspended, hold 500 ms, then release A/W/Shift and post Escape after 250 ms.
4. Require final X to remain a negative 45-Q9 multiple encoding 16..=48 Run steps and final Z to
   remain in inclusive `[-4096,-3648]`.
5. Require at least seven derived tangential-only steps, final Survey/yaw 32,768, continuous
   clock/frame progress, idle object state, and complete two-value process teardown.

## Controlled Variables

- Repeated Shift/W downs are consumed by the existing fixed input normalization when those keys
  remain held and become fresh state-changing downs if an external focus lifetime cleared them.
- The same helper, schema-4 evidence, exact-window selection, thread suspension, delay bounds, and
  key-release order remain authoritative.
- No intermediate product state, polling, retry, alternate endpoint, second process, product
  mutation, alias, fallback, or replacement report is added.

## Metrics

- Exact PID/window/thread identity; both atomic spans; initial and tangential holds; release/Escape
  intervals; final actor position/presentation/epoch; exact and derived tangential step counts;
  clock/frame/object state; output/exit/stderr/trailing-output shape; workflow duration/report
  bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- The initial Shift/W and second Shift/W/A prefixes must target the same visible PID/window/thread,
  use atomic prefix lengths two and three respectively, and each span no more than 50 ms.
- The second prefix must be followed by A-up after 500..=1,000 ms, W-up and Shift-up with adjacent
  0..=50 ms intervals, then Escape after 250..=750 ms.
- Final X must encode 16..=48 exact negative 45-Q9 steps; subtracting the maximum-nine-step
  coupled prefix must leave at least seven tangential-only steps. Final Z must remain in inclusive
  `[-4096,-3648]`.
- Actor identity/region/shape must remain stable with zero vertical velocity; final presentation
  must be Survey clip 0/yaw 32,768 with a later epoch.
- Ready/sample/live-frame counts must advance without new stalls, resets, suspends, resumes,
  render blocks, or object effects.
- Completion must report Escape, exit zero, exactly two values, empty stderr/trailing output, and
  no lingering process.
- Product, Runtime, renderer/GPU, source, resource, synchronization, schema, and process-count
  diffs must remain empty.

## Results

The first v67 full attempt passed every focused test and earlier native session, then rejected the
settled boundary action at final local `(-960,-3648)` Q9. X was on a cardinal lattice rather than
the required 45-Q9 diagonal-component lattice, proving that the second helper had relied on W
remaining implicitly held from the first helper. An isolated replay of the old action could still
finish at `(-1350,-3738)`, confirming that the result depended on external held-state lifetime.

The corrected clean-commit `canonical-prototype-v67` passed in 168.853 seconds with a
458,350-byte report. PID 21676 used window `37227806` and thread 32552. Initial Shift/W spanned
0.0011 ms and remained held for 15,010.0479 ms. The same thread received the atomic Shift/W/A
reassertion in 0.0022 ms; its key intervals were
`0.0012/0.0010/514.6863/0.1655/0.0547` ms, followed by Escape after 262.1643 ms.

The actor retained generation 1, region, shape, and zero vertical velocity. It finished at local
`(-1395,-3738)` Q9: 31 exact negative-X 45-Q9 steps minus the maximum-nine-step coupled prefix
prove at least 22 tangential-only commits. Completion retained Survey clip 0/yaw 32,768 and
advanced the animation epoch from 1 to 1,063.

The clock reached Ready/sample `1077/1078`; all 1,078 live frames completed with zero stalls,
render blocks, or object effects. The process returned Escape, exit zero, exactly two values, empty
stderr, and empty trailing output. All 103 engine-runtime, 48 Prototype, and 20 reference-host
tests passed. `runseal :guard` passed with zero Flavor denies and five existing warnings. The exact
validation ran from a detached clean worktree containing only the four acceptance changes.

## Conclusion

Accepted. Each native helper now declares the complete held state needed by its phase. The finite
boundary proof no longer depends on Shift/W surviving an external helper lifetime, while the
existing product, process, endpoint, tangential-admission, and graceful-completion contracts remain
unchanged.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
