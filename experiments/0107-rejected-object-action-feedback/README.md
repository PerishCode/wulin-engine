# Experiment 0107: Rejected Object Action Feedback

Status: Accepted

## Hypothesis

The Prototype can distinguish an exact in-radius facing rejection from passive selection by
projecting one immutable red `Rejected` feedback value through the existing frame transaction and
reusing the existing bounded acknowledgement slot, without adding action state, object mutation,
or GPU ownership.

## Scope

- Add `Rejected` as the third immutable object-target frame feedback kind beside `Selected` and
  `Activated`.
- Emit it only when a retained target resolves, passes the inclusive 512-Q9 proximity gate, and
  fails the strict committed-facing gate as `OutsideFacing`.
- Submit the exact qualified identity as the current frame candidate and start the existing
  12-successful-projected-frame acknowledgement only when that same feedback is returned.
- Report the frame completion as `applied=false`, count the attempt as ineligible, and preserve
  zero consumption, exclusion, and suppression.
- Resolve the exact identity in the existing surface pass with one fixed red mix while preserving
  the semantic object-ID attachment.

Feedback for missing targets, source/window failures, malformed state, out-of-radius attempts, or
arbitrary gameplay rejection is out of scope. A second timer, queue, action registry, mutation,
inventory, reward, dispatcher, respawn, persistence, networking, new pass/resource/descriptor,
copy, readback, synchronization, or Wulin behavior is also out of scope.

## Workload

1. Unit-test exact rejected candidate construction, submitted/projected completion, 12-frame
   successful projection countdown, unprojected completion, and unchanged consumption.
2. Capture and immediately replay Selected, Activated, and Rejected feedback for the same visible
   object. Require identical exact target pixels, three distinct colors, unchanged object-ID
   attachment, and exact clear restoration.
3. Run the visible native side-facing action. Require exact `OutsideFacing` proximity/facing,
   submitted and projected Rejected identity, `applied=false`, 11 remaining acknowledgement
   frames, one rejected frame, and no consumption or exclusion.
4. Preserve the admitted native action, source/window lifetime, rollback, restart, traversal,
   resource, and lifecycle gates in one final optimized full acceptance.

## Controlled Variables

- Target observation, typed resolution, exact proximity/facing, successful activation,
  consumption, exclusion, suppression, and source/window lifetime retain their ownership.
- The existing acknowledgement slot remains capacity one and decrements only after an exact
  successful projected frame.
- Visible records, root constants, dispatch dimensions, passes, resources, descriptors, copies,
  readback, and synchronization remain structurally unchanged.

## Metrics

- Exact feedback identity/kind, target pixel count, color and object-ID hashes, replay, and clear.
- Exact proximity delta/distance, committed yaw/direction/dot, completion, acknowledgement,
  committed/ineligible counts, consumption, exclusion, and feedback-frame counts.
- Focused and full duration, Sidecar invocations, artifact inventory, handles, threads, and private
  bytes.

## Acceptance Criteria

- Rejected projection targets exactly the same pixels as Selected and Activated, changes only the
  resolved color, and replays deterministically.
- Only exact in-radius `OutsideFacing` produces Rejected feedback; other ineligible paths remain
  feedback-free.
- A rejected attempt commits no object effect or lifetime state, and its acknowledgement advances
  only on exact projected frames.
- Focused, guard, and final full-runtime acceptance pass without structural resource change.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, schema-3 finite sandbox centered at
`(2^40, -2^40)`, and the maintained focused/full state-driven acceptance workflows.

## Evidence

All 96 engine-runtime tests, ten interaction-policy tests, workbench tests, workspace Clippy, and
Deno formatting/type checks pass.

`canonical-frame-v11` passes in 45.636 seconds. Selected, Activated, and Rejected each cover exactly
3,472 pixels of ID 987. Their color hashes are respectively
`6d483d9613b36ef11db4df17469c0bb857734391ea879a067a1297355b4f6292`,
`315a59de357b1e93ea127158a19c2d88693f9d211014e897d3f5416494a3a683`, and
`d66e6bd4b1a4d233c5de9155e8be74a456fa4f36726de74fcf3d361feab8d5c1`; the object-ID
attachment remains unchanged, immediate replay is exact, and clear restoration passes.

`canonical-prototype-v24` passes in 77.910 seconds. The admitted native action still projects exact
Activated feedback and consumption. The independent native side-facing action resolves ID 496 at
delta `(160,0)` Q9 / 25,600 Q18 with yaw/direction/dot `49,152 / (0,-1) / 0`; it submits and
projects exact Rejected feedback, returns `applied=false`, retains 11 acknowledgement frames, and
reports committed/ineligible/consumed/activated/rejected/exclusion as `0 / 1 / null / 0 / 1 /
null`.

`runseal :guard` passes with zero Flavor denies. The first guard exposed the stale two-kind
Workbench static contract. Subsequent source-boundary pressure moved interaction readiness mapping
out of the 525-line composition flow and into an inline interaction report view; `main.rs` now has
482 lines, `interaction.rs` has 452, and the existing integration test verifies the rejected JSON
shape without another source child or lint exception.

The final-worktree `canonical-runtime-v15` passes in 257.299 seconds. Existing source replacement,
same-source departure/return, rollback, restart, 32+32 traversal, and two lifecycle cycles remain
green. Five warm and eight measured resource publications retain 492 handles and 21 threads;
private bytes move from 419,368,960 to 419,319,808 (-49,152). The report contains 24 files /
25,346,275 bytes and records 980 Sidecar invocations. Stage times are 6.901 seconds setup, 24.387
bootstrap, 16.849 Prototype, 12.350 actor lifecycle, 28.305 simulation actor, 97.594 canonical
correctness, 13.625 reactive traversal, 13.703 prepared traversal, 26.547 resources, and 15.054
lifecycle.

## Conclusion

Accepted. Exact facing rejection is now product-visible through the existing qualified frame
transaction and bounded acknowledgement, with no object effect or new retained/GPU ownership.

## Promotion

Promoted the immutable Rejected kind, exact red CPU/GPU surface oracle, successful-frame-only
rejection acknowledgement, native side-facing evidence, and the three-kind static guard. Promoted
no general failure feedback, second timer, queue, registry, mutation, inventory, reward,
dispatcher, respawn, persistence, networking, or Wulin behavior.

## Reproduction

```powershell
cargo test --locked -p engine-runtime -p workbench -p prototype
cargo clippy --locked --workspace --all-targets -- -D warnings
runseal :canonical-frame
runseal :canonical-prototype
runseal :guard
runseal :canonical-runtime
```

Generated reports remain ignored under `out/captures/`.
