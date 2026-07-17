# Experiment 0112: Native Prototype Focus Discontinuity

Status: Accepted

## Hypothesis

The live Prototype can consume a real focus-loss/resume discontinuity after readiness, clear a held
locomotion key before simulation, reset host elapsed time without backlog, and resume with no
locomotion effect or stale object-action state.

## Scope

- Against one exact visible Prototype window, post ordered `W down` then `WM_KILLFOCUS`.
- Allow at least one suspended host sample, then post `WM_SETFOCUS` and allow Reset plus later Ready
  sampling before graceful Escape.
- Require the final actor motion/presentation to equal the readiness actor exactly.
- Require one new suspend and resume, at least one suspended sample and post-resume reset, later
  Ready progress, and no render block.
- Require observation and interaction state to remain idle with no target, acknowledgement,
  consumption, or action counters.
- Preserve all existing Escape, window-close, and sustained capacity-one session gates.

Changing HostInput, HostClock, Prototype ordering, action policy, telemetry, session schema, or
Runtime behavior; retaining a message journal; replaying timestamps; or synthesizing focus state is
out of scope.

## Workload

1. Generalize the maintained native window harness action enough to post exact input, suspend,
   resume, or close actions through the same class/title/PID-qualified `PostMessageW` owner.
2. Add a bounded post-readiness focus session: held W plus focus loss, suspended wait, resume wait,
   then Escape.
3. Compare readiness/final actor state and exact host-clock counter deltas.
4. Prove final object state, exact actor state, and all existing session gates remain unchanged.

## Controlled Variables

- The product message loop remains message pump, input ingest/exit, activation-aware sampling,
  discontinuity observation, current-batch input admission, simulation, and frame.
- Focus loss continues to be the sole native input cleanup message and activation reducer input.
- The harness posts messages only; it does not call product internals or mutate clocks/policies.
- Runtime, renderer, shaders, frame ABI, resources, descriptors, copies, readback, and
  synchronization remain structurally unchanged.

## Metrics

- Exact native message order and PID/window match; readiness/final actor state; clock suspend,
  resume, suspended-sample, reset, ready, sample, and stall counters; frame counts; pending/target/
  acknowledgement/consumption/action counters; output count; exit code; stderr; session duration;
  and existing session invariants.

## Acceptance Criteria

- The exact visible process window receives W down before `WM_KILLFOCUS`, later receives
  `WM_SETFOCUS`, and exits normally through Escape.
- Final actor motion and presentation equal readiness exactly despite elapsed real time after
  resume.
- Final clock adds exactly one suspend and one resume, contains a suspended sample and a new reset,
  later advances Ready samples, adds no stall, and reports no elapsed backlog.
- No W locomotion effect, object target/action, render block, or diagnostic history appears.
- Focused tests, `runseal :guard`, and `runseal :canonical-prototype` pass with no product or
  engine/GPU/resource structural change.

## Results

`canonical-prototype-v29` passed in 92.183 seconds. In the focus session, the exact visible window
for PID 18472 received:

1. `WM_SETFOCUS`, `WM_KEYDOWN:W`, `WM_KILLFOCUS`;
2. after the suspended interval, `WM_SETFOCUS`;
3. after the recovery interval, the maintained Escape action.

Readiness at live frame/sample 5 reported clock counts
`suspend/resume/reset/suspended-sample/ready/stall = 0/0/1/0/4/0`. Completion at live
frame/sample 1643 reported `1/1/2/635/1006/0`: exactly one suspend, resume, and post-resume reset,
635 suspended samples, later Ready progress, and no stall or elapsed backlog. The complete final
actor state equaled readiness exactly, render blocks remained zero, object observation and
interaction remained idle, stdout contained exactly readiness plus completion, exit code was zero,
and stderr was empty.

The existing Escape, native window-close, forced-termination-silence, and sustained capacity-one
gates remained exact. The sustained gate retained 12 Rejected frames and 1,072 suppression frames.

## Conclusion

Accepted. The maintained real-process workflow now proves native held-input cleanup and
activation-before-sample recovery through the existing product path. No product schema, input or
clock policy, Runtime behavior, telemetry, journal, replay route, or engine/GPU/resource structure
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
