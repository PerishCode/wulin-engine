# Experiment 0032: Authored Object Presentation

Status: Accepted

## Hypothesis

The canonical runtime can consume archetype, material, orientation, and animation state
as explicit cooked object authority while preserving stable identity, bounded data
movement, atomic publication, deterministic rendering, and fixed GPU submission.

No live shader or CPU oracle should need to derive these presentation properties from an
object stable key. Physically reordered packs that preserve local-ID-to-presentation pairs
must remain output-identical, while controlled presentation mutations must cause exactly
the expected GPU-visible changes.

## Scope

This experiment replaces signed object payload schema 2 with schema 3. Each fixed region
contains three equally indexed planes:

1. 1,024 unchanged 20-byte spatial records;
2. 1,024 unchanged 4-byte authored local IDs;
3. 1,024 fixed 16-byte presentation records containing archetype, material, Q16 yaw, and
   packed animation clip/phase-offset/variant state.

All three planes share one region checksum, source namespace, background read, cache-slot
reservation, copy-fence completion, publication, and rollback outcome. The existing eight
synthetic mesh archetypes, eight animation clips, and 64 generated surface materials remain
controlled catalog inputs.

General asset formats, asset import, variable-size records, sparse occupancy, texture or
mesh streaming, gameplay state, ECS ownership, collision, navigation, networking, and
Wulin content are out of scope.

## Workload

1. Cook two schema-3 sources whose physical record order differs while all
   local-ID-to-spatial-to-presentation triples remain identical.
2. Cook controlled variants that change authored archetype, material, yaw, and animation
   independently without changing stable identity or spatial data.
3. Publish a 25-region window and read back all 25,600 active record, identity, and
   presentation entries. Join them to pack-index checksums and exact CPU expectations.
4. Compare reordered sources across GPU counters, pose samples, surface samples, color,
   object-ID, diagnostic, and capture hashes. Then require each controlled mutation to
   change its intended evidence without changing identity or grounding.
5. Exercise adjacent, diagonal, revisit, compensated alias, object I/O hold, object copy
   hold, corruption rollback, restart, prefetch, and rollover paths with triple-plane
   publication.
6. Run the warmed 64-publication resource plateau and 16 lifecycle cycles through the
   direct canonical workflow.

## Controlled Variables

- Signed `i64` identity, source-addressed 50-slot caches, 25 active regions, terrain
  projection, reverse-Z, grounding, terrain LOD, camera, visibility, and surface resolve
  remain fixed.
- `InstanceRecord`, authored local IDs, stable-key calculation, and semantic object IDs
  remain identity/spatial authority only.
- Mesh, animation, and surface catalogs are unchanged. Presentation records may only
  select valid entries and parameter values from those catalogs.
- Animation phase is the cooked phase offset plus the existing frame time tick; cooked
  content selects the clip and variation but does not replace runtime time progression.
- No compatibility reader or fallback presentation synthesis is permitted.

## Metrics

- Exact file, region-payload, per-plane, source-namespace, and active-page checksums.
- Object I/O bytes and per-plane GPU copy bytes/counts for cold, adjacent, diagonal,
  revisit, and alias publication.
- GPU-read triple permutation validity and exact local-ID-to-presentation joins.
- Visible/static/animated counts, archetype mask, active/shared poses, clip/phase/variant,
  material samples, color/object-ID/diagnostic hashes, grounding, and contact residuals.
- Copy/publication fence behavior, rollback state, D3D12 validation/device status, fixed
  dispatch/draw counts, handles, and private bytes.

## Acceptance Criteria

- Schema 3 deterministically round-trips all three planes and rejects invalid catalog
  indices, yaw values, animation encodings, duplicate/out-of-range local IDs, corrupt
  presentation bytes, noncanonical ranges, and malformed sizes.
- Reordering physical triples changes source/file identity but not stable-key-indexed
  presentation, GPU behavior, attachments, or capture evidence.
- Controlled mutations preserve stable identity, object IDs, positions, grounding, and
  contact while producing the exact expected archetype, material, yaw, or animation
  evidence change.
- A cold 25-region publication copies exactly 25 pages from each object plane. Adjacent,
  diagonal, revisit, and compensated alias movement retain the bounded cache and expected
  page-copy counts without mixed triples.
- Holds and corruption never publish a partial triple. Failure retains the prior complete
  frame, restart recovers, and prefetch/rollover never exposes stale presentation data.
- Skeletal and surface submission counts remain fixed and D3D12 validation reports no
  error. After the warmed resource sample, the final handle count must not exceed baseline,
  no intermediate sample may exceed it by more than one transient OS/driver handle, private
  bytes retain the 16 MiB bound, and all lifecycle cycles leave no descendant.
- Live shader and oracle code contains no stable-key derivation of archetype, material,
  yaw, animation enablement, clip, phase offset, or animation variant.

## Evidence

The intended direct workflow is:

```powershell
runseal :canonical-runtime
```

Generated evidence remains ignored under
`out/captures/0032-authored-object-presentation/`.

## Results

The direct workflow passed in 426.1 seconds. Each base object source contained 440
schema-3 regions, 18,022,400 payload bytes, and 18,051,072 total file bytes. The two
physical orders had different file/source identities but identical identity-keyed
presentation, skeletal, surface, grounding, contact, color, object-ID, diagnostic, and
PNG evidence.

All four controlled presentation sources preserved spatial records, local identities,
stable keys, semantic IDs, grounding, contact, and terrain evidence. Their GPU-read
presentation hashes changed. Archetype, material, and yaw mutations changed rendered
color evidence while preserving skeletal aggregate counts; animation changed the exact
animated/static and pose workload. GPU and CPU oracles remained equal for every source.

A cold publication copied 25 spatial, 25 identity, and 25 presentation pages. The
adjacent and fresh diagonal transitions copied exactly 5 and 9 pages from each plane.
Revisit, compensated alias, four I/O/copy holds, presentation-byte corruption rollback,
terrain corruption rollback, restart, prepared rollover, 32 reactive crossings, and 32
prepared crossings retained complete triples and matched terrain/object publication.

The warmed resource baseline was 531 handles and 398,200,832 private bytes. One sample at
publication 40 observed a transient 532nd handle; later samples returned below baseline.
The final sample after 64 publications was 516 handles and 397,205,504 private bytes. All
16 complete lifecycle cycles left no workbench, broker, inspect, cargo, or wrapper
descendant.

## Conclusion

Accepted. Object presentation is now explicit cooked authority. Spatial, identity, and
presentation planes have one source checksum and one transactional GPU publication;
stable keys remain identity only and no live shader or oracle derives presentation from
them.

## Promotion

Promoted schema-3 object storage, presentation authoring/validation, triple-plane bounded
residency/readback, and explicit skeletal/surface consumption. ADR 0035 records the
durable boundary. General asset import remains a later gate.
