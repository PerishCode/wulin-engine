# Experiment 0117: Native Out-of-Range Key Rejection

Status: Accepted

## Hypothesis

The live reference host preserves the full Win32 `WPARAM` key value until checked conversion and
rejects values above `u8::MAX`, so `0x145` cannot truncate to its low-byte E alias and rotate the
Prototype camera.

## Scope

- Start one real Prototype process at exact default camera orbit 0.
- After readiness, post native key-down `0x145`, then W-down, to the same exact visible
  class/title/PID-qualified window and post Escape after a bounded delay.
- Use camera-relative Walk as the product-level oracle. Checked rejection leaves orbit 0 and maps
  W to negative world Z; low-byte truncation to E (`0x45`) would rotate to orbit 1 and map W to
  negative world X.
- Preserve all existing input, session, object, restart, failure, and lifecycle gates.

Changing `HostInput`, Win32 capture, camera/locomotion policy, product reports, Runtime behavior,
terrain ownership, renderer/GPU resources, or synchronization is out of scope.

## Workload

1. Extend the acceptance-only native helper key vocabulary with one labeled `0x145` transition.
2. Add one post-readiness session that orders `0x145`, W, and delayed Escape against one exact
   process window.
3. Require exact orbit-zero readiness, actor identity/shape, negative-Z-only Walk displacement,
   Walk presentation, clock continuity, idle object state, and zero render block.
4. Extend the existing session guard to require both the live process proof and the retained
   `u8::try_from` checked-conversion authority.

## Controlled Variables

- `0x145` is intentionally one greater high byte above the valid E key: its low eight bits are
  exactly `0x45`.
- W is the only valid movement key in the post-readiness action.
- W remains held until Escape; scheduling may vary the bounded step count, but every step must be
  exact orbit-zero negative Z.
- Product output remains the existing readiness/completion pair. No input value or edge is added
  to product evidence.

## Metrics

- Exact schema/class/title/PID/window and native message order; posted virtual key and key/exit
  intervals; readiness camera orbit/rig; actor identity/region/shape/velocity; final Q9
  displacement, step count, presentation and epoch; clock counters; render/object state; output
  count; exit code; stderr; duration; and all previous Prototype gates.

## Acceptance Criteria

- One visible exact process window receives `WM_SETFOCUS`, labeled `WM_KEYDOWN` for virtual key
  325, W-down, and delayed Escape in that order.
- Readiness is camera orbit 0. Final X delta is zero; Z delta is a negative nonzero whole multiple
  of 32 Q9 with step count in `1..=43`.
- Final presentation is imported Walk clip 1 at yaw 49,152 with a later epoch. Actor handle,
  region, half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

`canonical-prototype-v33` passed in 114.141 seconds. The exact visible window for PID 16624
received virtual key 325 and W-down 2.2231 ms apart, then Escape 222.059 ms after W.

Readiness committed camera orbit 0. Completion moved the actor exactly 13 Walk steps from
`(0, 0)` to `(0, -416)` Q9, retained imported Walk clip 1 and yaw 49,152, and advanced the
animation epoch. A low-byte truncation of 325 to E (`69`) would instead have committed orbit 1
and negative-X motion, so the measured direction excludes truncation.

Clock reset/suspend/resume/stall counts stayed `1/0/0/0`; Ready advanced `1 -> 27`, samples
advanced `2 -> 28`, render blocks remained zero, and object policies stayed idle. The session
emitted exactly readiness plus completion and exited cleanly. All prior Prototype gates passed in
the same workflow.

The session orchestrator crossed its 500-line quality threshold during development, so repeated
startup/jump baseline comparisons were consolidated into one local assertion helper. This changed
no product or acceptance semantics and returned the file below the enforced limit.

## Conclusion

Accepted. The real Win32-to-product loop now proves that an out-of-range key value whose low byte
aliases a live action is checked and rejected rather than truncated. No product input telemetry,
history, compatibility decoder, controller state, Runtime route, or engine/GPU/resource ownership
was added.

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
