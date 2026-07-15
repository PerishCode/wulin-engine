# ADR 0055: Planar-First Terrain-Body Advance

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0051 accepts one caller-owned fixed vertical terrain step. ADR 0054 accepts one caller-owned
bounded planar translation while explicitly deferring gravity ordering. A future runtime body or
locomotion controller must not choose that order accidentally.

Vertical-first and planar-first differ on controlled inputs. Vertical-first re-grounds a supported
body at its source before a downhill move, leaving it separated with zero velocity until the next
tick. It can also raise an upward-moving body before the step-limit test, silently turning vertical
velocity into unproven jump/swept clearance. Planar-first keeps the step decision tied to
start-of-tick body geometry and lets the accepted vertical step consume new downhill separation
immediately.

## Decision

- Define one caller-owned combined terrain-body advance that always runs the complete bounded
  planar translation before exactly one fixed vertical step.
- Feed accepted or unchanged-blocked translation output into the vertical integrator. Horizontal
  blocking does not suppress gravity or other vertical progress.
- Reuse the planar destination terrain sample when its candidate position is the vertical input
  position. Query the retained origin only when a blocked nonzero displacement leaves those
  positions different.
- Return complete translation and vertical-step evidence plus one final motion, grounded decision,
  query count, rate, and denominators. Retain no body, input, time, or intermediate state.
- Treat any invalid input/query/arithmetic/contact failure as failure of the whole copied-value
  transaction. Do not return a partial translation or vertical step.
- Keep jump clearance, swept collision, horizontal velocity, input mapping, live schedule driving,
  and actor/body storage outside this decision.

## Consequences

- A later caller has one unambiguous spatial tick primitive without inheriting an order from call
  site convenience.
- Downhill movement begins vertical separation response in the same tick; blocked planar intent can
  still fall, land, or depart at the retained origin.
- Step-up policy does not vary with same-tick vertical integration. Jumping across terrain features
  remains unproven until a discrete or swept clearance experiment defines it explicitly.
- Accepted moves need no duplicate height lookup. Blocked moves may require one additional bounded
  CPU lookup at the origin, which remains observable evidence rather than hidden work.
- The transaction is CPU-only and caller-invoked, so focused tests, one real-process gate, and the
  repository guard are proportionate acceptance evidence. Its gate remains in the canonical
  wrapper for the next candidate that can affect full GPU/lifecycle evidence.

## Evidence

Experiment 0052 implemented the planar-first transaction by directly composing the accepted
translation and vertical integrator. All 49 focused runtime tests passed. Six direct advance tests
prove same-tick downhill response, blocked-horizontal vertical progress, signed seams,
validation/overflow rollback, and exact one/two-query order.

A final 20.52-second fresh-process gate exercised a real 128-Q16 terrain rise. Accepted uphill and
downhill paths reused one destination sample; blocked uphill used exactly destination then retained
origin while preserving planar input. Two differently grouped 60-step runs and immediate direct
replay produced SHA-256
`7463970a8748a5aa02567c2ea94b64d2b8e527968360d30b34cef2568db02142` with unchanged schedule and
presentation state and zero non-CPU work. `runseal :guard` passed in 5.3 seconds with zero Flavor
denies. The gate is live in the canonical wrapper; no full GPU/lifecycle run was charged to this
CPU-only composition.
