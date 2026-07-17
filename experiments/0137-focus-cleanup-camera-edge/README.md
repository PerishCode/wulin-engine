# Experiment 0137: Focus-Cleanup Camera Edge

Status: Accepted

## Hypothesis

The established HostInput and frame-driven camera policies already define a camera press ordered
before FocusLost differently from pending simulation-bound actions. In one ingest, focus cleanup
should clear held state while retaining both sample-scoped edges; the camera policy should consume
the press exactly once, and a following empty ingest should not repeat it.

## Scope

- Compose the current E-down and FocusLost messages in one focused camera-policy test.
- Require held=false, pressed=true, and released=true after that ingest.
- Commit the existing clockwise candidate from orbit 0 to orbit 1.
- Require the following empty ingest to expire both edges and retain orbit 1 without another step.
- Advance the full Prototype workflow revision from v51 to v52.

HostInput implementation, camera policy, main-loop ordering, simulation-action cancellation,
native process/session count, Runtime, renderer/GPU resources, source formats, and synchronization
are out of scope.

## Workload

1. Audit HostInput focus cleanup and the Prototype main-loop camera candidate/commit ordering.
2. Feed E-down followed by FocusLost through the real `HostInput` type used by the camera policy.
3. Assert exact held/pressed/released state and commit the existing camera candidate.
4. Feed one empty ingest and assert edge expiry, no held state, and no second orbit step.
5. Run focused camera tests, the repository guard, and the complete v52 Prototype workflow.

## Controlled Variables

- Q/E direction, four exact rigs, candidate purity, runtime-before-policy commit, held-repeat
  suppression, opposite-edge cancellation, and traversal behavior remain unchanged.
- Focus cleanup continues to release all held keys without deleting pressed edges from the same
  ingest.
- Jump, observation, and interaction remain pending simulation-bound policies with their accepted
  Suspended/Reset cancellation behavior.
- No product branch, sample argument, camera action queue, intermediate report, compatibility path,
  or native process is added.

## Metrics

- Exact HostInput held/pressed/released facts after focus cleanup and empty ingest, camera orbit
  before/after commit, focused test count, complete workflow duration, report bytes, total test
  counts, Flavor findings, and process cleanup.

## Acceptance Criteria

- E-down/FocusLost in one ingest must produce held=false, pressed=true, and released=true for E.
- The current camera policy must produce and commit exactly orbit 1 from orbit 0.
- One empty ingest must produce pressed=false and released=false while retaining orbit 1.
- All four focused camera tests and the complete Prototype workflow must pass.
- HostInput, camera product code, main-loop ordering, native process count, report schema, Runtime,
  renderer/GPU, source, resource, and synchronization diffs must remain empty.

## Results

The audit confirmed that HostInput intentionally preserves state-changing edges for the complete
ingest even when FocusLost later releases the key. The camera candidate is frame-driven and is
prepared and committed before every frame; unlike Jump/F/Enter pending intents, it does not defer
admission to a nonzero simulation step.

The new composition test passed alongside the three existing camera tests. E-down followed by
FocusLost produced held=false with pressed=true and released=true, selected orbit 1, and committed
that candidate. The next empty ingest expired both edges, retained held=false, and left the camera
at orbit 1. No product source changed.

`canonical-prototype-v52` passed on its first run in 176.068 seconds. The complete existing native
session/process matrix, Sidecar lifecycle, source failures, finite-boundary liveness, focus action
suppression, and object feedback gates remained unchanged. The ignored report was 447,306 bytes.

All 103 engine-runtime, 46 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Runtime, renderer/GPU, source, resource, or
synchronization code changed.

## Conclusion

Accepted. A camera key ordered before focus loss remains one sample-scoped discrete camera action:
focus cleanup clears held state, while the retained press may commit once and cannot repeat after
edge expiry. Simulation-bound action cancellation remains a separate policy rather than an implied
camera rule.

## Reproduction

```powershell
cargo test --locked -p prototype --test camera_policy
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
