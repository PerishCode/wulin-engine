# ADR 0068: Neutral Canonical Operator Identity

- Status: Accepted
- Date: 2026-07-15
- Supersedes: ADR 0063 operator report naming clause
- Superseded by: None

## Context

The only canonical acceptance wrapper has continued to identify its report and output collection as
Experiment 0060 while its live workload evolved through Experiments 0061-0064. That name is now
false ownership: it makes current cooked sources and evidence look like one historical cleanup and
invites future stages either to overwrite that history or add another compatibility collection.

The setup and capture functions are already generic. Only the live wrapper constants and current
operational evidence path carry the obsolete label.

## Decision

- Give the sole direct wrapper revision `canonical-runtime-v1` and collection `canonical-runtime`.
- Let the existing generic setup derive `out/cooked/canonical-runtime/` and
  `out/captures/canonical-runtime/acceptance.json` without a special case.
- Update current operational documentation to the neutral evidence directory and current actor
  lifecycle/simulation vocabulary.
- Add a stable guard for the exact single revision/collection and absence of the old label from the
  live wrapper.
- Keep historical Experiment 0060 and ADR 0063 documents unchanged. Add no old-path fallback,
  report copier, alias wrapper, or migration utility.
- Preserve the complete canonical gate order and all runtime/GPU behavior.

## Consequences

- Current acceptance evidence has truthful runtime ownership rather than historical experiment
  ownership.
- Future experiments extend one neutral operator instead of renaming or layering new wrappers.
- Existing ignored local output under the old directory is disposable and has no compatibility
  status.

## Evidence

Experiment 0065 found only the two live wrapper constants and one current documentation path. A
focused static/operator gate proved exact neutral identity, old-label absence, strict help/argument
behavior, and complete TypeScript checking in 2.1 seconds. `runseal :init` and the 14.33-second
repository guard passed with zero denies.

No process, source cook, frame, GPU, resource, synchronization, or lifecycle work was required. The
long canonical workflow was not run because the workload implementation and order are unchanged.
