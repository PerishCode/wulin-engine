# Experiment 0080: Mandatory Live Documentation Authority Cleanup

Status: Accepted

## Hypothesis

The repository can delete its stale 0066-era duplicate runtime-state ledger, retain one explicit
authority for changing live capability truth, and prevent direct prototype Sidecar commands from
returning to live operator documentation. This can be done without rewriting historical evidence
or changing runtime, application, cook, lifecycle, or acceptance behavior.

## Scope

Remove the `## State` section from `docs/architecture/repository-model.md`. Replace it with a short
authority statement: `AGENTS.md` section 4 owns the changing current runtime boundary, while the
repository model owns stable dependency, directory, naming, promotion, and generated-data rules.

Add one focused live-operator-surface guard owner that retains the existing exact wrapper-set check,
rejects a stage-specific current-state ledger or experiment cutoff in the repository model, rejects
direct `sidecar.prototype.toml` lifecycle commands in `README.md` and `AGENTS.md`, and requires both
live operator documents to retain `runseal :prototype start`.

Historical experiment/ADR prose, runtime/application code, prototype acceptance support,
`sidecar.prototype.toml`, generated bootstrap ownership, and product behavior are out of scope.

## Workload

1. Inventory the repository model against the accepted boundary and record concrete contradictory
   claims rather than refreshing them in place.
2. Delete the duplicate state snapshot and retain only one link to the current runtime authority.
3. Run the focused live-documentation gate against the clean tree.
4. Temporarily introduce one direct prototype Sidecar command into a live operator document and
   require the maintained guard to fail before build/test work; remove it and require the gate to
   pass again.
5. Run the authoritative repository gate.

## Controlled Variables

- `AGENTS.md` section 4 remains the sole detailed current runtime boundary.
- Stable repository dependency direction, top-level ownership, naming, promotion, and generated
  data policy remain byte-for-byte unchanged.
- Historical ADR and experiment documents remain settled evidence and are not scanned as live
  operator documentation.
- No wrapper lifecycle, Sidecar manifest, cook output, bootstrap schema/path, Rust source, shader,
  GPU resource, synchronization, or acceptance report changes.

## Metrics

- Lines and contradictory stage claims removed from the duplicate state ledger.
- Count of changing live runtime authorities after cleanup.
- Positive and deliberate-negative live-documentation gate outcomes.
- `runseal :guard` result and Flavor deny/warning counts.
- Runtime, application, GPU, format, generated-output, and lifecycle deltas.

## Acceptance Criteria

- `repository-model.md` contains no `## State`, `Experiments through NNNN` cutoff, or duplicated
  capability snapshot and points explicitly to `AGENTS.md` section 4 for changing live truth.
- `README.md` and `AGENTS.md` retain the maintained `runseal :prototype start` entry and contain no
  direct prototype Sidecar lifecycle command.
- The focused gate passes on the accepted sources and fails on a deliberately reintroduced direct
  Sidecar command before expensive validation begins.
- `runseal :init` and `runseal :guard` pass. No runtime/application behavior, acceptance workflow,
  Sidecar lifecycle, generated output, or historical evidence changes.

## Reference Environment

The experiment uses the repository-pinned Deno/Rust toolchains and current Windows checkout. Its
proof is static except for the unchanged repository validation workflow.

## Evidence

- The audit found five concrete contradictions in the duplicate state ledger: the retired public
  actor projection, retired zero-command-only prototype, and already promoted nonzero locomotion,
  actor-relative camera, and GPU actor candidate. It also found the 0066/0069 cutoff and obsolete
  single-operator description.
- The repository model removed 85 stage-snapshot lines and added eight stable authority lines. Its
  dependency direction, ownership table, naming, promotion, and generated-data sections are
  unchanged.
- Focused Deno formatting and type checking passed. Static source scans found no repository-model
  state heading/cutoff, found `runseal :prototype start` in both live operator documents, and found
  no direct prototype Sidecar lifecycle command.
- A deliberate `sidecar start --config sidecar.prototype.toml` insertion in `README.md` made the
  maintained guard fail in 0.7 seconds at the live operator surface, before Rust build/test
  work. Removing the line restored the clean source.
- The first merge-checkpoint guard exposed that two new orchestration lines pushed the wrapper over
  its 500-line source boundary. The pre-existing exact wrapper-set behavior moved intact beside the
  new documentation checks in the focused live-operator-surface owner; no check was deleted or
  weakened.
- `runseal :init` and the single merge-checkpoint `runseal :guard` passed. Guard reported zero
  Flavor denies and the same five pre-existing warnings.
- No Rust source, runtime/application behavior, renderer/GPU resource, shader, format, cook,
  generated output, Sidecar manifest/lifecycle, or canonical acceptance support changed. A GPU or
  canonical-prototype run therefore could not add evidence for this static cleanup.

## Conclusion

Accepted. Changing live capability now has one documentation authority, the stable repository
model no longer carries a stale stage snapshot, and live prototype instructions cannot bypass the
maintained self-contained operator.
