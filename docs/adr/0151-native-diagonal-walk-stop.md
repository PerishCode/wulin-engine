# ADR 0151: Native Diagonal Walk Stop

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0148 Native Diagonal Walk Stop

## Context

The maintained diagonal-Walk process proved atomic W/A admission followed by exact retained-left
Walk after W release. It still exited while A remained held, so the same real process did not prove
that releasing the last locomotion direction transitions to stationary Survey while retaining the
committed facing. A stronger gate must preserve the accepted exact movement decomposition and add
no intermediate product state.

## Decision

- Reuse the existing diagonal-Walk child and report schema.
- Keep W/A as one exact-window-thread atomic prefix and retain the accepted delayed W release.
- After another at least 250 ms, release A; wait at least 250 ms in the resulting stationary state
  before the existing Escape completion.
- Preserve the unique positive `(-23,-23)` diagonal plus `(-32,0)` left Walk decomposition.
- Require final Survey clip 0/yaw 32,768, actor lifetime, clock continuity, idle object state, zero
  render blocks, and exact two-value cleanup.
- Advance the complete Prototype workflow to v63.
- Add no child, product output/state, intermediate query, retry, telemetry, threshold relaxation,
  Runtime behavior, or renderer/GPU/source/resource ownership.

## Consequences

- The real host boundary now proves the complete W/A direction-key lifetime from atomic admission
  through partial release to final stationary cleanup.
- The stationary tail preserves the prior movement decomposition while making the last release
  observable through exact Survey presentation and retained yaw.
- The original diagonal-to-left Walk proof remains live as the movement prefix of the stronger
  session.

## Evidence

`canonical-prototype-v63` passed first-run in 163.081 seconds with a 455,824-byte report. PID 16472
used window thread 23708 and a 0.0013 ms W/A atomic interval/span. W-up followed after 265.2565 ms,
A-up after another 261.3792 ms, and Escape after a 259.2750 ms stationary hold.

Final local position `(-857,-345)` Q9 decomposed exactly into 15 diagonal and 16 left-only Walk
steps. Actor identity/region/shape remained stable, vertical velocity and render blocks stayed
zero, and final presentation was Survey clip 0/yaw 32,768 with epoch `1 -> 79`. Clock Ready/sample
advanced `2/3 -> 93/94`; object state remained idle and process output contained exactly two values
with exit code zero and empty stderr.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero
denies and five existing warnings.
