# ADR 0159: Native Retained-Target Outside-Radius Admission

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0156 Native Retained-Target Outside-Radius Admission

## Context

Prototype object-action policy already returned typed `OutsideRadius`, but the real-process matrix
proved only MissingTarget, OutsideFacing, successful activation/consumption, and capacity rejection
directly. The existing Rejected child retained a resolved source-qualified target after its 12-frame
red acknowledgement, so it could prove the missing outcome without another target scan or process.

A first attempt split D-down and D-up across native helpers. It proved the outcome but allowed
preparation of the second helper to extend the actual held interval beyond its reported sleep.
Acceptance needs the complete movement-driving interval to be observable.

## Decision

- Extend the existing Rejected object-feedback child after its current OutsideFacing lifetime.
- Release F/Enter, post D-down, wait at least 500 ms, and post D-up inside one native helper and one
  exact PID/window/thread.
- Submit Enter alone after motion; do not post F or perform another nearest scan.
- Keep the original source-qualified target resolved and compute its final delta and Q18 squared
  distance independently from cooked-source and signed terrain-position evidence.
- Require at least ten positive-X 32-Q9 Walk steps, zero Z translation, stable actor identity and
  presentation, and zero final vertical velocity.
- Require exact final `committed=0`, `ineligible=2`, `Activated=0`, `Rejected=12`, `suppression=0`,
  and `renderBlock=0` totals.
- Advance complete Prototype acceptance directly to v71.
- Add no product state or behavior, process, scan, retry, fallback, compatibility alias, telemetry,
  product delay, relaxed threshold, Runtime/GPU/source/synchronization owner, or workspace resource
  cleanup.

## Consequences

- Current `OutsideRadius` has a direct native real-process witness over a retained target.
- The final Enter-only action cannot replace its target through F acquisition.
- Exact source identity plus final Q9/Q18 proximity distinguishes range rejection from missing,
  unavailable, replaced, outside-window, facing, or capacity outcomes.
- Motion timing is owned by one measured down/wait/up helper interval rather than implicit
  cross-helper held state.
- The existing Rejected process count and 12-frame feedback lifetime remain fixed.
- Experiment 0156 performs no resource cleanup; the scheduled compatibility/resource boundary
  remains Experiment 0160.

## Evidence

The clean `canonical-prototype-v71` run passed in 172.403 seconds with a 463,475-byte report. PID
23372, window `25888704`, and thread 32748 posted the initial F/Enter batch in 0.001 ms, held the
rejection for 265.0754 ms, posted the F-up/Enter-up/D-down prefix in 0.0025 ms, held D for 513.2267
ms, and then posted D-up. The final batch contained Enter without F and delayed Escape by 263.1885
ms.

The actor moved exactly `(+960,0)` Q9, or 30 positive-X Walk steps, then returned to Survey with the
same generation-1 handle and zero step velocity. The same resolved source-qualified local ID 495
remained under publication token 2. Its final delta was `(-1184,-32)` Q9 and exact squared distance
was 1,402,880 Q18, beyond the inclusive 262,144-Q18 radius.

The child retained zero commits, exactly two ineligible actions, 12 Rejected frames, zero
Activated/suppression/render-block frames, 195 target frames, and 260 live frames. Exit code,
two-value output, stderr, trailing output, and process cleanup were exact. All 103 engine-runtime,
48 Prototype, and 20 reference-host tests passed; Flavor retained zero denies and five existing
warnings.
