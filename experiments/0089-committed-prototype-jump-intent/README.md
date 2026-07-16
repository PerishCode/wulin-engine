# Experiment 0089: Committed Prototype Jump Intent

Status: Accepted

## Hypothesis

The prototype can compose sample-scoped Space presses, its last exact committed grounded witness,
the existing batch-entry vertical velocity delta, explicit host-time outcomes, and the sole actor
transaction through one capacity-one application policy: an eligible intent remains pending across
fractional work, stalls, and typed render backpressure, and is consumed only by a successful
nonzero actor commit, without adding engine input state, an action queue, or a second simulation
path.

## Scope

Add one private prototype Jump policy with two bits of live state: pending intent and the last
committed grounded value. Initialize grounded true from the existing exact grounded spawn.

Space press while grounded sets pending. Pending commands a fixed
`initial_step_velocity_delta_q16` of `4369`, the nearest integer fixed-step displacement for 4 m/s
at 60 Hz. A successful zero-step advance retains pending. Typed render block and Stalled host sample
retain it without retaining elapsed time. A successful nonzero advance clears pending and replaces
grounded from `last_step_grounded`. Reset or Suspended clears pending; mid-air presses are ignored.

Use the existing Survey/Walk presentation policy. The source has no selected jump clip, so do not
invent animation, blending, root motion, coyote time, input buffering beyond the one pending bit,
air control changes, configurable tuning, repeated airborne action, or gameplay effects.

## Workload

1. Prove grounded edge admission, capacity-one duplicate behavior, and mid-air rejection in focused
   policy tests.
2. Prove zero-step and typed-block retention, nonzero consumption, exact witness update, landing
   re-eligibility, Reset/Suspended cancellation, Stalled retention, and invalid observation rollback.
3. Extend the native input helper with visible-window Space/VK 32 and no key-up requirement.
4. Add one focused real process that must publish delta 4369, a nonzero fixed-step commit, exact
   positive vertical velocity/center trajectory under existing gravity, final grounded false,
   pending false, and policy grounded false.
5. Preserve exact horizontal position, Survey presentation/epoch, actor identity, camera anchoring,
   traversal target, source behavior, frame ordering, and zero normal-path render blocks.
6. Require default/restart/W/E/Escape/15-second boundary processes to publish delta zero, pending
   false, grounded true, and retain all previous invariants.
7. Run focused Rust/Deno checks, `runseal :canonical-prototype`, `runseal :init`, and
   `runseal :guard`.

## Controlled Variables

- Host message normalization and activation-before-sample ordering remain unchanged. Prototype
  applies Reset/Suspended cancellation before admitting the current message batch's Space edge, so
  a focus transition clears only old intent and cannot erase the new action from that same batch.
- Press-edge lifetime, rational schedule, gravity `-179`, impulse application, terrain contact,
  playable bounds, presentation, camera, traversal, and render admission remain unchanged.
- Policy observes an actor advance only after runtime success. A blocked outcome has no committed
  witness and therefore cannot consume pending or update grounded.
- Reset/Suspended cancellation prevents an action from crossing lost elapsed/focus continuity.
  Stalled retains only the boolean intent, never the discarded elapsed backlog.
- The one-time readiness policy record is acceptance evidence, not an inspect route, input history,
  or recurring telemetry surface.

## Metrics

- Pending/grounded state after every edge, host sample, zero/nonzero advance, block, and landing.
- Native process/window/key identity and visible-window admission.
- Exact command delta, step count, output velocity/center, grounded witness, actor/presentation/
  camera/traversal/frame state, and render-block count.
- Focused test counts, maintained workflow duration, Flavor results, and implementation ownership
  diff.

## Acceptance Criteria

- Exactly one grounded Space press can be pending. Held/repeat/duplicate or mid-air presses cannot
  add another intent.
- Ready zero-step, Stalled, and typed render-block outcomes retain pending without retaining elapsed
  time. Reset and Suspended clear it. Invalid advance evidence changes no policy state.
- The first successful nonzero commit carrying pending applies delta 4369 once, clears pending, and
  updates policy grounded from the exact false output witness. A later exact true landing permits a
  new press.
- A visible-window native Space process has unchanged XZ, Survey presentation, exact positive
  vertical trajectory, following camera Y, unchanged traversal, and zero normal-path blocks.
- All older product witnesses retain delta zero and exact behavior. No engine API/input/action
  state, RuntimeActor flag, terrain query, compatibility path, alternate mode, jump animation,
  renderer/GPU/resource/synchronization/source/format/asset, or Wulin change is added.
- Focused tests and maintained prototype/init/guard workflows pass. `canonical-actor` and the long
  canonical runtime workflow are not required if their accepted engine/renderer boundaries are
  unchanged.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference host, normalized Space edge,
finite prototype sandbox, existing actor transaction, exact grounded witness, and actor-relative
camera path.

## Evidence

Focused checks pass 83 engine-runtime, 22 prototype, and 20 reference-host tests. Four Jump-policy
tests prove capacity-one grounded admission, mid-air rejection, zero-step and typed-block retention,
Stalled retention, Reset/Suspended cancellation, nonzero consumption, landing re-eligibility, and
invalid-witness rollback. Rust formatting and Deno formatting/type checks pass.

The first native-process attempts exposed one ordering defect: the visible-window helper posts
focus activation before Space, but the loop initially admitted the edge before applying the clock
Reset, so a new action was cleared as stale intent. The final ordering observes the activation/time
sample first and then admits the current input edge. The focused cancellation test and final real
process prove that Reset still clears an older pending action while a post-reset edge begins a new
one.

Final `canonical-prototype-v14` passed in 67,470.114 ms. Its dedicated process proved that the
reported process/window identity matched a visible target receiving Space/VK 32. The first
committed command carried delta `4369`, gravity `-179`, zero XZ displacement, and Survey
presentation. One fixed step changed velocity `0 -> 4190` and center height `141824 -> 146014`,
exactly the expected rise `4190`; the transition reported one query and
`lastStepGrounded: false`. Actor identity, half-height, XZ, Survey clip/yaw/epoch,
camera anchoring, traversal desire, and zero render blocks remained exact. The policy published
pending false and grounded false after the commit.

Default, restart, native-W, native-E, Escape, and 15-second boundary processes retained delta zero,
pending false, grounded true, and all previous actor/presentation/camera/traversal/lifecycle
invariants. `runseal :init` and `runseal :guard` passed. The long canonical runtime workflow was not
run because engine, renderer, GPU resources, synchronization, source lifetime, lifecycle, formats,
assets, and stage ownership did not change.

## Conclusion

Accepted. Prototype now owns one bounded action intent whose eligibility comes only from committed
grounded evidence and whose consumption occurs only with the sole successful nonzero actor
transaction. Focus/time discontinuities clear old intent before current-batch input admission;
fractional work, stalls, and render backpressure retain one boolean action but no elapsed backlog.

## Promotion

Promoted one private prototype Jump policy, one fixed Space/VK 32 native-process witness, readiness
evidence, and strict maintained assertions. No engine action state, actor field, input queue,
terrain query, compatibility route, alternate simulation path, jump animation, renderer/GPU path,
config surface, asset, gameplay effect, or Wulin behavior was promoted.
