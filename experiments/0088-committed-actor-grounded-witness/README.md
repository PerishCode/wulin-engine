# Experiment 0088: Committed Actor Grounded Witness

Status: Accepted

## Hypothesis

The sole actor transaction can publish the exact grounded result of its final committed fixed step
by carrying the value already computed by planar-first advance through the private motion batch and
`ActorStateTransition`, without storing contact policy in `RuntimeActor`, issuing another terrain
query, exposing a blocked candidate, or introducing jump behavior.

## Scope

Add `last_step_grounded: Option<bool>` to `MotionBatch` and `ActorStateTransition`. Zero emitted
steps produce `None`. A successful nonzero batch replaces the option after each existing
`TerrainBodyAdvance` and therefore returns the final step's exact `grounded` value.

Only `ActorSimulationOutcome::Advanced` contains the transition. A pending-window render block may
prepare a candidate but publishes no transition or grounded witness. Mid-batch and fatal failures
return no output. Runtime actor storage and `actor.read` remain exact motion/presentation authority
without a cached contact flag.

Replace prototype evidence that infers `groundedAfterBatch` from zero velocity with the committed
witness. Do not add Space input, an action latch, jump eligibility/tuning/presentation, coyote time,
contact hysteresis, a duplicate query, a new verb, a compatibility field, or Wulin behavior.

## Workload

1. Prove zero-step `None`, a positive departure `Some(false)`, and exact ground hold `Some(true)` in
   private motion-batch tests.
2. Prove a multi-step workload reports its last step rather than an any/all aggregate by using a
   trajectory whose grounded state changes within the batch.
3. Prove fractional actor simulation returns `None`, nonzero simulation carries the exact final
   value, partitioned state remains exact, and overflow/query failures expose no transition.
4. Replace the diagnostic response and maintained evidence revisions in place.
5. In the actor workflow, require false for the admitted upward impulse, true for existing grounded
   animation steps, null for fractional work, and no transition field for pending backpressure.
6. In the prototype workflow, require true for every readiness transaction and report that exact
   value instead of a hard-coded inference.
7. Run focused Rust/Deno checks, `runseal :canonical-actor`, `runseal :canonical-prototype`,
   `runseal :init`, and `runseal :guard`.

## Controlled Variables

- Existing terrain translation, vertical acceleration/contact, grounded definition, query reuse,
  0..=8 bound, schedule, initial velocity delta, presentation/epoch selection, and render preflight
  ordering remain unchanged.
- `None` means no fixed step produced a current contact result; it does not mean false or unknown
  terrain availability.
- The last-step value describes the committed output transition. It is not retained engine state,
  and caller policy owns whether/how to remember it between advances.
- Published-window failures remain fatal. Pending-window-only backpressure remains typed and cannot
  be mistaken for committed contact state.

## Metrics

- Step count, query count, final motion, and exact `lastStepGrounded` for zero, departure, landing,
  hold, multi-step transition, and partitioned workloads.
- Presence/absence of transition evidence for advanced, fractional, failed, and blocked outcomes.
- Prototype exact grounded witness across stationary, locomotion, camera, exit, and boundary
  processes.
- Focused test counts, maintained workflow durations, Flavor results, and implementation ownership
  diff.

## Acceptance Criteria

- Zero emitted steps serialize `lastStepGrounded: null`, preserve the complete actor, and perform
  zero queries.
- Every successful nonzero batch reports exactly the existing final planar-first step's grounded
  result. A batch that changes contact state proves last-step rather than any/all semantics.
- Overflow, query, published-window, and mid-batch failures expose no transition witness.
- A pending-window render block contains no `actorSimulationAdvance` or grounded field and commits
  neither actor nor schedule; prior committed actor/schedule/pending/retained-frame evidence remains
  exact.
- Prototype readiness uses `lastStepGrounded: true` as its sole grounded-after-batch proof. It does
  not infer contact from vertical velocity or make a second terrain query.
- No cached RuntimeActor contact, default/alias/old response revision, inspect verb, action intent,
  product jump, renderer/GPU/resource/synchronization/source/format/asset, or Wulin change is added.
- Focused checks and maintained actor/prototype/init/guard workflows pass. The long canonical
  runtime workflow is not required if renderer/GPU/resource/synchronization/lifecycle ownership is
  unchanged.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, Windows reference workbench, finite canonical
fixtures, existing exact terrain query/contact, rational schedule, retained actor transaction, and
typed render-admission path.

## Evidence

Focused engine-runtime tests pass 83 cases. Zero-step batches return `None` with exact motion and
zero queries. A positive velocity delta returns `Some(false)`. A controlled three-step trajectory
uses delta `200` and acceleration `-100`: it departs false, remains separated false, then lands
exactly on step three and returns `Some(true)` with the original body/velocity. That proves the
field is final-step replacement rather than any/all aggregation. Fractional actor preparation
returns `None`; one-step impulse and three-step landing simulation return false/true respectively;
overflow and mid-batch failure still expose no output.

Final `canonical-actor-v6` passed in 38,654.178 ms. The strict advanced response publishes
`lastStepGrounded: null` for the fractional zero-step transaction, false for the admitted velocity
delta `16384`, and true for the existing grounded Survey→Walk and same-clip yaw transactions. The
pending-window delta candidate prepared one step/query but returned no `actorSimulationAdvance` or
grounded field, committed actor/schedule `0/0`, and preserved exact actor, schedule, pending
composition, and retained frame. GPU actor and animation-epoch evidence remained exact.

`canonical-prototype-v13` passed in 67,343.185 ms with 83 engine-runtime, 18 prototype, and 20
reference-host tests. First, restart, native-W, native-E orbit, Escape, and 15-second held-W boundary
processes all published driver revision v6 and exact committed `lastStepGrounded: true`. The
maintained `groundedAfterBatch` report now carries that field directly instead of assigning a
constant from zero-velocity/body observations. Locomotion, presentation, facing, gravity, camera,
traversal, readiness, and zero normal-path render blocks remained exact.

Rust/Deno formatting and type checks passed. `runseal :init` and `runseal :guard` passed. The long
canonical runtime workflow was not run because renderer, GPU resource, synchronization, source
lifetime, lifecycle, format, asset, and stage-boundary ownership did not change.

## Conclusion

Accepted. Every successful nonzero actor transaction publishes the exact grounded result of its
final existing planar-first step; zero-step transactions publish no contact fact, and blocked or
failed candidates cannot leak speculative eligibility. The runtime stores no second contact state
and the prototype gains no action policy.

## Promotion

Promoted one private optional batch witness, one serialized actor-transition field, response/driver
revisions, and exact maintained actor/prototype assertions. No RuntimeActor field, query, verb,
compatibility schema, input latch, jump behavior, renderer/GPU path, or Wulin surface was promoted.
