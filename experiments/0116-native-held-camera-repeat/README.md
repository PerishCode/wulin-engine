# Experiment 0116: Native Held Camera Repeat

Status: Accepted

## Hypothesis

The live reference host suppresses a repeated native E-down while E remains held across
Prototype readiness, so a simultaneous W-down uses the already committed orbit-one camera
without producing a second camera press edge or orbit transition.

## Scope

- Post one native E-down before readiness and deliberately omit its matching key-up.
- Require readiness to report the existing committed clockwise orbit index 1.
- After readiness, post another E-down followed immediately by W-down to the same exact visible
  class/title/PID-qualified window, then post Escape after a bounded delay.
- Use exact camera-relative Walk displacement and presentation as the product-level oracle:
  retained orbit 1 maps local W to negative world X, zero Z, and yaw 32,768.
- Preserve every existing Escape, window-close, focus, Jump, object, restart, failure, and
  lifecycle gate.

Changing `HostInput`, the Win32 adapter, camera policy, locomotion policy, product reports,
Runtime behavior, terrain ownership, renderer/GPU resources, or synchronization is out of scope.

## Workload

1. Add one acceptance-only native action sequence for repeated E-down, W-down, and delayed Escape.
2. Add one real-process session starting with the existing pre-readiness clockwise camera input.
3. Validate exact native process/window/message identity, readiness orbit, final actor identity,
   horizontal displacement, Walk presentation, clock continuity, and zero render block.
4. Extend the existing session guard to require this live proof and the retained reference-host
   duplicate-down suppression authority.

## Controlled Variables

- E remains held from the startup batch through readiness and completion; no E-up is posted.
- The repeated E-down and W-down are one ordered native action against the same window.
- W remains held until Escape; movement length may vary with host scheduling, but every committed
  step must be the exact 32-Q9 orbit-one direction.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window and native message order; key-post and delayed-Escape
  intervals; readiness camera orbit/rig/anchor; actor handle/region/shape/velocity; final Q9
  displacement, step count, presentation and epoch; clock counters; render/object state; output
  count; exit code; stderr; workflow duration; and all previous Prototype gates.

## Acceptance Criteria

- Startup sends exactly `WM_SETFOCUS`, `WM_KEYDOWN:E`; the post-readiness action sends exactly
  `WM_SETFOCUS`, repeated `WM_KEYDOWN:E`, `WM_KEYDOWN:W`, and delayed
  `WM_KEYDOWN:Escape` to the same visible PID/window.
- Readiness is committed orbit 1. Final X delta is a negative nonzero whole multiple of 32 Q9,
  Z delta is zero, and step count is bounded in `1..=43`.
- Final presentation is imported Walk clip 1 at yaw 32,768 with a later epoch. Actor handle,
  region, half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies remain idle, and stdout is exactly readiness plus completion with
  exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

`canonical-prototype-v32` passed in 109.679 seconds. The exact visible window for PID 20468
received startup `WM_SETFOCUS` plus E-down and retained the same window handle through the
post-readiness action. The repeated E-down preceded W-down by 2.2142 ms, and Escape was posted
205.2474 ms later.

Readiness committed camera orbit 1 with the exact actor-relative rig. Completion retained orbit
1 behavior: the actor moved from `(0, 0)` to `(-352, 0)` Q9, exactly 11 negative-X Walk steps.
Final presentation was imported Walk clip 1, yaw 32,768, with animation epoch `1 -> 24`.
Generation, region, half height, and zero vertical velocity remained exact; the terrain-following
center changed only from 141,824 to 142,720.

Clock reset/suspend/resume/stall counts stayed `1/0/0/0`; Ready advanced `2 -> 28`, samples
advanced `3 -> 29`, render blocks remained zero, and object policies stayed idle. The session
emitted exactly readiness plus completion and exited cleanly. All prior Prototype gates passed in
the same workflow.

## Conclusion

Accepted. A real Win32 process now proves that a repeated native E-down cannot synthesize a second
camera press edge while E remains held: same-batch W locomotion uses the already committed orbit
1 exactly. No input history, controller state, product schema, Runtime route, or
engine/GPU/resource ownership was added.

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
