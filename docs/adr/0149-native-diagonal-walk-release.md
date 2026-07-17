# ADR 0149: Native Diagonal Walk Release

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0146 Native Diagonal Walk Release

## Context

The maintained real diagonal-Walk process proved atomic W/A ingestion and exact 23-Q9 normalized
movement but exited while both keys remained held. It did not prove a live locomotion direction
transition after one diagonal component was released. The existing two-value session exposes no
intermediate actor snapshot, so any extension must remain exactly recoverable from final state.

## Decision

- Reuse the existing diagonal-Walk child and report schema.
- Keep W/A as one exact-window-thread atomic prefix.
- After at least 250 ms, release W while retaining A; after another at least 250 ms, use the
  existing Escape completion.
- Require final displacement to decompose uniquely into positive counts of `(-23,-23)` diagonal
  steps and `(-32,0)` left steps.
- Require final Walk clip 1/yaw 32,768, actor lifetime, clock continuity, idle object state, zero
  render blocks, and exact two-value cleanup.
- Advance the complete Prototype workflow to v61.
- Add no child, product output/state, intermediate query, retry, telemetry, threshold relaxation,
  Runtime behavior, or renderer/GPU/source/resource ownership.

## Consequences

- The real host boundary now proves a same-session diagonal-to-cardinal locomotion turn after key
  release.
- Coprime fixed components make both phases observable from final state without adding a third
  product value or inspect path.
- The original atomic diagonal admission remains live as the first prefix of the stronger session.

## Evidence

`canonical-prototype-v61` passed first-run in 169.600 seconds with a 454,638-byte report. PID 26360
used window thread 16028 and a 0.0015 ms W/A atomic interval/span. W-up followed after 264.2536 ms;
Escape followed after another 260.9478 ms.

Final local position `(-848,-368)` Q9 decomposed exactly into 16 diagonal and 15 left-only Walk
steps. Actor identity/region/shape remained stable, vertical velocity and render blocks stayed
zero, and final presentation was Walk clip 1/yaw 32,768 with epoch `1 -> 63`. Clock Ready/sample
advanced `2/3 -> 92/93`; object state remained idle and process output contained exactly two
values with exit code zero.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero
denies and five existing warnings.
