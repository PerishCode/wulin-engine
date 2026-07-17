# Experiment 0111: Native Window-Close Session Completion

Status: Accepted

## Hypothesis

The existing Prototype session contract can prove a real native window-close as the second exact
graceful completion route, with the same bounded two-value output and final-state authority as
Escape, without changing product behavior or adding live telemetry.

## Scope

- Post `WM_CLOSE` to the exact visible Prototype window owned by the launched process.
- Require sequence-one canonical readiness followed by one sequence-two completion whose reason is
  exactly `window-close`.
- Reuse the existing session completion invariant for process identity, actor handle, clock/frame
  monotonicity, idle object policy state, zero copied state, and zero event history.
- Preserve the existing Escape and sustained capacity-one session gates unchanged.
- Keep the native harness action explicit and bounded to one exact process/window pair.

Calling `DestroyWindow` from the harness, terminating the process, changing the product message
loop, adding another output value, event stream, inspect verb, journal, replay route, or product
artifact is out of scope.

## Workload

1. Extend the maintained native Prototype harness with one exact `WM_CLOSE` action and evidence.
2. Generalize the existing graceful-exit reader only enough to select Escape or window-close while
   retaining the same output, timeout, stderr, process-exit, and cleanup checks.
3. Add one real idle window-close session to the focused Prototype workflow.
4. Require the existing Escape and capacity-one sustained sessions to remain exact.

## Controlled Variables

- Prototype source, message loop, `CompletionReason`, session schema/revision, Runtime idle
  ordering, actor/object policies, and frame behavior remain unchanged.
- The harness uses `PostMessageW(WM_CLOSE)` against the exact class/title/PID match; it does not
  synthesize a completion value or call product internals.
- Runtime, renderer, shaders, frame ABI, resources, descriptors, copies, readback, and
  synchronization remain structurally unchanged.

## Metrics

- Native window/PID match, posted message, readiness/completion sequences, reason, process identity,
  actor handle, clock/frame totals, object policy state, stdout value count, trailing output, exit
  code, stderr, focused duration, and existing Escape/sustained invariants.

## Acceptance Criteria

- A real visible Prototype window accepts one posted `WM_CLOSE`, exits successfully, and emits
  exactly readiness plus completion with reason `window-close`.
- Completion retains the same process and actor identity, monotonic final clock/frame state, idle
  object policies, no event history, no copied object state, and no trailing output.
- Escape, forced-termination, failure, and sustained capacity-one gates remain unchanged.
- Focused tests, `runseal :guard`, and `runseal :canonical-prototype` pass with no product or
  engine/GPU/resource structural change.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained bounded Prototype session workflow.

## Evidence

All 96 engine-runtime tests, 45 Prototype tests, and 20 reference-host tests pass. Workspace
Clippy, Deno checks, Flavor, and `runseal :guard` pass with zero new deny issue.

`canonical-prototype-v28` passes in 86.089 seconds. The new live process reaches readiness at live
frame 5, and the harness finds its visible class/title/PID-qualified window for process 21236. It
posts one `WM_CLOSE` with no activation or key transition and no direct destroy. The same process
emits exactly sequence-one readiness and sequence-two completion with reason `window-close`, exits
zero with empty stderr and no trailing output, and finishes at live frame/sample count 356 with the
same actor handle and idle object policies.

The existing Escape completion remains `escape`; forced readiness termination remains
completion-free; and the sustained capacity-one session still retains exactly 12 rejected frames
and 1,069 suppression frames. No Prototype Rust, Runtime, renderer, shader, frame ABI, resource,
descriptor, copy, readback, or synchronization source changed, so the focused workflow owns the
acceptance risk.

## Conclusion

Accepted. Both declared graceful completion reasons now have exact real-process evidence under the
same bounded two-value session contract.

## Promotion

Promoted one process-qualified visible-window `WM_CLOSE` harness action, exact native action
evidence, and a maintained real window-close session gate. Promoted no product control, message-loop
change, schema change, output event, direct destroy, process termination, telemetry, inspect route,
journal, replay, engine behavior, or GPU/resource change.

## Reproduction

```powershell
cargo test --locked -p prototype -p reference-host
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :guard
runseal :canonical-prototype
```

Generated reports remain ignored under `out/captures/`.
