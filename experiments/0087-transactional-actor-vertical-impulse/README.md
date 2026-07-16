# Experiment 0087: Transactional Actor Vertical Impulse

Status: Accepted

## Hypothesis

The sole actor simulation command can carry one batch-entry vertical step-velocity delta that is
applied exactly once before the first emitted fixed step and participates in the existing atomic
schedule/actor/render-admission transaction, without introducing jump policy, retained intent, an
independent actor mutation route, or a second simulation path.

## Scope

Replace the strict `ActorSimulationCommand` and workbench payload in place with one required
`initial_step_velocity_delta_q16`. When a prepared schedule emits one or more steps, checked-add
that value to the retained `TerrainBodyMotion` step velocity before the first existing planar-first
advance. Later steps use the resulting retained velocity without reapplying the delta.

A zero-step transaction applies no delta and preserves the complete actor. The command is not
retained: a caller that still wants the transition after a fractional advance or render block must
submit it again. The prototype supplies zero and changes no product behavior.

Do not add grounded eligibility, a jump verb, input buffering, coyote time, repeated-action policy,
impulse tuning, jump presentation, horizontal dynamics, actor mutation outside the sole
transaction, compatibility defaults, an inspect alias, renderer/GPU work, or Wulin behavior.

## Workload

1. Prove zero-step identity, a positive one-step departure, and exact ordering before the existing
   per-step acceleration in private motion/simulation tests.
2. Prove a nonzero multi-step batch applies the delta only at batch entry and that its continuation
   with zero delta is exact.
3. Prove checked-add overflow fails before any terrain query and exposes no output.
4. Replace every live command construction and strict inspect request in place. Unchanged callers
   explicitly send zero; missing or unknown fields remain invalid.
5. Extend the maintained actor workflow with one successful positive departure and a pending-window
   candidate carrying a nonzero delta. Require exact actor/schedule rollback after the typed block.
6. Run focused Rust and TypeScript checks, `runseal :canonical-actor`,
   `runseal :canonical-prototype`, `runseal :init`, and `runseal :guard`.

## Controlled Variables

- Rational 60 Hz scheduling, the 0..=8 step bound, planar-first translation/contact ordering,
  per-step acceleration, presentation validation, animation epoch selection, and generation handle
  remain unchanged.
- The delta is checked in signed Q16 before the first query of a nonzero batch. Existing vertical
  integration then adds the command's per-step acceleration exactly as before.
- Actor and schedule candidates remain copies until published/pending render-window preflight
  succeeds. Published-window failures remain fatal; pending-window-only backpressure remains typed.
- The inspect route remains diagnostic transport for the same runtime transaction, not an
  additional authority. Its response may expose existing input/output state only.

## Metrics

- Exact input/output step velocity and center height for zero-, one-, and multi-step batches.
- Terrain query count at success, overflow, and pending-window block.
- Schedule tick/remainder/advance counts and complete retained actor identity before and after
  failure or block.
- Strict payload rejection, focused test counts, maintained workflow duration, Flavor results, and
  implementation ownership diff.

## Acceptance Criteria

- Zero emitted steps preserve the complete actor and perform zero terrain queries even when the
  submitted delta is nonzero.
- The first emitted step observes exactly one checked velocity delta before existing acceleration
  and integration. Further steps do not reapply it.
- Batch-entry overflow performs zero terrain queries and commits neither schedule nor actor.
- A nonzero-delta candidate blocked by a non-prefetch pending window reports prepared work but
  preserves the complete retained actor, schedule, pending composition, and retained frame.
- Missing `initial_step_velocity_delta_q16`, the retired payload shape, and unknown aliases are
  rejected by the strict live protocol. No optional/default/compatibility path remains.
- Prototype zero-delta behavior, the sole renderer/GPU actor path, resources, synchronization,
  source formats, assets, and Wulin ownership remain unchanged.
- Focused checks and maintained actor/prototype/init/guard workflows pass. The long canonical
  runtime workflow is not required if renderer/GPU/resource/synchronization/lifecycle code does not
  change.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference workbench, finite canonical
fixtures, existing rational schedule, retained actor, exact terrain query/contact, and typed render
admission path.

## Evidence

Focused engine-runtime tests pass 81 cases. New private cases prove that a zero-step batch ignores a
submitted delta, one emitted step applies `+1000` before `-100` acceleration to produce velocity
`900` and center `65536 -> 66436`, and a three-step batch produces velocities `900/800/700` and
center `67936`. Splitting that workload into one delta-bearing step plus two zero-delta steps is
exact. `i32::MAX + 1` fails on batch step 1 before any query, and the prepared simulation schedule
copy remains unchanged.

The final typed-command `canonical-actor-v5` passed in 56,173.626 ms. The strict old request fails
with missing `initial_step_velocity_delta_q16`; `initial_velocity_delta_q16` fails as an unknown
alias. A
fractional request carrying delta `4096` prepared zero steps/queries and preserved the exact actor.
The admitted one-step process changed vertical velocity `0 -> 16384` and center height
`141824 -> 158208`, proving delta-before-integration through the live transaction. Its pending-window
witness carried delta `8192`, prepared one step and one terrain query, returned `render-blocked`,
and reported schedule/actor commits `0/0`; exact actor, schedule, pending composition, and retained
frame assertions passed. Existing GPU actor and animation-epoch gates remained exact.

`canonical-prototype-v12` passed in 81,959.918 ms with 81 engine-runtime, 18 prototype, and 20
reference-host tests. Stationary, restart, native-W, native-E orbit, Escape, and 15-second held-W
boundary processes all published driver revision v5 with explicit
`initialStepVelocityDeltaQ16: 0`; locomotion, Survey/Walk, facing, gravity, camera, traversal,
readiness, and zero normal-path render blocks remained exact.

Rust and Deno formatting/type checks passed. The guard correction loop first rejected an
eight-argument private motion function under clippy, then rejected two five-word evidence constants
under Flavor. The scalar projection is now one private typed `MotionBatchCommand`, the constants
remain local and concise, and no lint exception or public abstraction was added. `runseal :init`
and the final `runseal :guard` passed. The long
canonical runtime workflow was not run because no renderer, GPU resource, synchronization, source
lifetime, lifecycle, format, asset, or stage-seal ownership changed.

## Conclusion

Accepted. The sole actor command can apply one checked vertical velocity delta exactly once at the
entry of a nonzero prepared fixed-step batch. Zero-step work consumes nothing, and all existing
failure/backpressure paths preserve the atomic schedule/actor boundary. This is transaction
capability only; jump eligibility and intent policy remain unselected.

## Promotion

Promoted the required field into `ActorSimulationCommand`, the private motion batch, strict
workbench request, prototype zero-delta caller, and maintained actor/prototype evidence revisions.
No new verb, compatibility decoder, retained intent, actor mutation route, renderer/GPU path, or
product action was promoted.
