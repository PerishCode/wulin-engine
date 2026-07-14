# Experiment 0031: Canonical Runtime Convergence

Status: Accepted

## Hypothesis

The accepted signed terrain, schema-2 object authority, bounded GPU residency,
skeletal/surface execution, and atomic traversal path can operate as one runtime without
the local, generated, schema-1, standalone-render-mode, or recursive-experiment paths
that were retained while the architecture was being discovered.

Removing those paths should preserve canonical evidence while materially reducing code,
operator vocabulary, startup branching, lifecycle churn, and the number of states that
can own the same GPU resources.

## Scope

The live baseline becomes:

- signed terrain packs only;
- signed schema-2 object packs with explicit local IDs only;
- one source-addressed terrain cache and one source-addressed object cache;
- one atomic canonical composition/traversal renderer after an idle workbench shell;
- one compact Sidecar control plane for sources, global scheduling, traversal, camera,
  probes/captures, fault gates, status, and lifecycle;
- one non-recursive canonical acceptance workflow.

Historical experiment READMEs, ADRs, and settled results remain as decision history.
Their old wrappers, runtime modes, formats, fallback behavior, and status projections do
not remain executable merely to reproduce that history.

## Removal Set

1. Remove local object and terrain pack codecs/cook modes, signed object payload schema
   1, synthesized ordinal identity, and explicit-to-ordinal identity restoration.
2. Remove local/global cache keys that do not include canonical source identity, local
   terrain aliases as content identity, generated object submission, and
   `objects.disable` fallback.
3. Make signed global config and both source namespaces required throughout composition;
   remove local pair scheduling, optional source projections, selectable fixture/order,
   and standalone composition enable/disable.
4. Remove standalone load, synchronous resident, local cooked, procedural async,
   meshlet-only, skeletal-only, surface-only, and terrain-only frame modes. Retain their
   proven reusable GPU subsystems only where canonical composition consumes them.
5. Replace the accumulated inspect vocabulary and workbench wrapper with one canonical
   control surface. Remove old experiment wrappers/support from guard and operations.
6. Replace recursive compatibility execution with direct codec, state, visual, failure,
   movement, bounded-load, and lifecycle checks against the converged runtime.

## Baseline Inventory

Before convergence the repository contains 134 Rust files / 30,655 lines, 9 HLSL files
/ 2,520 lines, and 58 Runseal TypeScript files / 16,583 lines. Runseal exposes 33
wrappers and 25 support files. The workbench protocol contains 79 control variants and
75 named events.

These counts are descriptive evidence, not optimization targets. The pass condition is
removal of redundant ownership and selectors, not a particular percentage of deleted
lines.

## Workload

1. Record all live references to old formats, cache keys, generated sources, renderer
   modes, controls, wrappers, and compatibility recursion before deletion.
2. Delete in format, runtime, and operator-workflow batches. Compile and run focused
   tests after every batch; no compatibility adapter may be introduced to make an old
   caller compile.
3. Cook two physically reordered schema-2 object sources and one signed terrain source.
   Start the idle workbench, open canonical sources, publish one far signed window, and
   require composition to become the sole rendered mode.
4. Re-run exact order A/B, GPU-read record/identity authority, grounding, contact,
   skeletal, surface, semantic, perception, and full attachment comparisons.
5. Exercise adjacent, diagonal, revisit, compensated alias, prefetch, rollover, object
   I/O hold, object copy hold, terrain I/O hold, terrain copy hold, corruption rollback,
   restart, and 32 reactive plus 32 prepared crossings without invoking an older
   experiment workflow.
6. In one process, run 32 representative warm publications, then require the handle
   count to remain unchanged for 60 continuous seconds. From that warmed sample, run 64
   additional canonical publications/probes and sample after every eight. Handle count
   must not increase; final private bytes must remain within 16 MiB of the sample.
7. Run 16 complete Sidecar start/ready/canonical-frame/stop cycles. After every stop,
   require zero workbench, broker, inspect, cargo, or wrapper descendants for that
   namespace.
8. Run the full repository guard and scan live source/config files for every forbidden
   compatibility symbol and command.

## Controlled Variables

- Exact signed `i64` identity, centered GPU projection, reverse-Z, active radius 2, 25
  active regions, 50 cache slots, fixed indirect submission, terrain LOD, skeletal and
  surface catalogs, GPU-read payload authority, and atomic publication remain accepted.
- Object schema 2 remains fixed at 1,024 records plus 1,024 unique local IDs per region.
  Sparse occupancy and variable record counts remain out of scope.
- The idle workbench may render calibration before sources are supplied. It is not a
  second content mode and owns no world residency.
- No source migration utility is required. This is an experimental repository with no
  commercial or broad compatibility commitment.
- New gameplay, ECS, assets, collision, navigation, networking, legacy import, and mod
  content are out of scope for this convergence phase.

## Pass Criteria

- Live application/crate/tool code contains no local object/terrain pack reader or
  writer, object payload schema 1, synthesized ordinal identity, generated object
  source, local/global non-source cache key, or fallback-to-generated transition.
- Renderer frame dispatch has only idle-shell and canonical-composition outcomes. No
  operator command can select a historical renderer mode, composition order, fixture,
  or local schedule.
- Runseal retains at most `init`, `guard`, `gpu-lab`, `workbench`, and the new canonical
  workflow. The canonical workflow invokes no older experiment wrapper.
- All canonical content, movement, failure, oracle, attachment, and fixed-capacity
  evidence passes with no mixed source, early publication, stale descriptor, validation
  error, unbounded allocation, or device removal.
- The 64-publication resource plateau and all 16 lifecycle cycles pass exactly as
  specified. Process restart is not accepted as a substitute for the in-process plateau.
- `AGENTS.md`, source indexes, commands, Flavor boundaries, and operational docs describe
  only files and workflows that still exist.

## Evidence

The accepted canonical workflow is:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0031-canonical-runtime-convergence/`.

The complete report is written to
`out/captures/0031-canonical-runtime-convergence/acceptance.json`.

## Results

The complete direct workflow passed in 403 seconds without invoking an older
experiment wrapper. Physically reordered schema-2 object packs had distinct source and
file identities while producing identical identity-keyed object behavior, grounding,
contact, color, raw object-ID, diagnostic, and PNG evidence. Adjacent, diagonal,
revisit, compensated alias, all four I/O/copy holds, both corruption rollbacks,
restart, prepared rollover, and 32 reactive plus 32 prepared crossings all retained a
complete published pair with no mixed authority or device failure.

The warmed same-process sample contained 531 handles and 391,708,672 private bytes.
Across 64 additional publications, no sample exceeded 531 handles; the final sample
contained 516 handles and 391,204,864 private bytes. All 16 complete
start/ready/canonical-frame/stop cycles left no workbench, broker, inspect, cargo, or
wrapper descendant in the Sidecar namespace.

The live inventory fell from 134 Rust files / 30,655 lines to 118 / 21,255, from nine
HLSL files / 2,520 lines to six / 1,715, and from 58 Runseal TypeScript files / 16,583
lines to six / 1,383. Runseal now contains exactly five wrappers and one support file.

## Conclusion

Accepted. Signed terrain and schema-2 authored objects now have one source-addressed,
bounded, atomically published runtime. Idle shell and terrain-first canonical
composition are the only frame outcomes. Historical formats, generated sources,
standalone content modes, control selectors, recursive wrappers, and fallback
transitions remain decision history rather than executable compatibility surfaces.
