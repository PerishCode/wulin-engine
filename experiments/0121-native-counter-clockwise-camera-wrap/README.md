# Experiment 0121: Native Counter-Clockwise Camera Wrap

Status: Accepted

## Hypothesis

When native Q-down and W-down are queued atomically before the exact Prototype window thread
resumes, the sole `HostInput` preserves the Q press edge and the pure camera candidate wraps from
orbit 0 to orbit 3. Camera-relative W must therefore commit only exact positive-X Walk.

## Scope

- Start one real Prototype process at default camera orbit 0.
- After readiness, briefly suspend only its exact visible window thread and queue ordered
  `WM_SETFOCUS`, Q-down, and W-down before restoring the thread.
- Post Escape after one bounded delay outside the atomic batch.
- Use camera-relative Walk as the product-level oracle: orbit 3 maps local W to positive world X,
  zero Z, and yaw 0.
- Preserve every existing input, session, object, restart, failure, and lifecycle gate.

Changing `HostInput`, Win32 capture, camera or locomotion policy, product reports, Runtime behavior,
terrain/traversal ownership, renderer/GPU resources, synchronization, or sources is out of scope.

## Workload

1. Add one acceptance-only native Q/W atomic sequence with delayed Escape.
2. Add one post-readiness session that validates exact thread/batch evidence, orbit-zero initial
   camera, actor identity and shape, positive-X-only Walk, presentation, clock continuity, idle
   object state, and zero render block.
3. Keep the saturated general camera acceptance owner unchanged by assigning the new bounded
   invariant to its own focused file.
4. Extend structural guards, focused Prototype report revision, init ownership, and live
   documentation.
5. Run Deno formatting/type checks, focused Rust tests, `runseal :guard`, `runseal :init`, and
   `runseal :canonical-prototype`.

## Controlled Variables

- The exact window thread is suspended only while Q and W are queued and is restored in `finally`.
- Both key delays are zero. The batch span is bounded to 50 ms; Escape is posted at least 200 ms
  after W.
- Q and W remain held until Escape. Scheduling may vary the number of committed fixed steps, but
  every step must be exact positive X.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window and native message order; window thread ID; atomic batch span;
  Q-to-W and delayed-Escape intervals; initial orbit/rig; actor handle/region/shape/velocity; final
  Q9 displacement, step count, presentation and epoch; clock counters; frame/object state; output
  count; exit code; stderr; workflow duration; and all prior Prototype gates.

## Acceptance Criteria

- One visible exact process window receives `WM_SETFOCUS, WM_KEYDOWN:Q, WM_KEYDOWN:W` in one atomic
  batch, followed by delayed `WM_KEYDOWN:Escape`.
- The batch reports one positive exact window thread ID, zero key delays, one nonnegative posting
  interval, and a span in `0..=50` ms.
- Readiness is orbit 0. Final X delta is a positive nonzero whole multiple of 32 Q9; Z delta is
  zero, with step count in `1..=43`.
- Final presentation is imported Walk clip 1 at yaw 0 with a later epoch. Actor handle, region,
  half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

The initial post-cleanup guard reached Cargo and reported that the repository-local Agility SDK
removed by Experiment 0120 was absent. The maintained `runseal :gpu-lab correctness` workflow
restored pinned SDK 1.619.4 and passed on the RTX 4070 Ti SUPER with checksum
`7ae6c64a0b95628a`, zero mismatch, and no D3D12 errors. This was a resource bootstrap, not a code
or threshold change.

`canonical-prototype-v37` passed on its first run in 139.716 seconds. The exact visible window for
PID 6788 used thread 18860 to queue Q and W atomically. Their posting interval and complete batch
span were both 0.0011 ms, and Escape followed after 232.1975 ms.

Readiness retained camera orbit 0. Completion moved the actor exactly 14 Walk steps from `(0, 0)`
to `(448, 0)` Q9, retained imported Walk clip 1 and yaw 0, and advanced the animation epoch
`1 -> 35`. With exact native W as the only locomotion key, positive-X-only movement is possible
only after the Q candidate wraps to orbit 3.

Clock reset/suspend/resume/stall counts stayed `1/0/0/0`; Ready advanced `2 -> 40`, samples
advanced `3 -> 41`, render blocks remained zero, and object policies stayed idle. The session
emitted exactly readiness plus completion and exited cleanly. All 103 engine-runtime, 45 Prototype,
and 20 reference-host tests plus every prior Prototype process gate passed. Deno checks, init, and
the repository guard passed with zero Flavor denies.

## Conclusion

Accepted. A real Win32 process now proves that one Q press edge wraps the existing pure camera
candidate counter-clockwise from orbit 0 to orbit 3 before W locomotion is authored. No input
history, controller state, product schema, Runtime route, traversal change, or engine/GPU/resource
ownership was added.

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
