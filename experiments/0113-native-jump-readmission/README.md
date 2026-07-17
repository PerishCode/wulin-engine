# Experiment 0113: Native Prototype Jump Readmission

Status: Accepted

## Hypothesis

The live Prototype can complete one exact grounded Jump lifetime and admit a second native Space
press only after the first trajectory has returned the policy to grounded, without another terrain
query path, action queue, gameplay state, or product report field.

## Scope

- Start from one idle sequence-one readiness with the exact grounded actor and Jump policy.
- Post one visible-window Space down, wait at least 1,250 ms with no host stall, then post Space
  up/down to create one new normalized press edge.
- Exit through Escape before the second exact 48-step flight can land.
- Require the final actor to lie on the existing fixed `4369/-179` discrete vertical trajectory for
  a bounded nonzero second-flight step count, with unchanged X/Z, body shape, presentation, epoch,
  and identity.
- Preserve all existing Escape, window-close, focus-discontinuity, forced-silence, and sustained
  capacity-one session gates.

Changing Jump policy, grounded authority, simulation ordering, host time/input behavior, completion
schema, Runtime behavior, terrain queries, presentation, renderer/GPU resources, or synchronization
is out of scope.

## Workload

1. Add one exact Space release/press action to the maintained native window harness.
2. Add a bounded post-readiness session with a measured lower bound between the first and second
   Space postings and one same-helper monotonic interval from the second Space post through Escape.
3. Derive the second-flight step count from final exact vertical velocity and verify the complete
   discrete trajectory against readiness ground height.
4. Prove no focus discontinuity, host stall/backlog, render block, horizontal movement, object
   state, or existing session regression.

## Controlled Variables

- The existing Jump policy remains capacity one and grounded only by the final committed terrain
  witness.
- The fixed impulse is 4,369 Q16-per-step, gravity is -179 Q16-per-step, and landing remains the
  existing planar-first exact terrain contact.
- First Space remains held until the deliberate release/press; native repeat suppression is not
  bypassed.
- The process emits only existing readiness and completion values. Acceptance records only its
  own native actions and wall-time bounds.

## Metrics

- Exact class/title/PID/window and native Space message evidence; first-to-second lower wall-time
  bound; same-helper second-to-Escape interval; readiness/final actor motion, presentation, and
  identity; derived second-flight step/rise/velocity; clock counters; render/object state; output
  count; exit code; stderr; duration; and all prior session invariants.

## Acceptance Criteria

- The first-to-second posting interval is at least 1,250 ms, all admitted host samples remain
  non-stalled, and no suspension/reset occurs.
- Space up/down creates the second press edge; Escape is posted within 700 ms of the second
  Space-down post, bounding the second flight before its 48th landing step.
- Final vertical velocity uniquely derives an integer step count in `1..=43`; final center height
  equals `ground + 4369*n - 179*n*(n+1)/2` exactly and remains above ground.
- Actor handle, X/Z, half height, presentation, and animation epoch remain exact; object policies
  stay idle; render blocks remain zero; stdout remains exactly two values; exit is zero with empty
  stderr.
- Focused checks, `runseal :guard`, and `runseal :canonical-prototype` pass without product or
  engine/GPU/resource structural change.

## Results

`canonical-prototype-v30` passed in 100.135 seconds. The exact visible window for PID 2292 first
received `WM_SETFOCUS`, `WM_KEYDOWN:Space`. The measured lower bound before the second action was
1,265.727 ms with zero host stalls, beyond the existing 48-step flight. The same window then
received `WM_SETFOCUS`, `WM_KEYUP:Space`, `WM_KEYDOWN:Space`, and, after a same-helper monotonic
104.278 ms interval, `WM_KEYDOWN:Escape`.

Readiness at live frame/sample 4 reported grounded true, zero vertical velocity, ground center
141,824, and clock ready/reset/stall `3/1/0`. Completion at live frame/sample 1,616 retained reset
one and stall zero while Ready advanced to 1,615. Its final velocity 3,116 derives exactly seven
second-flight steps:

```text
4369 - 179 * 7 = 3116
4369 * 7 - 179 * 7 * 8 / 2 = 25571
141824 + 25571 = 167395
```

Actor generation, X/Z, half height, Survey presentation, yaw, and animation epoch remained exact.
Object policies stayed idle, render blocks remained zero, stdout contained exactly readiness plus
completion, exit code was zero, and stderr was empty. The focus gate retained 645 suspended samples;
the sustained capacity gate retained 12 Rejected and 1,051 suppression frames.

One split-helper attempt was deliberately rejected with final velocity/height `0/141824`: separate
Escape PowerShell startup could not provide a usable second-flight upper bound. Keeping the
re-press, monotonic delay, and Escape post in one maintained helper removed that host-process
latency from the evidence.

Two warm-cache preflight reruns also exposed the older 10 ms visible-window polling interval racing
the first nonzero Run readiness. Tightening only the maintained harness poll to 1 ms preserved the
exact visible class/title/PID requirement and restored every existing startup-input gate without a
product change.

## Conclusion

Accepted. A real Prototype process now proves one complete Jump landing and exact second native
press readmission through existing grounded/contact authority. No product policy, report schema,
terrain query, action state/history, Runtime behavior, or engine/GPU/resource structure changed.

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
