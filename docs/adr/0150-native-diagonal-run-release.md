# ADR 0150: Native Diagonal Run Release

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0147 Native Diagonal Run Release

## Context

The maintained real diagonal-Run process proved atomic Shift/W/A ingestion and exact 45-Q9
normalized movement but exited while all keys remained held. It did not prove a live Run direction
transition after one diagonal component was released. The existing two-value session exposes no
intermediate actor snapshot, so any extension must remain exactly recoverable from final state.

## Decision

- Reuse the existing diagonal-Run child and report schema.
- Keep Shift/W/A as one exact-window-thread atomic prefix.
- After at least 250 ms, release W while retaining Shift+A; after another at least 250 ms, use the
  existing Escape completion.
- Require final displacement to decompose uniquely into positive counts of `(-45,-45)` diagonal
  steps and `(-64,0)` left steps.
- Require final Run clip 2/yaw 32,768, actor lifetime, clock continuity, idle object state, zero
  render blocks, and exact two-value cleanup.
- Advance the complete Prototype workflow to v62.
- Add no child, product output/state, intermediate query, retry, telemetry, threshold relaxation,
  Runtime behavior, or renderer/GPU/source/resource ownership.

## Consequences

- The real host boundary now proves a same-session diagonal-to-cardinal Run turn after key release.
- The orthogonal fixed components make both phases observable from final state without adding a
  third product value or inspect path.
- The original atomic diagonal-Run admission remains live as the first prefix of the stronger
  session.

## Evidence

`canonical-prototype-v62` passed first-run in 169.946 seconds with a 455,214-byte report. PID 15960
used window thread 30396, 0.0015/0.0018 ms Shift/W/A intervals, and a 0.0033 ms atomic span. W-up
followed after 268.9852 ms; Escape followed after another 262.2319 ms.

Final local position `(-1744,-720)` Q9 decomposed exactly into 16 diagonal and 16 left-only Run
steps. Actor identity/region/shape remained stable, vertical velocity and render blocks stayed
zero, and final presentation was Run clip 2/yaw 32,768 with epoch `1 -> 48`. Clock Ready/sample
advanced `2/3 -> 78/79`; object state remained idle and process output contained exactly two values
with exit code zero.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero
denies and five existing warnings.
