# ADR 0156: Native Missing-Target Action Recovery

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0153 Native Missing-Target Action Recovery

## Context

Pure product tests established that ineligible object-action attempts consume their intent, but
the maintained Activated process moved directly from focus cancellation to a valid F/Enter action.
It did not prove that a real missing-target Enter press is consumed once and leaves the later
exact-object action lifetime usable.

## Decision

- Reuse the existing Activated object-focus child and its stale F/Enter cancellation,
  resume/reset, source, actor, feedback, suppression, and completion authorities.
- After resume, post Enter-down with no target and wait at least 250 ms.
- Then atomically post Enter-up/F-down/Enter-down on the same PID/window/thread and use the existing
  250 ms delayed Escape.
- Require exactly one ineligible and one committed action, 12 Activated frames, zero Rejected
  frames, at least one suppression frame, exact source-qualified consumption/exclusion, and a
  cleared final target.
- Keep missing-target and recovery transport validation in a bounded object input-gate module.
- Add no product queue, retry, polling, telemetry, acknowledgement timer, state mutation, process,
  output, alias, fallback, or compatibility decoder.

## Consequences

- The real process now proves that `MissingTarget` is a one-shot ineligible outcome.
- Releasing the prior Enter key in the atomic recovery prefix establishes the next normalized Enter
  press without adding a product action state.
- The valid action retains the same renderer feedback, consumption, exclusion, and suppression
  lifetime.
- Product behavior and acceptance process count remain unchanged.

## Evidence

`canonical-prototype-v68` passed in 175.470 seconds with a 460,265-byte report. PID 2580 used
window `142084402` and thread 22988. Missing-target Enter remained admitted for 263.0453 ms; the
later Enter-up/F-down/Enter-down prefix spanned 0.0021 ms and Escape followed after 267.9987 ms.

Completion recorded exactly one ineligible and one committed action, 12 Activated/object-target
frames, zero Rejected frames, and two suppression frames. Consumed/excluded identity was source
namespace `99d9511b8cea49a59d771d97874d56bb7790a79c880490353852bc75aa4fd94d`, owner region
`(1099511627776,-1099511627776)`, local ID 496; final target/acknowledgement were null.

The clock recorded one suspend/resume, two total resets, zero stalls, Ready/sample `260/343`, and
343 live frames with zero render blocks. Exit was zero with exactly two values and empty
stderr/trailing output. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed;
Flavor remained at zero denies and five existing warnings.
