# ADR 0152: Native Diagonal Run Stop

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0149 Native Diagonal Run Stop

## Context

The maintained diagonal-Run process proved atomic Shift/W/A admission followed by exact
retained-left Run after W release. It still exited while Shift+A remained held, so the same real
process did not prove that releasing the last locomotion direction transitions to stationary
Survey while the gait modifier remains held and committed facing is retained. A stronger gate must
preserve the accepted exact movement decomposition and add no intermediate product state.

## Decision

- Reuse the existing diagonal-Run child and report schema.
- Keep Shift/W/A as one exact-window-thread atomic prefix and retain the accepted delayed W release.
- After another at least 250 ms, release A while retaining Shift; wait at least 250 ms in the
  resulting stationary state before the existing Escape completion.
- Preserve the unique positive `(-45,-45)` diagonal plus `(-64,0)` left Run decomposition.
- Require final Survey clip 0/yaw 32,768, actor lifetime, clock continuity, idle object state, zero
  render blocks, and exact two-value cleanup.
- Advance the complete Prototype workflow to v64.
- Add no child, product output/state, intermediate query, retry, telemetry, threshold relaxation,
  Runtime behavior, or renderer/GPU/source/resource ownership.

## Consequences

- The real host boundary now proves the complete Shift/W/A direction-key lifetime from atomic
  admission through partial release to Shift-only stationary cleanup.
- The stationary tail preserves the prior movement decomposition while making the last direction
  release observable through exact Survey presentation and retained yaw.
- The original diagonal-to-left Run proof remains live as the movement prefix of the stronger
  session.

## Evidence

`canonical-prototype-v64` passed first-run in 163.101 seconds with a 456,292-byte report. PID 27272
used window thread 30344, 0.0016/0.0010 ms Shift/W/A intervals, and a 0.0026 ms atomic span. W-up
followed after 273.5453 ms, A-up after another 259.1053 ms, and Escape after a 266.3092 ms
stationary hold.

Final local position `(-1680,-720)` Q9 decomposed exactly into 16 diagonal and 15 left-only Run
steps. Actor identity/region/shape remained stable, vertical velocity and render blocks stayed
zero, and final presentation was Survey clip 0/yaw 32,768 with epoch `1 -> 79`. Clock Ready/sample
advanced `2/3 -> 91/92`; object state remained idle and process output contained exactly two values
with exit code zero and empty stderr.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero
denies and five existing warnings.
