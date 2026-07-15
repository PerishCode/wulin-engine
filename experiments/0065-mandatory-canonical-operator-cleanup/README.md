# Experiment 0065: Mandatory Canonical Operator Cleanup

Status: Accepted

## Hypothesis

The sole canonical acceptance operator can shed its obsolete Experiment 0060 revision, cook/capture
collection, and evidence path in favor of one neutral current identity without changing commands,
gate order, runtime behavior, or GPU evidence semantics, and a stable repository guard can prevent
that historical ownership from returning.

## Scope

Replace the live wrapper revision `mandatory-simulation-control-cleanup-v1` with
`canonical-runtime-v1` and collection `0060-mandatory-simulation-control-cleanup` with
`canonical-runtime`. The generic setup then owns `out/cooked/canonical-runtime/` and
`out/captures/canonical-runtime/acceptance.json` directly. Update current operational documentation
to that path and to the accepted actor lifecycle/simulation vocabulary.

Add one focused guard that requires exactly one neutral wrapper revision and collection, forbids
the old label in the live wrapper, and requires the documented neutral evidence directory. Keep
historical Experiment 0060/ADR 0063 files and their index entries as decision history.

Do not add a compatibility collection, copy/move utility, fallback output path, schema version,
command alias, wrapper alias, runtime change, workload reorder, or GPU behavior change.

## Workload

1. Audit every live reference to the old revision and collection separately from historical docs.
2. Replace the two wrapper constants and current evidence path directly.
3. Prove the canonical wrapper still exposes only `runseal :canonical-runtime` and its strict help/
   argument surface.
4. Prove the wrapper type-checks with the neutral collection flowing through setup, bootstrap,
   capture, temporal, failure, traversal, plateau, and report calls.
5. Run the stable identity guard, `runseal :init`, and `runseal :guard`.
6. Do not run the full canonical workflow: this cleanup changes only labels and output directories,
   not workload commands, ordering, runtime, renderer, GPU resources, synchronization, or lifecycle.

## Controlled Variables

- Every imported support function, gate invocation, order, target, count, and acceptance field other
  than `revision` remains unchanged.
- The reference target remains `(2^40,-2^40)` and the report remains one ignored JSON file.
- `prepareCanonicalSetup` remains generic; no Experiment 0065 special case is added.
- Historical docs retain their original experiment paths and conclusions.

## Metrics

- Exact live wrapper revision/collection count and value.
- Old-label count in the live wrapper.
- Exact documented current evidence directory.
- Wrapper help/invalid-argument result, Deno check, init result, guard result, and Flavor denies.
- Runtime/source/GPU/frame/process work performed by the cleanup checks.

## Acceptance Criteria

- The live canonical operator has exactly one revision `canonical-runtime-v1` and one collection
  `canonical-runtime`; the old Experiment 0060 label is absent from it.
- Current operational documentation names `out/captures/canonical-runtime/`; historical records
  remain intact and are not treated as compatibility surfaces.
- No alias, fallback, migration, duplicate wrapper, or alternate report path exists.
- Strict help/argument behavior and TypeScript checking pass.
- `runseal :init` and `runseal :guard` pass with zero runtime/source/GPU/frame/process work required
  by this cleanup and no full canonical run.

## Reference Environment

The experiment uses the pinned Windows reference platform, the direct non-recursive Runseal
operator, its generic canonical setup/capture support, current repository guard, and ignored `out/`
ownership.

## Evidence

The reference audit found exactly two live historical constants in the direct wrapper and one
current evidence path in `AGENTS.md`. Generic setup/capture support was already collection-neutral;
historical Experiment 0060/ADR 0063 documents and their index entries were left unchanged.

A 2.1-second focused operator gate required the exact single revision `canonical-runtime-v1`, exact
single collection `canonical-runtime`, zero old-label occurrences in the live wrapper, and the
documented neutral capture directory. The wrapper and guard type-checked, `--help` returned the one
canonical usage line, and an extra argument failed before setup with `unexpected argument`.

`runseal :init` passed in 0.34 seconds and `runseal :guard` passed in 14.33 seconds with zero Flavor
denies. The cleanup checks launched no workbench/prototype process, cooked no source, and performed
no frame, GPU, readback, synchronization, or lifecycle work. The full canonical workflow was not
run because command order and every runtime/GPU gate are byte-unchanged; only report identity and
ignored output directories changed.
