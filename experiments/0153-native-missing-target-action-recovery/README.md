# Experiment 0153: Native Missing-Target Action Recovery

Status: Accepted

## Hypothesis

The existing Activated object-focus child can prove that a committed `MissingTarget` action is
consumed exactly once and does not poison later exact-object interaction. After focus readmission,
one Enter press with no retained target should increment only the ineligible count; a later fresh
F/Enter action on the same PID/window must still produce the existing Activated acknowledgement,
consumption, exclusion, and suppression lifetime.

## Scope

- Reuse the existing exact-PID Activated object-focus process, stale F/Enter focus-loss batch,
  resume/reset proof, source fixture, stationary actor, acknowledgement slot, and Escape completion.
- After resume, post Enter-down alone, hold for at least 250 ms, then atomically post
  Enter-up/F-down/Enter-down before the existing 250 ms delayed Escape.
- Require exactly one ineligible and one committed action, 12 Activated frames, zero Rejected
  frames, at least one suppression frame, exact source-qualified consumption, and a cleared target.
- Split native missing-target/recovery input evidence into its own bounded object input-gate owner.
- Advance canonical Prototype acceptance from v67 to v68.

Product observation/interaction policy, Runtime, renderer/GPU resources, canonical object state,
source formats, synchronization, session schema, process count, and broader gameplay semantics are
out of scope.

## Workload

1. Launch the existing Activated child to idle readiness.
2. Atomically post F-down/Enter-down immediately before focus loss, wait 250 ms, resume, and wait
   another 250 ms so the existing cancellation/reset proof completes.
3. Post Enter-down with no target and wait at least 250 ms for one committed nonzero simulation
   action to consume it as `MissingTarget`.
4. On the same PID/window/thread, atomically post Enter-up/F-down/Enter-down and delay Escape by
   250 ms.
5. Consume the existing completion and require the exact final interaction, feedback, identity,
   focus/clock, frame, and process-lifetime state.

## Controlled Variables

- The initial cancellation batch, focus timings, object source/position/radius/facing, actor,
  camera, traversal, renderer feedback, session output, and child count remain unchanged.
- The missing-target press has no target acquisition, feedback candidate, retry, polling, or
  retained acceptance state.
- Releasing Enter in the recovery prefix creates a fresh normalized press edge in the same atomic
  batch as F acquisition.
- No product telemetry, queue, second acknowledgement timer, compatibility route, fallback,
  product mutation, or additional process is added.

## Metrics

- Exact PID/window/thread identity; stale-cancellation, missing-target, and recovery message order;
  atomic spans and hold/exit intervals; clock suspend/resume/reset counts; ineligible/committed
  counts; Activated/Rejected/object-target/suppression frames; source-qualified identity;
  observation/interaction final state; output/exit/stderr/trailing-output shape; workflow duration
  and report bytes; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- Initial F/Enter/focus-loss must remain one exact-window atomic batch with prefix length two and a
  0..=50 ms span; the same window must resume through exactly one suspend, resume, and post-resume
  reset.
- Enter-down with no target must use the same PID/window, carry no delayed exit or atomic prefix,
  remain held for at least 250 ms, and produce no feedback candidate.
- Recovery must atomically queue Enter-up/F-down/Enter-down on the same positive window thread with
  prefix length three, 0..=50 ms intervals/span, and Escape after 250..=750 ms.
- Completion must report `ineligibleCount=1`, `committedCount=1`, pending/acknowledgement null,
  observation target null, exactly 12 Activated and object-target frames, zero Rejected frames, and
  at least one suppression frame.
- Consumed and nearest-exclusion identities must exactly match the source namespace, owner region,
  and authored local ID selected by the existing independent source oracle.
- Actor/process identity must remain stable; clock/frame progress must have zero stalls/render
  blocks or elapsed backlog.
- Exit must be zero with exactly readiness plus completion and empty stderr/trailing output.
- Product, Runtime, renderer/GPU, source, synchronization, schema, and process-count diffs must
  remain empty.

## Results

`canonical-prototype-v68` passed on its first full run in 175.470 seconds with a 460,265-byte
report. PID 2580 used window `142084402` and thread 22988. The stale F/Enter/focus-loss prefix
spanned 0.0012 ms. After the exact resume/reset boundary, Enter-down with no target remained
admitted for 263.0453 ms without a feedback candidate.

The same thread then atomically queued Enter-up/F-down/Enter-down in 0.0021 ms with
`0.0011/0.0010` ms intervals; Escape followed after 267.9987 ms. Completion recorded exactly one
ineligible action before exactly one committed action. The latter produced 12 Activated and 12
object-target frames, zero Rejected frames, and two projected suppression frames.

The committed identity retained source namespace
`99d9511b8cea49a59d771d97874d56bb7790a79c880490353852bc75aa4fd94d`, owner region
`(1099511627776,-1099511627776)`, and authored local ID 496 in both consumption and nearest
exclusion. Final observation target and acknowledgement were null, pending was false, and the
stationary actor identity remained unchanged.

The clock recorded one suspend, one resume, two total resets, zero stalls, and Ready/sample
`260/343`; all 343 live frames had zero render blocks. The process returned Escape, exit zero,
exactly two values, empty stderr, and empty trailing output. All 103 engine-runtime, 48 Prototype,
and 20 reference-host tests passed. `runseal :guard` passed with zero Flavor denies and five
existing warnings. No product Rust, Runtime, renderer/GPU, source, synchronization, schema, or
process-count change was made.

## Conclusion

Accepted. A missing-target Enter intent is a one-shot ineligible commit, not a poisoned action
lifetime. The same exact process can immediately release that key, reacquire the canonical object,
and complete the existing source-qualified Activated/consumption/suppression lifecycle without new
product state or acceptance processes.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
