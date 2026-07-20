# ADR 0166: Focused Activated-Frame Acceptance

- Status: Accepted
- Date: 2026-07-20
- Experiment: 0163 Focused Activated-Frame Acceptance

## Context

The complete `runseal :canonical-prototype` matrix is authoritative but costs
approximately 157 seconds and launches 18 real processes. Debugging the exact
Activated acknowledgement path currently requires rerunning unrelated movement,
camera, Jump, boundary, failure, restart, and lifecycle cases.

Experiment 0161 replaced a fragile wall-time proxy with exact visible-frame
completion, and Experiment 0162 removed redundant report aliases. Their resulting
Activated process is independently executable, but before this decision it had no
maintained operator mode and its invariant validator was reachable only through the
combined Activated/Rejected gate.

Visible-pixel acceptance also requires an interactive Windows input desktop. When the
RDP session is disconnected, screen capture fails with Win32 error 5 after product
startup unless that prerequisite is checked explicitly.

## Decision

- Extend the existing `runseal :canonical-prototype` wrapper with one exact
  `--case=activated-frame` argument; do not add another wrapper.
- Build Prototype, cook only the base and startup-traversal centers, and launch one
  exact Activated recovery process.
- Export a narrow Activated validator from the existing object-feedback gate and reuse
  its source oracle, product-state, input, frame-observer, transport, and
  single-owner checks.
- Observe the exact visible Prototype client through
  `PrintWindow(PW_CLIENTONLY | PW_RENDERFULLCONTENT)` after DWM composition and GDI
  synchronization. Do not use the RDP desktop DC as the maintained D3D12 swap-chain
  evidence surface.
- On observer timeout, restore z-order and post product Escape before rejecting typed
  `completionObserved=false` evidence so exact final product state remains diagnosable.
- Write a separate generated focused report that full acceptance never reads.
- Require an accessible interactive input desktop before focused build/cook and retain
  observer stderr if it exits before readiness.
- Preserve the no-argument full matrix and require it to pass as v78 before acceptance.
- Add no fallback capture, fixed success dwell, retry, product telemetry, compatibility
  surface, new process in the full matrix, or product/engine/rendering change.

## Consequences

- The common Activated debugging loop should become one process and less than 30
  seconds while remaining behaviorally identical to the full case.
- Disconnected-desktop failures become immediate and actionable instead of appearing
  after source cooking and product startup.
- RDP desktop-DC staleness no longer makes the frame observer alternate between current
  and initial swap-chain images.
- Focused success is useful for iteration but cannot replace full acceptance.
- The wrapper set and all product/runtime ownership remain unchanged.

## Evidence

The interactive-desktop preflight reduced disconnected-session failure from 14.0 to
6.7 seconds. Screen/client DC experiments then proved intermittent RDP staleness while
exact product completion still reported 12 Activated frames and correct final state.

The final window-composition implementation passed three consecutive focused runs in
13.956, 13.716, and 13.789 seconds. The final 27,314-byte focused report contains one
unique PID, product Escape, exact 12 Activated / 13 suppression / zero render-block
frames, one commit, cleared target/acknowledgement, and zero copied subtrees. Its
observer recorded `0/777/0`, six Activated samples, and two clear samples in
1,808.173 ms. The representative run is 11.38 times faster than v77.

Full `canonical-prototype-v78` passed in 160.144 seconds with 18 unique PIDs, 17 Escape
reasons, one window-close reason, complete native-input/readiness/completion evidence,
zero retired raw transport aliases, and zero copied subtrees. The full Activated
observer recorded `0/724/0`, six Activated samples, and two clear samples in
1,557.354 ms; all product and remaining behavior gates passed.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Eleven
acceptance support tests, Flavor zero denies/five existing warnings, guard, init, diff
checks, and process cleanup also passed. Product, Runtime, renderer/GPU, source,
gameplay, synchronization, full process count, and resource ownership are unchanged.
