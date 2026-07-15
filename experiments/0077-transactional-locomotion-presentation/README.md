# Experiment 0077: Transactional Locomotion Presentation

Status: Accepted

## Hypothesis

The sole runtime actor simulation transaction can accept one typed motion/presentation command and
commit the desired imported animation clip with the corresponding fixed-step motion, while
preserving exact fractional, failure, and pending-window rollback. The prototype can then use
Survey while stationary and Walk while moving without a separate presentation mutation path.

## Scope

Replace the scalar `Runtime::advance_simulation_actor` command arguments with one
`ActorSimulationCommand` containing horizontal displacement, step-up bound, vertical acceleration,
and a complete validated `ActorPresentation`. Replace the motion-only retained-slot commit with one
complete actor-state replacement after the existing render admission point.

Presentation changes only when the prepared schedule emits at least one fixed step. Zero-step
fractional calls still commit schedule remainder while leaving the complete actor unchanged.
Invalid presentation, terrain/motion failure, active-window failure, and typed pending-window block
commit neither actor nor schedule.

The prototype initially authors imported Survey clip 0. Its command policy selects Survey for zero
horizontal displacement and Walk clip 1 for every nonzero cardinal or diagonal displacement, while
keeping imported archetype 7, material 63, yaw 0, phase offset 0, and variation 0 fixed.

Directional yaw, Run selection, animation blending/transitions, root motion, presentation-clock
changes, new actor storage, input remapping, traversal/prefetch changes, new inspect verbs, renderer
or shader changes, and Wulin content are out of scope.

## Workload

1. Prove complete retained actor replacement validates desired presentation, preserves generation,
   and rolls back invalid/stale candidates.
2. Replace the existing `simulation.actor.advance` payload in place with the complete command and
   replace its response revision; retain no optional legacy payload or parser fallback.
3. In the focused actor gate, commit Survey-to-Walk with one admitted step, then request
   Walk-to-Survey on a known pending-window block and require byte-exact actor, schedule, pending
   pair, and retained-frame rollback.
4. Require fractional elapsed and all existing partition/motion failures to preserve presentation
   and existing deterministic schedule behavior.
5. In direct prototype processes, require stationary input/output/current clip 0. Under the
   process-qualified native W input, require input clip 0 and committed output/current clip 1 with
   the existing exact displacement, camera follow, traversal target, zero render blocks, restart,
   failures, and Sidecar cleanup.

## Controlled Variables

- Runtime simulation frequency, elapsed admission, planar-first terrain motion, query count,
  candidate render preflight, commit order, and typed pending backpressure remain unchanged.
- Presentation uses the existing schema-3 record and renderer/GPU actor path. No palette, descriptor,
  resource, pass, barrier, fence, wait, or presentation-clock owner changes.
- Clip selection is a prototype policy over the already normalized command; the runtime does not
  infer gameplay state from displacement.
- A command with zero emitted simulation steps cannot change actor presentation.
- Existing workbench verb and response names are replaced in place rather than versioned beside old
  forms.

## Metrics

- Actor input/output generation, motion, and presentation equality or transition.
- Prepared step/query, schedule commit, actor commit, and presentation mutation counts.
- Fractional, invalid-presentation, terrain/motion, active-window, and pending-window rollback.
- Stationary/native-W clip, displacement, camera/frame, traversal, block, restart, and cleanup
  evidence.
- Engine API, renderer/shader/GPU resource, synchronization, format, and inspect-verb deltas.

## Acceptance Criteria

- One admitted nonzero-step command atomically commits motion and the exact desired presentation;
  generation is unchanged and mutation count is one only when presentation differs.
- Zero-step commands preserve the full actor. Every rejected or typed-blocked command preserves the
  full actor and schedule; pending composition and retained rendering remain exact.
- Stationary prototype processes use Survey clip 0 throughout. Native W changes Survey to Walk clip
  1 in the same committed transaction that moves the actor, with no yaw/material/archetype/phase/
  variant changes and no render block.
- Focused actor and prototype workflows plus `runseal :guard` pass. No old Runtime signature,
  motion-only commit type/name, schema-2 response, optional legacy payload, fallback, second
  presentation mutation, inspect verb, renderer/shader/GPU resource, synchronization, or format
  path remains.

## Reference Environment

The experiment uses the pinned Rust/Deno toolchains, reference Windows host, existing signed
canonical sources, capacity-one imported-Fox actor, fixed W/A/S/D command, and sole renderer.

## Evidence

- `runseal :canonical-actor` passed as `canonical-actor-v3` in 30.932 seconds. Invalid archetype 8
  failed before schedule/query work; a zero-step Survey actor carrying desired Walk committed only
  the schedule remainder; the admitted shared-window step changed local X `0 -> 1` and clip
  `0 -> 1`; the following desired Survey pending-window block reported step/query `1/1`, schedule/
  actor/presentation mutations `0/0/0`, and retained clip 1, schedule, pending pair, and frame.
- The same actor workflow passed the existing frame-safe GPU admission, replay, clear, rejection,
  despawn/respawn, and capture invariants. No GPU resource or execution expectation changed.
- `runseal :canonical-prototype` passed as `canonical-prototype-v6` in 38.080 seconds. Direct
  processes 6272 and 11564 reported input/output/current animation 0, zero displacement, zero render
  blocks, and anchor/frame `3/3`. Process 33244 reported input animation 0, output/current animation
  1, exact W displacement `-32` Q9 in one step/query, zero render blocks, and anchor/frame `3/3`.
- All three prototype processes preserved the exact traversal token/target invariants, bootstrap
  failures, restart equality, native input qualification, Sidecar restart/stop, and final cleanup.
- Focused tests passed 75 engine-runtime, 11 prototype, and 21 reference-host cases. Strict clippy,
  Deno formatting/type checking, `runseal :init`, and `runseal :guard` passed; guard reported zero
  Flavor denies and the five existing warnings.
- Two exploratory actor-gate runs failed only because maintained support first compared request
  `yaw_q16` against status `yawQ16`, then treated JSON property order as semantic. The final support
  uses typed field invariants; production transaction behavior was unchanged by those corrections.

## Conclusion

The hypothesis is accepted. Motion and desired presentation now enter one typed actor-simulation
command, and the complete actor is admitted and committed with its schedule after nonzero fixed-step
preparation. Fractional commands and every failure/block retain the prior presentation. The plain
prototype therefore presents Survey while stationary and changes to Walk in the same transaction
that moves it. No scalar compatibility signature, motion-only commit type, schema-2 response,
optional legacy payload, fallback, second presentation mutation, new inspect verb, animation blend,
directional yaw, renderer/shader/GPU resource, synchronization, or format path remains.
