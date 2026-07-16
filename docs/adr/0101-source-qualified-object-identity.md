# ADR 0101: Source-Qualified Object Identity

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0098 Source-Qualified Object Identity

## Context

Owner region plus authored local ID is unique only inside one object source. After source
replacement the same pair can address different content, so retaining a nearest result without its
source lifetime would permit silent aliasing.

The published CPU snapshot already owns the exact `.wlr` source namespace and commits it atomically
with its pages. Composition tokens are too short-lived because ordinary traversal changes them;
stable-seed namespace is too broad because it intentionally remains equal across source variants.
A gameplay-persistent ID would require authoring, persistence, and server contracts that do not yet
exist.

## Decision

- Define `CanonicalObjectIdentity` as `(ObjectSourceNamespace, owner RegionCoord, authored local ID)`.
- Make `CanonicalObject` carry that nested identity as its sole address representation.
- Replace `Runtime::query_canonical_object(region, local_id)` with strict
  `Runtime::query_canonical_object(identity)`. Retain no overload, optional namespace, alias, or
  fallback.
- Compare the requested namespace with the current published snapshot before region/page lookup.
  Any mismatch fails without returning an object from the replacement source.
- Copy the same identity into every nearest candidate. Keep nearest ordering based on distance,
  owner region, and authored local ID because source is constant within one committed scan.
- Expose the existing zero-allocation namespace value and serialize it as exact lowercase SHA-256
  hex. Require that value in the workbench payload and reject the old schema.
- Treat source qualification as snapshot/source identity only. Do not claim stability across recooks,
  physical reorder, content revision, or server/gameplay persistence.

## Consequences

- A future retained target can detect source replacement instead of silently resolving to another
  object at the same local address.
- Physical A/B packs intentionally produce different qualified identities even when independent
  oracles prove their raw/spatial object content equal.
- Traversal within the same source preserves identity; revisit restores the earlier namespace;
  failed pair publication retains the prior identity atomically.
- Existing exact/nearest JSON response shapes change without a compatibility schema. Current
  repository consumers move together.
- This decision adds no retained target, source registry, allocation, I/O, GPU work, interaction, or
  gameplay-persistent identity.

## Evidence

Experiment 0098 passes all 95 engine-runtime tests. `canonical-frame-v5` passes in 13.731 seconds
with exact independent namespace/identity equality, strict mismatch/old-payload failures, zero query
work, and unchanged GPU hashes. `canonical-prototype-v18` passes in 75.899 seconds with the native
F+W source-qualified observation unchanged.

The final-worktree `canonical-runtime-v6` passes in 238.700 seconds. A/B namespaces differ, both stale identities fail
after replacement, A revisit restores its exact identity, and adjacent movement, both failed pairs,
restart, 32+32 traversal samples, the bounded resource checkpoint, and two lifecycle cycles pass.
Resources finish at 492 handles and 21 threads with private bytes 299,008 above baseline under the
unchanged allowance.
Repository guard passes with zero Flavor deny issues.
