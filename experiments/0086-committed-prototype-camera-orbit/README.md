# Experiment 0086: Committed Prototype Camera Orbit

Status: Accepted

## Hypothesis

The prototype can compose sample-scoped Q/E press edges with the existing checked actor-relative
camera mutation through one application-owned candidate/commit policy: exactly one quarter-turn is
committed only after the runtime accepts the complete candidate camera, without adding an engine
input surface, another projection authority, or frame-rate-dependent camera state.

## Scope

Replace the prototype's single fixed camera constants with one private four-state quarter-orbit
policy. Each state is a complete actor-relative rig with the existing vertical offsets, field of
view, and XZ distance. Q requests one counter-clockwise step, E requests one clockwise step, and
opposite presses in the same input sample cancel.

The application prepares a rig from its committed orbit index, calls the existing
`Runtime::set_actor_relative_camera`, and updates its committed index only after that call succeeds.
Later empty ingests and held keys request no additional step. The runtime continues to own actor
projection and atomic scene-camera replacement.

Do not add pointer/gamepad transport, free-look, smoothing, interpolation, collision, zoom, pitch,
configuration, an engine camera controller, an inspect endpoint, a second camera mutation, or a
compatibility path. Locomotion, simulation, presentation, traversal, sources, renderer, GPU, and
Wulin behavior remain unchanged.

## Workload

1. Prove the four exact rigs, clockwise/counter-clockwise wrap, opposite-edge cancellation, and
   held/empty-sample non-repetition in focused prototype policy tests.
2. Prove the policy retains its prior committed index until its candidate is explicitly accepted;
   rejected/uncommitted candidates must not alter later candidates.
3. Extend the existing process-qualified native input helper with E. Wait for the exact prototype
   window to become visible before posting the key so the sample is consumed by the live loop, not
   the hidden bootstrap loop.
4. Extend only the focused source fixture with the rotated `[+1,-1]` center already present in the
   manual operator's finite sandbox. Missing rotated source must remain a real traversal failure.
5. In the focused prototype workflow, require one native-E process to publish orbit index 1, exact
   rotated rig/camera values, a stationary actor/simulation, the exact rotated traversal desire,
   bounded existing latest-wins scheduling, and no render backpressure. Default, restart, W,
   Escape, and finite-boundary cases retain their prior behavior.
6. Run focused Rust tests, TypeScript formatting/checking, `runseal :canonical-prototype`,
   `runseal :init`, and `runseal :guard`.

## Controlled Variables

- The default rig remains position `[9, 4, 12]`, target `[0, -1, -3]`, and vertical field of view
  `60`. Quarter turns rotate only XZ using exact integer-valued `f32` coordinates.
- Input sampling order, edge lifetime, held locomotion, Escape close behavior, host time admission,
  actor/schedule transaction, and camera-before-frame order remain unchanged.
- Camera anchoring continues through the sole generation-qualified runtime method and private actor
  projection. No global actor coordinate or camera anchor is exposed.
- Readiness reports the committed orbit index and exact rig used for its frame; it is not an
  inspect/control surface or recurring telemetry stream.

## Metrics

- Exact rig bit patterns and orbit index after every candidate/commit sequence.
- Native key/process/window identity and visible-window admission.
- Real-process actor motion/presentation, simulation command/advance, orbit rig/camera, exact
  camera-derived traversal desire and bounded queue/schedule state, frame/anchor counts, and
  render-block count.
- Focused test counts, workflow duration, Flavor results, and implementation ownership diff.

## Acceptance Criteria

- The four states form one exact closed quarter-turn cycle; inverse steps restore the prior state
  and four steps restore the initial state byte-for-byte.
- One accepted Q or E press edge changes the candidate by one state. Holding the key or ingesting an
  empty sample does not repeat it, and simultaneous Q/E edges leave the candidate unchanged.
- Application state changes only when the candidate is committed after successful runtime camera
  mutation. Preparing or dropping a candidate has no state effect.
- A native-E process publishes committed orbit index 1 with position offset `[12, 4, -9]`, target
  offset `[-3, -1, 0]`, and exact actor-anchored camera values. Its actor remains stationary in
  Survey. Traversal desires the corresponding `[+1,-1]` target and remains within the existing
  one-slot latest-wins state machine with no block, failure, prefetch, or rollover.
- Default, restart, W, Escape, and finite-boundary witnesses remain exact. No runtime/renderer/GPU/
  source/synchronization/format/asset change, input latch/queue, second camera path, or compatibility
  alias is added.
- Focused tests and the maintained prototype/init/guard workflows pass. The long canonical runtime
  workflow is not required if the implementation remains entirely in prototype policy/evidence.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference host, existing normalized
keyboard edges, finite prototype sandbox, and sole actor-relative runtime camera mutation.

## Evidence

Focused prototype policy tests pass 18 cases total, including three new camera cases. They prove all
four exact rigs, clockwise/counter-clockwise wrap, same-sample Q/E cancellation, held/empty-ingest
non-repetition, and that preparing or dropping a candidate cannot change committed policy state.

The first real-process attempt rejected the proposed claim that traversal stayed unchanged: the
committed rotated camera correctly produced a different camera-derived target. A five-process
timing sweep then exposed the two valid async forms at readiness—direct scheduling or replacement
of one in-flight initial target through the existing depth-one queue. No product timing special case
was added.

The next attempt exposed a focused-fixture gap. The real runtime reported one attempt, zero schedule,
and an exact missing-source failure because the fixture cooked only base, `[+1,+1]`, and corrupt
centers. The fixture now also cooks `[+1,-1]`; the manual 441-region sandbox was already sufficient,
and strict missing-source behavior remains unchanged. A final support defect was also corrected:
the traversal target helper derived global identity but had hard-coded local center `[65,65]`; it
now derives both local axes from the expected origin/center.

`canonical-prototype-v11` passed in 77,670.888 ms with 77 engine-runtime, 18 prototype, and 20
reference-host tests. Native process 22,580 received visible-window E / virtual key 69 in 3,518.163
ms and published camera revision v2, orbit index 1, rig `[12,4,-9] / [-3,-1,0]`, and exact anchored
camera `[12,6.1640625,-9] / [-3,1.1640625,0]`. It remained stationary in Survey with zero render
blocks. Traversal desired exact global center `[baseX+1,baseZ-1]`, local `[65,63]`, scheduled once,
and reported no failure, block, prefetch, queue, or rollover in the accepted run; the maintained gate
also accepts the observed valid depth-one latest-wins replacement state.

Default/restart/W/Escape/15-second finite-boundary processes, startup failures, source corruption,
Sidecar PID replacement, and zero-process cleanup all passed. The four-center fixture produced 65
deduplicated regions with zero edge mismatches. Deno formatting/type checking and Rust formatting
passed. `runseal :init` passed. The first guard exposed one new Flavor deny because camera evidence
pushed host orchestration to 524 lines; extracting the exact camera invariant into a 90-line owner
reduced host to 436 lines. The final guard passed with zero denies and the five existing warnings.

## Conclusion

Accepted. Prototype owns one four-state camera orbit policy whose Q/E candidates are edge-driven and
whose committed index advances only after the existing runtime camera mutation succeeds. The
runtime still owns projection and atomic camera replacement; its existing camera-driven traversal
observes the committed view through the existing bounded latest-wins machinery.

## Promotion

Promoted the policy into `apps/prototype/src/camera.rs`, orbit index/rig into the one-time readiness
record, exact camera evidence into a focused prototype-camera support owner, native visible-window E
plus rotated-target assertions into `canonical-prototype-v11`, and general expected-local-center
derivation into the maintained traversal support. No new operator, inspect route, engine API,
renderer path, or compatibility surface was promoted.
