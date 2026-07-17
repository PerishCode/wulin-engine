# ADR 0138: Retired Startup Report Branches

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0135 Retired Startup Report Branches

## Context

Experiment 0130 deleted the `startupNativeInput` report field and the complete startup-action
acceptance route. Eight current native-session oracles nevertheless retained independent
`Object.hasOwn` rejections for that impossible field. These checks no longer protected a live
boundary; they duplicated settled compatibility history inside current semantic owners.

## Decision

- Delete the retired report-field branch from camera repeat/re-press, Run release/re-press,
  opposite locomotion, diagonal Walk/Run, and forward release.
- Keep their live native transport, timing, actor, camera, presentation, object, and completion
  oracles unchanged.
- Make the existing Prototype-session guard the sole removal authority by scanning all eight
  current source owners for the retired token.
- Retain no alias, fallback, decoder, compatibility report, or replacement runtime behavior.

## Consequences

- Current session oracles contain only contracts that can occur in the v50 workflow.
- A return of the retired field or any per-owner branch fails the central static guard.
- Product behavior, process count, two-value framing, Runtime, renderer/GPU resources, source
  formats, and synchronization remain unchanged.
- Historical rationale remains in Experiment 0130 / ADR 0133 and this cleanup record, not in live
  execution paths.

## Evidence

The audit found and deleted exactly eight duplicate branches. Only two forbidden-token references
remain in the central guard. `canonical-prototype-v50` passed in 174.239 seconds with all 16 normal
native sessions producing exactly two values; representative forward-release, focus-discontinuity,
and object-feedback invariants remained exact. The ignored report was 446,503 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. Flavor remained at zero
denies and five existing warnings.
