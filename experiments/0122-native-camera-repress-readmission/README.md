# Experiment 0122: Native Camera Re-Press Readmission

Status: Accepted

## Hypothesis

After native startup E-down commits camera orbit 1 and remains held through readiness, a later
atomic E-up/E-down pair creates one fresh press edge. Same-ingest W must use the new orbit-2
candidate and commit exact positive-Z Walk.

## Scope

- Start one real Prototype process with the existing pre-readiness E-down and require committed
  orbit-1 readiness.
- Briefly suspend only the exact visible window thread and queue ordered E-up, E-down, and W-down
  before restoring the thread.
- Post Escape after one bounded delay outside the atomic batch.
- Use camera-relative Walk as the product-level oracle: orbit 2 maps local W to positive world Z,
  zero X, and yaw 16,384.
- Preserve every existing input, session, object, restart, failure, and lifecycle gate.

Changing `HostInput`, Win32 capture, camera or locomotion policy, product reports, Runtime behavior,
terrain/traversal ownership, renderer/GPU resources, synchronization, or sources is out of scope.

## Workload

1. Add one acceptance-only atomic E-up/E-down/W sequence with delayed Escape.
2. Add one real-process session starting from the existing held-E/orbit-1 readiness.
3. Validate exact startup/window/thread/batch evidence, actor identity and shape, positive-Z-only
   Walk, presentation, clock continuity, idle object state, and zero render block.
4. Add a focused bounded invariant owner, structural guards, report revision, init ownership, and
   live documentation.
5. Run Deno formatting/type checks, focused Rust tests, `runseal :guard`, `runseal :init`, and
   `runseal :canonical-prototype`.

## Controlled Variables

- E remains held from startup through readiness. The exact window thread is suspended only while
  E-up/E-down/W-down are queued and is restored in `finally`.
- All three key delays are zero. The batch span is bounded to 50 ms; Escape is posted at least
  200 ms after W.
- The second E-down and W remain held until Escape. Scheduling may vary the committed fixed-step
  count, but every step must be exact positive Z.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window and native message order; window thread ID; atomic batch span;
  key-post and delayed-Escape intervals; readiness orbit/rig; actor handle/region/shape/velocity;
  final Q9 displacement, step count, presentation and epoch; clock counters; frame/object state;
  output count; exit code; stderr; workflow duration; and all prior Prototype gates.

## Acceptance Criteria

- Startup sends exactly `WM_SETFOCUS, WM_KEYDOWN:E` and readiness commits orbit 1.
- The same visible PID/window then receives
  `WM_SETFOCUS, WM_KEYUP:E, WM_KEYDOWN:E, WM_KEYDOWN:W` in one atomic batch, followed by delayed
  `WM_KEYDOWN:Escape`.
- The batch reports one positive exact window thread ID, three zero key delays, two nonnegative
  posting intervals, and a span in `0..=50` ms.
- Final X delta is zero; Z delta is a positive nonzero whole multiple of 32 Q9 with step count in
  `1..=43`.
- Final presentation is imported Walk clip 1 at yaw 16,384 with a later epoch. Actor handle,
  region, half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

`canonical-prototype-v38` passed on its first run in 138.736 seconds. The exact visible window for
PID 1752 first received startup E-down and published committed orbit-1 readiness. Its same window
then used thread 18524 to queue E-up, E-down, and W-down atomically. Consecutive posting intervals
were 0.0016 and 0.0010 ms, the complete key batch spanned 0.0026 ms, and Escape followed after
211.8739 ms.

Completion moved the actor exactly 13 Walk steps from `(0, 0)` to `(0, 416)` Q9, retained imported
Walk clip 1 and yaw 16,384, and advanced the animation epoch `1 -> 35`. If either E-up or the second
E-down had not admitted a fresh press edge, the camera would have remained at orbit 1 and W would
have moved negative X; the positive-Z-only result therefore proves orbit-2 re-admission.

Clock reset/suspend/resume/stall counts stayed `1/0/0/0`; Ready advanced `2 -> 40`, samples
advanced `3 -> 41`, render blocks remained zero, and object policies stayed idle. The session
emitted exactly readiness plus completion and exited cleanly. All 103 engine-runtime, 45 Prototype,
and 20 reference-host tests plus every prior Prototype process gate passed. Deno checks, init, and
the repository guard passed with zero Flavor denies.

## Conclusion

Accepted. A real Win32 process now proves that releasing held E and pressing it again creates a
fresh camera press edge after readiness, advancing the existing pure candidate from orbit 1 to
orbit 2 before W locomotion is authored. No input history, controller state, product schema,
Runtime route, traversal change, or engine/GPU/resource ownership was added.

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
