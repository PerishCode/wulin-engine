# Experiment 0123: Native Run Modifier Release

Status: Accepted

## Hypothesis

After one ordered native sequence commits Shift-down plus W-down as forward Run at readiness, a
later Shift-up clears only the gait modifier. W remains held, so completion must retain negative-Z
locomotion while transitioning from imported Run clip 2 to Walk clip 1.

## Scope

- Start one real Prototype process and one exact visible-window native sequence.
- Post Shift-down and W-down, hold them for at least 500 ms, release only Shift, then post Escape
  after at least another 200 ms.
- Require readiness to expose exact forward Run presentation and default orbit.
- Prove final negative-Z Walk/yaw 49,152 and a later animation epoch.
- Preserve every existing input, session, object, restart, failure, and lifecycle gate.

Changing `HostInput`, Win32 capture, gait/camera/locomotion policy, product reports, Runtime
behavior, terrain/traversal ownership, renderer/GPU resources, synchronization, or sources is out
of scope.

## Workload

1. Add one acceptance-only Shift+W, delayed Shift-up, and delayed-Escape sequence.
2. Read the existing readiness value while that one native sequence remains in flight, then await
   its release/exit evidence and the existing completion value.
3. Validate exact process/window native evidence, readiness Run, final Walk, retained forward
   direction, actor identity/shape, clock continuity, idle object state, and zero render block.
4. Add a focused bounded invariant owner, structural guards, report revision, init ownership, and
   live documentation.
5. Run Deno formatting/type checks, focused Rust tests, `runseal :guard`, `runseal :init`, and
   `runseal :canonical-prototype`.

## Controlled Variables

- Native message order is Shift-down, W-down, Shift-up, Escape. Readiness must be Run and
  completion must be Walk, so the fixed message queue order proves Shift-up is consumed between
  those two existing product values.
- W remains held until Escape, so locomotion direction and facing do not change.
- Total displacement may include legal Run and Walk steps. Exact final Walk presentation and the
  later epoch, rather than a timing-derived step split, are the transition oracle.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window, native messages, Run-hold and delayed-Escape intervals;
  readiness actor Run presentation/camera; final actor handle/region/shape/velocity, Q9
  displacement, Walk presentation and epoch; clock/frame/object state; output count; exit code;
  stderr; workflow duration; and all prior Prototype gates.

## Acceptance Criteria

- One visible PID/window receives exactly `WM_SETFOCUS, WM_KEYDOWN:Shift, WM_KEYDOWN:W,
  WM_KEYUP:Shift, WM_KEYDOWN:Escape`; Shift-up follows W-down by 500..=1,000 ms and Escape follows
  Shift-up by 200..=700 ms.
- Readiness commits imported Run clip 2/yaw 49,152 at orbit 0.
- Final X delta is zero and Z delta is negative, nonzero, and divisible by 32 Q9.
- Final presentation is imported Walk clip 1 at yaw 49,152 with a later epoch. Actor handle,
  region, half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

- Flavor first rejected an eleventh source child under `.runseal/support/prototype`. The new
  session oracle moved under the existing `sessions/` owner; the rule was not suppressed or
  widened.
- The first full v39 run exposed a pre-existing focus-discontinuity helper race: W-down and
  focus-loss could be sampled separately. The helper now queues focus, W-down, and focus-loss while
  the exact window thread is suspended, and the unchanged-actor focus gate remains strict.
- A second full run diagnosed the initial two-helper Run-release design reaching the single-region
  finite boundary at local Z `-3904` before the second PowerShell helper could release Shift. Raw
  actor evidence showed legal Survey at the boundary, not a gait-policy failure. One concurrent
  ordered helper removed that launch-time contamination while preserving the two-value product
  protocol.
- Focused real-process acceptance then passed for PID 11624: W-down to Shift-up was 517.0246 ms,
  Escape followed 206.0922 ms later, readiness was Run clip 2/yaw 49,152, completion was Walk clip
  1/yaw 49,152, epoch advanced `3 -> 19`, and the actor moved `-2144 Q9` in Z with zero X delta and
  zero render blocks.
- Final `canonical-prototype-v39` passed in 142.711 seconds. PID 22072 recorded 4.6121 ms between
  Shift-down/W-down, 505.9149 ms from W-down to Shift-up, and 208.2274 ms from Shift-up to Escape.
  Readiness committed 3 Run steps at local Z `-192`; completion retained the same actor/region at
  local Z `-2304`, Walk clip 1/yaw 49,152, and epoch `3 -> 19`. Total delta was `-2112 Q9`
  (66 exact 32-Q9 units); clocks advanced `2/3 -> 23/24` with reset/suspend/resume/stall
  `1/0/0/0`, object state idle, exactly two values, exit zero, and empty stderr.
- `runseal :guard` and `runseal :init` passed. Flavor retained zero deny issues and the five
  pre-existing warnings. No Rust product, Runtime, traversal, renderer, GPU resource, source, or
  synchronization code changed.

## Conclusion

Accepted. The live Prototype proves held Shift is only a current gait modifier: releasing Shift
transitions the sole retained actor from Run to Walk while held W, forward facing, actor identity,
clock continuity, and the existing two-value session contract remain exact. The incidental native
focus-helper repair strengthens an already accepted gate without changing product behavior.

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
