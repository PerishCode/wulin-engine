# ADR 0117: Native Midair Jump Rejection

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0114 Native Midair Jump Rejection

## Context

The Jump policy accepts Space only from its last exact committed grounded witness. Unit tests prove
midair presses are ignored, and Experiment 0113 proves a new press after a complete landing is
admitted. The complementary live boundary remains unproven: a second real native press before
landing must not add another impulse or retained action.

## Decision

- Maintain one real-process post-readiness session that posts the initial Space press, a bounded
  midair Space release/press, and a later bounded Escape from one exact-window helper.
- Use one monotonic helper clock to bound both native post intervals before the existing 48-step
  landing.
- Require final motion to satisfy only the existing single-impulse discrete trajectory. Add no
  Jump outcome field, event stream, inspect route, input history, or product clock/schedule state.

## Consequences

- Midair duplicate rejection gains one exact live process proof beside the accepted post-landing
  readmission proof.
- The harness gains reusable delayed-exit timing evidence; product input and Jump policy remain
  unchanged.
- This decision does not authorize double jump, coyote time, action buffering beyond the existing
  bit, jump presentation, gameplay effects, networking, or Runtime/GPU/resource changes.

## Evidence

Experiment 0114 passed `canonical-prototype-v31` in 102.146 seconds. One exact visible process
window received the first Space press, a second Space plus W after 208.749 ms, and Escape after
another 207.008 ms. Final vertical motion derived exactly the original single impulse at step 25:
velocity -106, rise 51,050, and center `141824 -> 192874`. The same midair batch produced 12 exact
Walk steps and Z delta -384 Q9, proving it reached product admission. Clock discontinuity/stall
counts, actor identity/region/X/shape, object state, render-block count, and the two-value clean
completion remained exact.
