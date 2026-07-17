# Experiment 0139: Camera Focus Readmission

Status: Accepted

## Hypothesis

Focus cleanup should end the lifetime of a held camera key without suppressing later input. After
E-down commits orbit 1 and FocusLost releases E, a later E-down should be admitted as a fresh
sample-scoped press and commit exactly one further clockwise step to orbit 2.

## Scope

- Commit the existing E-down camera candidate from orbit 0 to orbit 1.
- Ingest FocusLost and require release-only cleanup while retaining orbit 1.
- Ingest a later E-down and require held=true, pressed=true, and released=false.
- Commit the re-admitted candidate from orbit 1 to orbit 2.
- Advance the full Prototype workflow revision from v53 to v54.

HostInput implementation, camera policy, main-loop ordering, simulation-action cancellation,
native process/session count, Runtime, renderer/GPU resources, source formats, and synchronization
are out of scope.

## Workload

1. Feed E-down through the real retained `HostInput` and commit orbit 1.
2. Feed FocusLost and assert exact held/pressed/released cleanup facts and no camera step.
3. Feed a later E-down through the same owner and assert that it is a fresh press.
4. Prepare and commit the camera candidate and assert exact orbit 2.
5. Run focused camera tests, the repository guard, and the complete v54 Prototype workflow.

## Controlled Variables

- Q/E direction, four exact rigs, candidate purity, runtime-before-policy commit, held-repeat
  suppression, opposite-edge cancellation, and traversal behavior remain unchanged.
- The fresh same-ingest focus edge and already-held duplicate-down cleanup accepted by Experiments
  0137 and 0138 remain unchanged.
- No product branch, camera intent, controller, queue, sample argument, report field,
  compatibility path, or native process is added.

## Metrics

- Exact HostInput held/pressed/released facts across focus cleanup and re-press, camera orbit after
  each commit, focused test count, complete workflow duration, report bytes, total test counts,
  Flavor findings, and process cleanup.

## Acceptance Criteria

- The first E-down must produce and commit orbit 1 from orbit 0.
- FocusLost must produce held=false, pressed=false, and released=true without changing orbit 1.
- The later E-down must produce held=true, pressed=true, and released=false.
- The re-admitted camera candidate and commit must produce exact orbit 2.
- All six focused camera tests and the complete Prototype workflow must pass.
- HostInput, camera product code, main-loop ordering, native process count, report schema, Runtime,
  renderer/GPU, source, resource, and synchronization diffs must remain empty.

## Results

The new composition test passed alongside the five existing camera tests. Initial E-down committed
orbit 1. FocusLost cleared held state, emitted only release, and left the candidate at orbit 1.
The next E-down on the same HostInput owner was admitted as held=true, pressed=true, and
released=false; the camera then prepared and committed exact orbit 2. No product source changed.

`canonical-prototype-v54` passed on its first run in 175.160 seconds. The complete existing native
session/process matrix, Sidecar lifecycle, source failures, finite-boundary liveness, focus action
suppression, and object feedback gates remained unchanged. The ignored report was 447,322 bytes.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Runtime, renderer/GPU, source, resource, or
synchronization code changed.

## Conclusion

Accepted. Focus cleanup bounds the old held camera-key lifetime but does not poison the key: a
later down transition is a fresh press and advances the camera exactly once.

## Reproduction

```powershell
cargo test --locked -p prototype --test camera_policy
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
