# ADR 0083: Live Documentation Authority

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0080 Mandatory Live Documentation Authority Cleanup

## Context

`docs/architecture/repository-model.md` owns stable repository structure and dependency direction,
but it also accumulated a detailed `State` section. That snapshot stopped at Experiment 0066 and
now contradicts accepted behavior: it describes a public actor projection, zero-command prototype,
unpromoted camera/nonzero motion/GPU actor binding, and only one canonical operator after those
surfaces were retired or promoted.

`AGENTS.md` section 4 already owns the current runtime boundary and is maintained with core
ownership changes. Keeping a second stage ledger creates competing current truth; repeatedly
refreshing it would institutionalize that duplication. Separately, Experiment 0079 established
`runseal :prototype` as the sole live manual prototype entry, so direct Sidecar commands in live
operator documentation would recreate the acceptance-residue/setup ambiguity it removed.

## Decision

- Delete the stage-specific state ledger from the repository model. Keep that document limited to
  stable dependency direction, top-level ownership, naming, experiment promotion, and source/
  generated-data rules.
- Add a short authority statement linking changing runtime truth to `AGENTS.md` section 4.
- Add one focused live-operator-surface guard. It retains exact wrapper-set ownership, rejects a
  repository-model `State` section or experiment cutoff, requires the current-boundary link,
  requires the maintained prototype start command in `README.md` and `AGENTS.md`, and rejects
  direct prototype Sidecar lifecycle commands there.
- Do not scan or rewrite historical ADR/experiment narratives and do not add an alias, redirect, or
  second current-state document.

## Consequences

- Changing runtime capability has one current documentation owner instead of two drifting ledgers.
- The repository model becomes stable enough to describe structure without per-experiment edits.
- Live product instructions cannot silently bypass the maintained self-contained prototype setup;
  Sidecar remains the wrapper's underlying lifecycle implementation rather than an operator entry.
- Runtime/application behavior, prototype acceptance support, Sidecar lifecycle, generated output,
  and historical evidence remain unchanged.

## Evidence

Experiment 0080 removed 85 stage-snapshot lines and replaced them with eight stable authority
lines. Focused formatting/type checks and positive source scans passed. A deliberate direct
prototype Sidecar start command made the maintained guard fail in 0.7 seconds before build/test
work. `runseal :init` and the single merge-checkpoint `runseal :guard` passed with zero Flavor
denies and five pre-existing warnings. No runtime, application, GPU, format, Sidecar, generated
output, or canonical acceptance implementation changed.
