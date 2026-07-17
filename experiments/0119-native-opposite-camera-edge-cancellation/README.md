# Experiment 0119: Native Opposite Camera Edge Cancellation

Status: Accepted

## Hypothesis

When native Q-down, E-down, and W-down are queued atomically before the exact Prototype window
thread resumes, one `HostInput` ingest retains both opposite camera press edges. The pure camera
candidate cancels them to orbit zero, so W commits only exact negative-Z Walk.

## Scope

- Start one real Prototype process at default camera orbit 0.
- After readiness, briefly suspend only its exact visible window thread and queue ordered
  `WM_SETFOCUS`, Q-down, E-down, and W-down before restoring the thread.
- Post Escape after one bounded delay outside the atomic batch.
- Use camera-relative Walk as the product-level oracle: cancellation leaves orbit 0 and negative Z;
  losing either edge would select a nonzero orbit and rotate the movement axis.
- Preserve every existing input, session, object, restart, failure, and lifecycle gate.

Changing `HostInput`, Win32 capture, camera or locomotion policy, product reports, Runtime behavior,
terrain ownership, renderer/GPU resources, or synchronization is out of scope.

## Workload

1. Extend the acceptance-only native key vocabulary with Q and permit delayed Escape after an
   otherwise exact atomic key batch.
2. Add one post-readiness session that atomically queues Q-down, E-down, and W-down to one exact
   process window.
3. Validate exact thread/batch evidence, orbit-zero readiness, actor identity and shape,
   negative-Z-only Walk, presentation, clock continuity, idle object state, and zero render block.
4. Split the already-full session source into shared process framing and an explicit session-gate
   matrix without changing product behavior.
5. Extend the structural guard to retain the live proof and the existing opposite-edge policy
   authority.

## Controlled Variables

- The exact window thread is suspended only while the three native key messages are queued and is
  restored in `finally`.
- All three key delays are zero. The batch span is bounded to 50 ms; Escape is posted at least
  200 ms after W.
- Q and E remain held, but camera policy consumes their press edges only from the single ingest.
- W remains held until Escape. Scheduling may vary the number of committed fixed steps, but every
  step must be exact orbit-zero negative Z.
- Product output remains the existing readiness/completion pair. Acceptance retains only bounded
  native-action and derived invariant evidence.

## Metrics

- Exact schema/class/title/PID/window and native message order; window thread ID; atomic batch
  span; key-post and delayed-Escape intervals; readiness orbit/rig; actor handle/region/shape/
  velocity; final Q9 displacement, step count, presentation and epoch; clock counters; frame and
  object state; output count; exit code; stderr; workflow duration; and all prior Prototype gates.

## Acceptance Criteria

- One visible exact process window receives
  `WM_SETFOCUS, WM_KEYDOWN:Q, WM_KEYDOWN:E, WM_KEYDOWN:W` in one atomic batch, followed by delayed
  `WM_KEYDOWN:Escape`.
- The batch reports one positive exact window thread ID, three zero-delay keys, two nonnegative
  posting intervals, and a span in `0..=50` ms.
- Readiness is orbit 0. Final X delta is zero; Z delta is a negative nonzero whole multiple of
  32 Q9 with step count in `1..=43`.
- Final presentation is imported Walk clip 1 at yaw 49,152 with a later epoch. Actor handle,
  region, half height, and zero vertical velocity remain exact.
- Reset/suspend/resume/stall counts do not change, Ready/sample counts advance, render blocks
  remain zero, object policies stay idle, and stdout remains exactly readiness plus completion
  with exit zero and empty stderr.
- `runseal :guard` and `runseal :canonical-prototype` pass without product, Runtime,
  engine/GPU/resource, or synchronization changes.

## Results

`canonical-prototype-v35` passed on its first run in 128.648 seconds. The exact visible window for
PID 7632 used thread 1728 to queue Q, E, and W atomically. Consecutive posting intervals were
0.0012 and 0.0010 ms, the complete key batch spanned 0.0022 ms, and Escape followed after
238.676 ms.

Readiness committed camera orbit 0. Completion moved the actor exactly 14 Walk steps from
`(0, 0)` to `(0, -448)` Q9, retained imported Walk clip 1 and yaw 49,152, and advanced the
animation epoch. Any surviving single Q or E edge would have selected a nonzero camera candidate
and changed the movement axis, so the exact negative-Z output proves same-ingest cancellation.

Clock reset/suspend/resume/stall counts stayed `1/0/0/0`; Ready advanced `2 -> 39`, samples
advanced `3 -> 40`, render blocks remained zero, and object policies stayed idle. The session
emitted exactly readiness plus completion and exited cleanly. All prior Prototype gates passed in
the same workflow.

The session orchestrator was already exactly 500 lines before this experiment. Shared
readiness/completion process framing now remains in `sessions/mod.ts`, while the bounded session
matrix and cross-session comparisons live in `sessions/gates.ts`; both are required by init and
the repository guard.

## Conclusion

Accepted. A real Win32 process now proves that opposite Q/E press edges survive one native ingest
and cancel in the existing pure camera candidate before W locomotion is authored. No input
history, controller state, product schema, Runtime route, or engine/GPU/resource ownership was
added.

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
