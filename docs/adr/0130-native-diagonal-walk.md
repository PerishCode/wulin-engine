# ADR 0130: Native Diagonal Walk

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0127 Native Diagonal Walk

## Context

Walk diagonal normalization, the A-key path, and forward-left yaw had exact unit coverage but no
dedicated real native process proof. The first expanded run also showed that starting a PowerShell
helper before its child did not guarantee the helper had completed native type preparation before
a warm Prototype published readiness.

## Decision

- Add one atomic exact-window W/A startup batch, delayed Escape, and exact two-value diagonal Walk
  session.
- Require equal negative 23-Q9 X/Z components, clip 1, yaw 40,960, stable animation epoch, default
  orbit, clock continuity, zero blocks, and idle object state.
- Emit one fixed helper-ready marker after PowerShell `Add-Type` and before window search.
- Await that marker before spawning every startup-input Prototype, then require the helper's final
  actual window PID to equal the spawned child.
- Centralize startup requests and delete the superseded duplicate start-only action and Run
  sequence helpers.

## Consequences

- A-key ingestion, Walk diagonal normalization, and forward-left presentation have exact
  real-process evidence.
- Warm startup input ordering is a handshake rather than an assumption about concurrent process
  scheduling.
- Acceptance gains no retry, product delay, threshold relaxation, event stream, journal, copied
  state, or extra product output.
- Product input/locomotion/presentation, Runtime, renderer/GPU resources, synchronization, and
  object policy remain unchanged.

## Evidence

The first start-only `canonical-prototype-v42` run reproduced the old startup race at the existing
Run gate. The first run after the helper-ready handshake passed in 156.480 seconds with every
previous gate. After the helper process/evidence owner was split to satisfy source-size policy, the
final-worktree run passed again in 157.404 seconds. PID 1852 queued W/A on thread 2,472 in a 0.0013
ms atomic span. Readiness was `(-23,-23)`, Walk clip 1/yaw 40,960/epoch 5; completion was
`(-299,-299)`, 12 further exact diagonal steps with epoch still 5. Escape measured 213.7342 ms,
clocks advanced `4/5 -> 70/71`, render blocks stayed zero, object state stayed idle, and output
contained exactly two values. Flavor reported zero denies and `runseal :init` passed.
