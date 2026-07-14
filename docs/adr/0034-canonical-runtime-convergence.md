# ADR 0034: Canonical Runtime Convergence

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

Experiments 0001 through 0030 proved the engine architecture incrementally. Their live
implementation retained local formats, generated sources, standalone renderer modes,
operator selectors, and recursive acceptance wrappers after the canonical signed
runtime had made those discovery paths redundant. Multiple paths could still describe
or own the same content and GPU resources.

Experiment 0031 tests whether the accepted signed terrain, schema-2 object identity,
bounded residency, terrain-first composition, surface/occlusion execution, traversal,
prefetch, and rollover contracts remain intact after deleting those alternatives.

## Decision

- The only live terrain format is the signed `i64` `.wlt` pack.
- The only live object format is the signed schema-2 `.wlr` pack with 1,024 records and
  1,024 explicit authored local IDs per region.
- Terrain and object cache identity always includes the source namespace and signed
  global region. Each domain retains a fixed 50-slot GPU cache.
- A frame is either the source-free idle shell or fixed terrain-first canonical
  composition. Composition always includes arbitrary-Q8 grounding, GPU terrain LOD,
  skeletal execution, surface resolve, conservative occlusion, shared reverse-Z depth,
  and the shared integer semantic attachment.
- Terrain and objects publish only as one matched canonical pair. Camera traversal,
  prefetch, and safe-band rollover consume that pair contract.
- The live inspect vocabulary is the compact `source.*` / `canonical.*` surface plus
  lifecycle, status, camera, capture, perception, traversal, prefetch, and four fault
  gates. It cannot select a historical format, source kind, fixture, pass order, local
  schedule, or standalone renderer mode.
- `runseal :canonical-runtime` is the sole end-to-end engine acceptance workflow. It is
  direct and must not invoke historical experiment wrappers.
- Historical experiment READMEs and ADRs remain evidence. Their runtime paths are not
  compatibility commitments, and no migration adapter is required for deleted formats.

## Consequences

- New engine capabilities must consume the canonical runtime instead of adding a
  parallel content path or compensating fallback.
- Source switches, failures, holds, movement, and rollover retain a complete old pair
  until one complete new pair can publish.
- Surface resolve preserves terrain color, depth, and semantic values where no object
  wins; a later composition stage must not clear an earlier shared attachment.
- Operator vocabulary, resource ownership, startup state, and lifecycle cleanup are
  materially smaller. Reintroducing a removed path requires a new experiment and ADR.

## Evidence

Experiment 0031 passed the direct 403-second workflow, including reordered-source and
attachment invariance, all movement/failure/hold gates, 32 reactive and 32 prepared
crossings, a warmed 64-publication same-process resource plateau, and 16 complete
lifecycle cycles. The ignored full report is generated at
`out/captures/0031-canonical-runtime-convergence/acceptance.json`.
