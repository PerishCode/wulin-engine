# ADR 0157: Object Focus Evidence Single Ownership

- Status: Accepted
- Date: 2026-07-18
- Experiment: 0154 Object Focus Evidence Single Ownership

## Context

The admitted object-focus report emitted the exact recovery action twice: once as top-level
`nativeInput` and once as `focusRecovery.freshAction`. It also emitted three constant booleans that
restated facts already enforced by native message order, focus clock transitions, and final
interaction counts.

## Decision

- Keep the exact recovery action oracle once as top-level `nativeInput`, matching other session
  report ownership.
- Keep only cancellation/resume, missing-target, hold, and focus-clock evidence under
  `focusRecovery`.
- Delete `freshAction`, `missingTargetCommittedBeforeRecovery`,
  `staleObjectIntentsDidNotReachResumedSimulation`, and
  `freshObjectIntentsAfterFocusReadmitted` without replacements.
- Make the central Prototype guard reject all four tokens.
- Preserve every raw launch/completion value and exact behavior gate.
- Add no alias, fallback, decoder, migration layer, product change, or process.

## Consequences

- Recovery input evidence has one owner and one generated copy.
- Focus recovery reports only evidence specific to the focus/missing-target lifetime.
- Exact final counts and clock/message oracles remain the behavioral authority.
- The canonical report becomes smaller without changing product or acceptance cost.

## Evidence

The duplicate minified recovery values were each 349 bytes. `canonical-prototype-v69` passed in
172.610 seconds with a 459,491-byte report, 774 bytes below v68; all four retired tokens had zero
source/report occurrences.

Top-level `nativeInput` retained atomic thread 30036, 0.0025 ms span, exact recovery messages, and
268.3063 ms delayed Escape. `focusRecovery` retained the 0.0012 ms cancelled batch,
258.3602 ms missing-target hold, and exact suspend/resume/reset clock evidence.

Final state remained one ineligible and one committed action, 12 Activated frames, zero Rejected
frames, two suppression frames, exact source-qualified local ID 496, cleared
target/acknowledgement, 336 live frames, zero stalls/render blocks, exit zero, and exactly two
values. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor remained
at zero denies and five existing warnings.
