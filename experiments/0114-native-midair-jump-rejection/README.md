# Experiment 0114: Native Midair Jump Rejection

Status: Accepted

## Hypothesis

The live Prototype ignores a second normalized Space press while its exact committed grounded
witness is false, so the final actor remains on the original discrete Jump trajectory with no
second impulse, queue, coyote state, or product report field.

## Scope

- Start from one idle sequence-one readiness with the exact grounded actor and Jump policy.
- In one exact-window helper, post Space down/up, wait about 200 ms, post Space down plus W while
  the first flight is airborne, wait about 200 ms, then post Escape.
- Measure the first-to-second Space and second-Space-to-Escape `PostMessageW` return intervals with
  one monotonic clock.
- Require final motion to lie on the single existing `4369/-179` trajectory for a bounded nonzero
  step count. A wrongly applied second impulse must be arithmetically impossible to disguise.
- Require W displacement and Walk presentation from the same midair input batch, proving the
  second Space reached product admission rather than remaining unprocessed before Escape.
- Preserve all existing Escape, window-close, focus, Jump-readmission, forced-silence, and
  sustained capacity-one session gates.

Changing Jump policy, grounded authority, input/time ordering, session schema, Runtime behavior,
terrain queries, presentation, renderer/GPU resources, or synchronization is out of scope.

## Workload

1. Extend the maintained native action helper with one optional delayed Escape post and exact
   first/last-key span evidence.
2. Add one bounded post-readiness midair re-press session using only that helper.
3. Derive final single-flight step count from vertical velocity and verify exact height against the
   readiness ground body.
4. Prove the exact forward witness plus no focus discontinuity, host stall/backlog, render block,
   object state, or existing session regression.

## Controlled Variables

- The first Space release changes only held input; the original press edge remains admitted.
- The second Space press occurs before the exact 48th landing step.
- The fixed impulse remains 4,369 Q16-per-step and gravity remains -179 Q16-per-step.
- Product output remains the existing readiness/completion pair. Acceptance retains only native
  action and bounded timing evidence.

## Metrics

- Exact class/title/PID/window and native message order; first-to-second and second-to-Escape
  intervals; readiness/final actor motion/presentation/identity; derived single-flight
  step/rise/velocity; clock counters; render/object state; output count; exit code; stderr;
  duration; and all prior session invariants.

## Acceptance Criteria

- Both native intervals are at least 200 ms and at most 700 ms, so the second press and completion
  both occur before the first flight's exact landing.
- Final velocity uniquely derives an integer single-flight step count in `1..=43`; final center
  equals `ground + 4369*n - 179*n*(n+1)/2` exactly and remains above ground.
- Actor handle, X, region, and half height remain exact; Z advances by a positive whole number of
  32-Q9 Walk steps, and final presentation is exact imported Walk/yaw 49,152 with a later epoch.
  Clock reset/suspend/resume/stall counts do not change; object policies stay idle; render blocks
  remain zero.
- Stdout remains exactly readiness plus completion, exit is zero, and stderr is empty.
- Focused checks, `runseal :guard`, and `runseal :canonical-prototype` pass without product or
  engine/GPU/resource structural change.

## Results

`canonical-prototype-v31` passed in 102.146 seconds. The exact visible window for PID 3548 received
`WM_SETFOCUS`, Space down/up, then the second Space down plus W, and finally Escape. The exact
first-to-second Space interval was 208.749 ms; the W-post-to-Escape interval was 207.008 ms.

Readiness at live frame/sample 5 reported grounded true, ground center 141,824, zero vertical
velocity, and clock ready/reset/stall `4/1/0`. Completion at live frame/sample 606 retained reset
one and stall zero while Ready advanced to 605. Final velocity -106 derives exactly 25 steps from
only the first impulse:

```text
4369 - 179 * 25 = -106
4369 * 25 - 179 * 25 * 26 / 2 = 51050
141824 + 51050 = 192874
```

The same admitted midair batch produced 12 exact Walk steps, Z delta -384 Q9, imported Walk clip 1,
yaw 49,152, and a later animation epoch. Actor generation, region, X, half height, and material
remained exact. Object policies stayed idle, render blocks remained zero, stdout contained exactly
readiness plus completion, exit code was zero, and stderr was empty.

All prior session gates remained exact under the replacement acceptance-only native action schema
3 with no schema-2 alias: Jump readmission retained step 7 with a 1,263.759 ms landing lower bound
and 119.904 ms exit interval; focus retained 628 suspended samples; sustained capacity retained 12
Rejected and 1,054 suppression frames.

## Conclusion

Accepted. A real product loop now proves that a second native Space press is ingested while
airborne yet adds no impulse; the exact single-flight arithmetic and same-batch W witness jointly
exclude an unprocessed-input false positive. No product policy/report schema, Runtime query,
renderer/GPU resource, or synchronization path changed.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained bounded Prototype session workflow.

## Reproduction

```powershell
cargo test --locked -p prototype -p reference-host
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :guard
runseal :canonical-prototype
```

Generated reports remain ignored under `out/captures/`.
