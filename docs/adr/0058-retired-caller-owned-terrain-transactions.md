# ADR 0058: Retired Caller-Owned Terrain Transactions

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADRs 0051, 0054, and 0055 established pure copied-value vertical, planar, and combined terrain
transactions. ADRs 0056 and 0057 then established one runtime-owned retained body and the only live
read-compute-commit path that consumes those transactions. The original inspect commands, workbench
dispatch, public `Runtime` forwarders, and recurring Runseal gates now duplicate the accepted stored
path. Keeping them would preserve caller-owned mutation authority only as historical baggage.

The canonical wrapper also accumulated deterministic source setup alongside acceptance ordering.
That pushed the wrapper above the normal source-length limit and required a temporary 520-line
exception even though setup has one coherent typed responsibility.

## Decision

- Delete `canonical.terrain.body.step`, `.translate`, and `.advance` completely, including their
  protocol variants and payloads, workbench branches, public `Runtime` forwarders, dedicated
  Runseal support files, wrapper calls, and evidence fields. Add no aliases or fallback routes.
- Retain the pure terrain motion, translation, and planar-first modules and their focused tests.
  They remain private implementation authorities composed by retained advance, not parallel live
  mutation surfaces.
- Keep retained spawn/read/despawn and `canonical.terrain.body.retained.advance` as the complete
  live terrain-body control boundary until a later experiment proves schedule driving.
- Extract deterministic tests/builds, source cooking, source-identity assertions, and controlled
  corruption into one typed canonical setup owner. The direct wrapper continues to expose one
  linear gate order and does not invoke historical wrappers.
- Remove the temporary wrapper source-length exception and add stable guards for the deleted files,
  symbols, and verbs.

## Consequences

- Hosts can no longer bypass runtime-owned retained state through copied-value diagnostic commands.
  Historical experiment and ADR text remains evidence, not compatibility surface.
- The pure spatial contracts remain independently testable and reusable inside the retained
  transaction without expanding public runtime mutation authority.
- Canonical setup can evolve as one owner while acceptance order stays readable and under the
  default source-length rule.
- The next schedule-driving experiment starts from one retained mutation path. It still must prove
  elapsed-time ownership, batch failure semantics, focus/stall policy, and input independence.

## Evidence

Experiment 0055 removed three complete live command chains and 1,018 physical lines of dedicated
Runseal support. The direct wrapper fell from 508 to 405 physical lines (482 to 382 measured source
lines), and its temporary 520-line Flavor override was deleted. A stable repository guard rejects
restoration of any removed support file, command, protocol variant, runtime forwarder, error prefix,
or gate name.

The extracted setup completed its selected tests/build, cooked all eleven deterministic pack and
corruption artifacts, and passed the existing source-identity and corruption checks. All 56 focused
engine-runtime tests and the workbench check passed. A single fresh-process gate completed in 10.74
seconds: all ten historical verbs, including the three retired here, returned `unknown_event`; the
current retained route rejected stale generation and a negative limit before terrain lookup, kept
the stored body byte-for-byte unchanged, and despawned it exactly. `runseal :init` and
`runseal :guard` passed. The long canonical workflow was intentionally not run because this change
does not modify production frames, renderer/GPU resources, synchronization, or lifecycle behavior.
