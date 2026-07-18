# ADR 0153: Retired Post-Readiness Report Flags

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0150 Retired Post-Readiness Report Flags

## Context

Experiment 0130 removed startup-action acceptance and established one structural rule: the shared
session runner first parses exact product readiness and only then dispatches an action to that
known child PID. Eleven current report owners nevertheless retained 16 static
`actionAfterReadiness: true` fields plus 11 positive source-shape expectations. The fields copied
execution ordering but did not validate any independent product or semantic outcome.

## Decision

- Delete all 16 report producers and all 11 positive guard expectations.
- Keep the shared readiness-before-action execution order as the sole live authority.
- Make the existing Prototype session guard scan all 11 current owners and reject restoration of
  the retired token.
- Advance complete Prototype acceptance directly to v65.
- Add no replacement field, compatibility alias, optional decoder, registry, fallback, retry,
  product delay, or relaxed threshold.
- Change no product, Runtime, renderer/GPU, source, synchronization, resource, or process-count
  owner.

## Consequences

- Reports retain exact native transport, timing, actor, presentation, camera, object, clock, frame,
  completion, and cleanup facts without a redundant ordering boolean.
- The historical startup-action distinction has no executable or report compatibility surface.
- One static removal authority covers all current producers.
- Experiment 0150 performs no workspace resource cleanup; the scheduled resource boundary remains
  Experiment 0160.

## Evidence

The cleanup deleted 27 source occurrences: 16 report fields and 11 positive expectations. The sole
remaining token is the central forbidden check across all 11 current owners.

`canonical-prototype-v65` passed first-run in 162.812 seconds. Its 455,655-byte report contains zero
retired-field occurrences, down from 18 occurrences and 456,292 bytes in v64. All 16 normal
sessions retained two output values, exit zero, empty stderr, and empty trailing output; the
finite-boundary process retained two values and exit zero.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor retained zero
denies and five existing warnings. The existing Runtime v20 report already contained zero
occurrences because its Prototype checkpoint uses readiness-only captures.
