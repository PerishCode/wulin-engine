# Experiment 0118: Batch-Invariant Native Object Feedback

Status: Accepted

## Hypothesis

The intermittent Prototype object-feedback mismatch is an acceptance-fixture geometry problem, not
a lost or reordered product feedback transaction. A real native F/Enter batch plus fixed
stationary-facing source fixtures can prove exact Activated and Rejected projection for every
allowed first simulation batch without constraining product time.

## Scope

- Post native F-down and Enter-down while the exact visible class/title/PID-qualified window
  thread is briefly suspended, then resume the thread before product execution.
- Keep the actor stationary at yaw 0. Use the existing base center for a strictly positive-X
  nearest target and one separately cooked `base + 4` center for a non-positive-X nearest target.
- Cook the action center and its required diagonal traversal center into the same signed terrain
  and schema-3 object sources.
- Preserve the sustained capacity-one session by consuming at the base fixture, moving with D only
  after readiness, then releasing D before the second F/Enter attempt.
- Split the saturated acceptance input owner into low-level native transport, named actions, and
  composed sequences.

Changing Prototype, reference-host, Runtime, object policy, frame feedback, rendering, GPU
resources, synchronization, or product evidence is out of scope.

## Workload

1. Reproduce the mismatch with F/Enter/direction keys queued atomically against one exact window
   thread.
2. Compare the exact committed step count, actor position/yaw, nearest-object delta, facing dot,
   submitted feedback, and projected feedback.
3. Enumerate the fixed source geometry across the complete allowed `1..=8` first-batch domain and
   select stationary fixtures whose facing sign cannot vary with step count.
4. Run exact native Activated, Rejected, and sustained capacity-one real-process sessions, then all
   previous Prototype, restart, failure, Sidecar, input, and lifecycle gates.

## Controlled Variables

- Both primary action sessions post only F-down and Enter-down; no locomotion key affects their
  actor position, yaw, contact, or nearest target.
- The actor remains at local `(0, 0)` with Survey clip 0 and yaw 0 for any nonzero first batch.
- The base fixture nearest delta is exactly `(160, -32)` Q9, so its facing dot is `160`.
- The `base + 4` fixture nearest delta is exactly `(-224, -32)` Q9, so its facing dot is `-224`.
- The atomic helper may suspend only the exact window thread and must restore it in `finally`.
  Product time, focus policy, and simulation bounds remain unchanged.

## Metrics

- Exact schema/class/title/PID/window/thread and native message order; atomic batch span and
  per-key interval; first-batch step count; actor position/presentation/contact; nearest identity,
  delta, distance, source namespace, and independent source oracle; submitted/projected feedback;
  capacity consumption/exclusion/acknowledgement/suppression; clock/render/traversal state; process
  completion; source hashes and region counts; workflow duration; and all previous gates.

## Acceptance Criteria

- Each primary session queues `WM_SETFOCUS`, F-down, and Enter-down against one exact visible
  process window while its UI thread is suspended, resumes the thread, and reports one finite
  atomic span no greater than 50 ms.
- Activated remains stationary at the base center, resolves the exact positive-X target, submits
  and projects Activated once, and commits consumption.
- Rejected remains stationary at `base + 4`, resolves the exact negative-X target, submits and
  projects Rejected once, commits no consumption, and records one ineligible attempt.
- Both sessions accept any exact nonzero `1..=8` simulation batch without fixture-dependent
  feedback variance.
- The sustained session preserves the first consumed identity, selects an exclusion-aware second
  target after a bounded D motion, projects exactly 12 capacity Rejected frames, and retains
  suppression without copying object state.
- `runseal :guard` and `runseal :canonical-prototype` pass without a product, Runtime,
  engine/GPU/resource, synchronization, telemetry, retry, or threshold change.

## Results

Atomic F/Enter/direction posting reproduced the original mismatch during diagnosis, disproving
message splitting as its cause. The committed first transaction could legally contain `1..=8`
steps; locomotion crossed the fixed 256-Q9 object grid and changed the nearest target and facing
dot. Attempts to force one step through focus reset were also not exact. The final fixtures remove
both timing and motion from the facing oracle.

`canonical-prototype-v34` passed in 120.784 seconds. PID 10244 queued F and Enter on window thread
15328 with a 0.0012 ms atomic span. Its three-step stationary transaction stayed at `(0, 0)`,
selected qualified authored ID 496 at delta `(160, -32)` Q9, and submitted/projected Activated
exactly once.

PID 13120 queued the same keys on thread 11516 with a 0.0012 ms atomic span. Its one-step
stationary transaction stayed at `(0, 0)` in the `base + 4` region, selected qualified authored ID
495 at delta `(-224, -32)` Q9, and submitted/projected Rejected exactly once without consumption.

The sustained PID 21524 consumed base identity 496, held D for 1040.276 ms, released D before the
second action, retained one committed consumption and one ineligible attempt, resolved
exclusion-aware identity 503, projected 12 capacity Rejected frames and 87 suppression frames, and
copied no object state. The signed fixture sources contained 88 regions; terrain edge validation
reported 4,917 comparisons and zero mismatch.

All previous Prototype gates, exact two-value sessions, restart/Sidecar cleanup, Rust/Deno tests,
Flavor, and repository guards passed. The product, Runtime, reference-host, renderer, GPU resource,
descriptor, copy, readback, and synchronization paths were unchanged.

## Conclusion

Accepted. Real native object feedback now has an exact batch-invariant acceptance fixture. The
intermittent mismatch was removed by correcting the oracle geometry, not by changing product time,
loosening thresholds, adding retries, or hiding valid outcomes. Acceptance input transport is
split by ownership and retains no event history or compatibility surface.

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
