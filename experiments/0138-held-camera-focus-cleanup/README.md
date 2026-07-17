# Experiment 0138: Held Camera Focus Cleanup

Status: Accepted

## Hypothesis

HostInput should distinguish a fresh camera press from focus cleanup of a key that was already
held before the current ingest. After one committed E press, a duplicate E-down followed by
FocusLost should clear held state and emit only a release edge; the camera policy should receive
no second press and retain the committed orbit.

## Scope

- Commit the existing E-down camera candidate from orbit 0 to orbit 1.
- In the next ingest, compose duplicate E-down and FocusLost while E is already held.
- Require held=false, pressed=false, and released=true after focus cleanup.
- Require camera candidate and commit to retain orbit 1 without a second step.
- Advance the full Prototype workflow revision from v52 to v53.

HostInput implementation, camera policy, main-loop ordering, simulation-action cancellation,
native process/session count, Runtime, renderer/GPU resources, source formats, and synchronization
are out of scope.

## Workload

1. Feed E-down through the real `HostInput` and commit the existing camera candidate.
2. Feed duplicate E-down followed by FocusLost through the same retained input owner.
3. Assert exact held/pressed/released facts after the second ingest.
4. Prepare and commit the camera cleanup candidate and assert that orbit 1 remains exact.
5. Run focused camera tests, the repository guard, and the complete v53 Prototype workflow.

## Controlled Variables

- Q/E direction, four exact rigs, candidate purity, runtime-before-policy commit, opposite-edge
  cancellation, and traversal behavior remain unchanged.
- Duplicate-down suppression and focus cleanup continue to be owned solely by HostInput.
- The fresh same-ingest E-down/FocusLost path accepted by Experiment 0137 remains unchanged.
- No product branch, camera intent, controller, queue, sample argument, report field,
  compatibility path, or native process is added.

## Metrics

- Exact HostInput held/pressed/released facts, camera orbit before and after cleanup, focused test
  count, complete workflow duration, report bytes, total test counts, Flavor findings, and process
  cleanup.

## Acceptance Criteria

- The first E-down must produce and commit orbit 1 from orbit 0.
- Duplicate E-down followed by FocusLost while E is held must produce held=false, pressed=false,
  and released=true.
- The cleanup camera candidate and commit must retain orbit 1.
- All five focused camera tests and the complete Prototype workflow must pass.
- HostInput, camera product code, main-loop ordering, native process count, report schema, Runtime,
  renderer/GPU, source, resource, and synchronization diffs must remain empty.

## Results

The new composition test passed alongside the four existing camera tests. The first E-down
selected and committed orbit 1. On the retained input owner, duplicate E-down was suppressed
because E was already held; the following FocusLost cleared held state and produced only the
release edge. The camera policy observed no pressed edge, prepared orbit 1, and retained orbit 1
after commit. No product source changed.

`canonical-prototype-v53` passed on its first run in 170.211 seconds. The complete existing native
session/process matrix, Sidecar lifecycle, source failures, finite-boundary liveness, focus action
suppression, and object feedback gates remained unchanged. The ignored report was 447,346 bytes.

All 103 engine-runtime, 47 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Runtime, renderer/GPU, source, resource, or
synchronization code changed.

## Conclusion

Accepted. Focus cleanup of an already-held camera key is not a second camera action: duplicate
down is suppressed, focus loss emits only release cleanup, and the committed orbit remains exact.

## Reproduction

```powershell
cargo test --locked -p prototype --test camera_policy
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
