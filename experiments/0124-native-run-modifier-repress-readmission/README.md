# Experiment 0124: Native Run Modifier Re-press Readmission

Status: Accepted

## Hypothesis

After native W-down commits forward Walk at readiness, one later native Shift-down selects the
existing Run modifier without clearing W. Completion must retain negative-Z locomotion while
transitioning from imported Walk clip 1 to Run clip 2.

## Scope

- Start one real Prototype process and one exact visible-window native sequence.
- Post W-down, hold Walk for at least 500 ms, press Shift, then post Escape after at least another
  200 ms.
- Require readiness to expose exact forward Walk presentation and default orbit.
- Prove final negative-Z Run/yaw 49,152 and a later animation epoch.
- Preserve every existing input, session, object, restart, failure, and lifecycle gate.

Changing `HostInput`, Win32 capture, gait/camera/locomotion policy, product reports, Runtime
behavior, terrain/traversal ownership, renderer/GPU resources, synchronization, or sources is out
of scope.

## Workload

1. Add one acceptance-only W-down, delayed Shift-down, and delayed-Escape sequence.
2. Read readiness while the native sequence remains in flight, then await its Shift/exit evidence
   and the existing completion value.
3. Validate exact process/window native evidence, readiness Walk, final Run, retained forward
   direction, actor identity/shape, clock continuity, idle object state, and zero render block.
4. Add one focused bounded invariant owner, structural guards, report revision, init ownership,
   and live documentation.
5. Run Deno formatting/type checks, focused Rust tests, `runseal :guard`, `runseal :init`, and
   `runseal :canonical-prototype`.

## Controlled Variables

- Native message order is W-down, Shift-down, Escape. Readiness must be Walk and completion must be
  Run, so the fixed queue order proves Shift-down is consumed between those two product values.
- W remains held until Escape, so locomotion direction and facing do not change.
- Total displacement may include legal Walk and Run steps. Exact final Run presentation and later
  epoch, rather than a timing-derived step split, are the transition oracle.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window, native messages, Walk-hold and delayed-Escape intervals;
  readiness actor Walk presentation/camera; final actor handle/region/shape/velocity, Q9
  displacement, Run presentation and epoch; clock/frame/object state; output count; exit code;
  stderr; workflow duration; and all prior Prototype gates.

## Acceptance Criteria

- One visible PID/window receives exactly `WM_SETFOCUS, WM_KEYDOWN:W, WM_KEYDOWN:Shift,
  WM_KEYDOWN:Escape`; Shift-down follows W-down by 500..=1,000 ms and Escape follows Shift-down by
  200..=700 ms.
- Readiness commits imported Walk clip 1/yaw 49,152 at orbit 0.
- Final X delta is zero and Z delta is negative, nonzero, and divisible by 32 Q9.
- Final presentation is imported Run clip 2 at yaw 49,152 with a later epoch. Actor handle, region,
  half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

- Focused real-process acceptance passed for PID 3076: W-down to Shift-down was 508.1849 ms,
  Escape followed 206.3442 ms later, readiness was Walk clip 1/yaw 49,152, completion was Run clip
  2/yaw 49,152, epoch advanced `3 -> 19`, and the actor moved `-1568 Q9` in Z with zero X delta
  and zero render blocks.
- `canonical-prototype-v40` passed on its first run in 147.077 seconds. PID 11268 recorded
  508.8601 ms from W-down to Shift-down and 207.5121 ms from Shift-down to Escape. Readiness
  committed Walk clip 1/yaw 49,152 at local Z `-64`; completion retained the same actor/region at
  local Z `-1792`, Run clip 2/yaw 49,152, and epoch `3 -> 19`. Total delta was `-1728 Q9`
  (54 exact 32-Q9 units); clocks advanced `2/3 -> 24/25` with reset/suspend/resume/stall
  `1/0/0/0`, object state idle, exactly two values, exit zero, and empty stderr.
- `runseal :guard` and `runseal :init` passed. Flavor retained zero deny issues and the five
  pre-existing warnings. No Rust product, Runtime, traversal, renderer, GPU resource, source, or
  synchronization code changed.

## Conclusion

Accepted. The live Prototype proves held Shift is readmitted as the current gait modifier:
pressing Shift transitions the sole retained actor from Walk to Run while held W, forward facing,
actor identity, clock continuity, and the existing two-value session contract remain exact.

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
