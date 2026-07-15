# ADR 0054: Bounded Terrain-Body Translation

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0052 established exact canonical planar position translation, while ADR 0049 and ADR 0051
established exact caller-owned terrain contact and fixed vertical motion. These authorities can
already describe a body at a destination, but blindly accepting the destination's resolved contact
would permit an arbitrarily high terrain rise to lift the body. That would silently turn
non-penetration into unlimited step climbing before any locomotion policy is proven.

The next dependency is therefore not horizontal velocity or input. It is one explicit boundary
between exact planar displacement/contact and future gameplay step tuning.

## Decision

- Add one caller-owned bounded planar terrain-body translation transaction. Its inputs are copied
  body motion, exact signed X/Z Q9 displacement, and an explicit nonnegative Q16 step-up limit.
- Split execution transactionally: validate the limit and construct the normalized candidate first;
  only then query the committed terrain once at the candidate position and resolve exact contact.
- Accept a candidate if and only if its upward correction is no greater than the supplied limit.
  Accepted output uses the resolved candidate body and preserves input vertical velocity.
- When the correction exceeds the limit, report the candidate/contact evidence but return input
  motion unchanged as output. Do not clamp, partially translate, search for an alternate position,
  or retain a fallback result.
- Accept separated downhill candidates without downward snapping. This transaction proves bounded
  non-penetration composition, not grounding or locomotion ordering.
- Expose one strict diagnostic workbench operation for process evidence. It owns no body identity,
  live state, clock consumption, input mapping, frame action, or renderer behavior.

## Consequences

- A future locomotion experiment can supply displacement and step policy without bypassing exact
  coordinates or contact, while this experiment does not preselect speed, acceleration, slope, or
  input semantics.
- A caller can observe why a move was blocked through candidate/contact evidence, but the reusable
  output contract remains atomic: fully accepted resolved motion or exactly unchanged input.
- Downhill movement can leave the body separated until later vertical integration resolves it;
  ordering between planar translation and gravity remains an explicit future decision.
- Footprint collision, slopes, sweep/tunneling, object collision, horizontal velocity, body stores,
  actor presentation, and gameplay control remain unproven and cannot depend on this ADR.
- Because the change is CPU-only and adds no frame/GPU/resource/lifecycle behavior, focused tests,
  one real-process gate, and the repository guard are the proportionate acceptance path. Its gate is
  retained in the canonical wrapper for the next change that legitimately requires a full run.

## Evidence

Experiment 0051 implemented the transaction without an alternate coordinate/contact path or
persistent prepared state. All 43 focused runtime tests passed. Its six direct tests prove exact
limit decisions, blocked input/output identity, no downhill snap, velocity preservation, signed
seams, partition convergence, validation-before-query, overflow, and transactional failures.

A 13.90-second fresh-process gate found and exercised a real 128-Q16 terrain rise, accepted it at
equal and greater limits, blocked it below the limit, crossed a signed region seam, preserved
schedule/presentation state, and reproduced SHA-256
`391d3dde3b853590da02f45137cd554bd430be7b0004c1dc639dac2cd2d6d23a`. All non-CPU work counters
were zero. `runseal :guard` passed in 4.3 seconds with zero Flavor denies. The gate is part of the
live canonical wrapper; no full GPU/lifecycle run was charged to this CPU-only transaction.
