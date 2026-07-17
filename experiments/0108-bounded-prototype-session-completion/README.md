# Experiment 0108: Bounded Prototype Session Completion

Status: Accepted

## Hypothesis

The Prototype can expose exact state after its one canonical readiness by emitting one immutable
completion value only after a graceful process exit, without adding repeated readiness, an event
stream, an inspect path, retained event history, or Runtime ownership.

## Scope

- Keep the existing canonical readiness as sequence one and advertise one fixed session contract.
- After a graceful Escape or window close, emit exactly one sequence-two completion value after
  the Runtime is idle and before host teardown.
- Include only the already-owned final actor, host-clock status, bounded frame counters, retained
  object target status, and capacity-one interaction status.
- Move readiness serialization out of the Prototype composition root into the same explicit
  session-report owner.
- In one real process, commit the existing first object action before readiness, issue a second
  Enter edge after readiness, exit with Escape, and require exact unchanged consumption plus one
  additional ineligible attempt.

An output event stream, repeated readiness, periodic snapshots, an inspect verb, a diagnostic
bootstrap seed, retained per-event results, a journal, replay, product artifact writes, canonical
object mutation, or a broader product subsystem is out of scope.

## Workload

1. Unit-test the fixed readiness/completion sequence, graceful reason encoding, exact final
   capacity-one state, and checked frame totals without launching the GPU runtime.
2. Preserve the existing normal Escape process gate and require one readiness followed by one
   matching graceful completion with no trailing output.
3. Run one sustained native process: post `F+Enter+D`, read its exact Activated readiness, release
   and press Enter again, then press Escape. Require the same consumed identity, committed count
   one, ineligible count one, no acknowledgement, active suppression, a later final actor/frame,
   and a successful process exit.
4. Preserve all focused Prototype tests and gates, then run the final optimized full acceptance
   once after the boundary is stable.

## Controlled Variables

- Runtime, renderer, canonical source/snapshot, actor transaction, object resolution, feedback,
  acknowledgement, exclusion, suppression, and source/window lifetime remain unchanged.
- The completion report reads final state only after the message loop ends and performs no
  Runtime mutation.
- Forced termination, bootstrap failure, and fatal runtime failure emit no successful completion.
- GPU records, shaders, passes, resources, descriptors, copies, readback, and synchronization
  remain structurally unchanged.

## Metrics

- Exact process identity, output sequence, completion reason, actor handle/state, clock counters,
  frame totals, target state, interaction counters, consumed identity, acknowledgement, and
  suppression frame count.
- Number of stdout values, native input transitions, process exit code, stderr, focused/full
  duration, Sidecar invocations, artifacts, handles, threads, and private bytes.

## Acceptance Criteria

- A successful live process emits exactly one canonical readiness and one matching graceful
  completion; forced or failed processes do not claim completion.
- The sustained native process proves a post-readiness Enter was admitted after the first
  committed action and changed only the existing ineligible count.
- Completion retains the exact consumed identity and capacity-one committed count, clears no
  source/session lifetime, and reports no event history or copied canonical object state.
- Focused tests, `runseal :guard`, and final full-runtime acceptance pass without structural
  engine, GPU, or resource changes.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained focused/full state-driven acceptance workflows.

## Evidence

All 96 engine-runtime tests, six canonical-object position/proximity tests, 44 Prototype tests
including two exact session-report tests, 20 reference-host tests, workspace Clippy, Deno checks,
Flavor, and the repository guard pass.

`canonical-prototype-v25` passes in 81.332 seconds. Every forced readiness process and the
15-second finite-boundary process report `completionEmitted=false` with no trailing output. The
normal Escape process emits exactly sequence-one readiness and sequence-two completion with one
process identity, reason `escape`, exit code zero, empty stderr, and no third value.

The sustained native process posts `F+Enter+D`, reaches Activated readiness for qualified ID 496
at live frame 5, then receives an exact Enter release/press edge before Escape. Its completion at
live frame 970 retains the same consumed identity and committed count one, advances only the
existing ineligible count to one, clears the acknowledgement and target, projects suppression for
954 frames, and reports no event history or copied object state.

The final `canonical-runtime-v16` passes in 252.427 seconds with all source/window, rollback,
restart, 32+32 traversal, and two lifecycle gates unchanged. Five warm and eight measured resource
publications retain 495 handles and 21 threads; private bytes move from 419,770,368 to 421,027,840
(+1,257,472). The report contains 24 files / 25,346,292 bytes and records 979 Sidecar invocations.

## Conclusion

Accepted. One bounded graceful completion makes exact post-readiness Prototype behavior observable
without a live query surface or recurring telemetry.

## Promotion

Promoted the fixed two-value successful-session contract, exact graceful reason/final-state
serialization, forced-termination absence checks, sustained post-readiness Enter evidence, and
explicit session/process report ownership. Promoted no event stream, inspect verb, retained event
result, journal, replay, product file write, Runtime state, registry, inventory, persistence,
networking, or Wulin semantics.

## Reproduction

```powershell
cargo test --locked -p prototype -p reference-host
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :canonical-prototype
runseal :guard
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/`.
