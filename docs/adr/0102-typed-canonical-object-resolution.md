# ADR 0102: Typed Canonical Object Resolution

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0099 Typed Canonical Object Resolution

## Context

ADR 0101 source-qualifies canonical object addresses and prevents aliasing after source replacement,
but the exact lookup reports source replacement, departure from the active window, invalid caller
input, and malformed committed data through one untyped error channel. A future retained product
target must be able to treat ordinary lifetime loss as state without hiding corruption or parsing
error text.

Resolving copied content every frame, retaining it without validation, or adding a persistent object
registry would each introduce policy or authority before Prototype 0 has proven target retention.
The committed CPU snapshot already contains everything needed for an explicit bounded resolution.

## Decision

- Define tagged `CanonicalObjectResolution` with `Resolved(CanonicalObject)`, `SourceReplaced`, and
  `OutsidePublishedWindow`.
- Replace `Runtime::query_canonical_object` with `Runtime::resolve_canonical_object(identity)` and
  replace the workbench query verb with `canonical.objects.resolve`. Keep no compatibility method,
  verb, overload, alias, or fallback.
- Validate authored local-ID range and committed snapshot shape before returning a typed lifetime
  outcome.
- Return `SourceReplaced` when the complete well-formed identity names a namespace other than the
  committed snapshot. Do not look up replacement-source content at the same local address.
- Return `OutsidePublishedWindow` when the source remains current but the owner region is not active.
- For an active current-source address, validate page shape, uniqueness/presence of the requested
  authored ID, stable seed, and exact identity, then return `Resolved`.
- Keep pre-publication access and invalid or malformed caller/committed state as errors.
- Perform resolution only when explicitly called, over the sole committed immutable CPU snapshot,
  with no allocation, source I/O, GPU work, synchronization, automatic reload, or mutation.

## Consequences

- A later product owner can clear or reconsider a target after ordinary source/window lifetime loss
  without parsing text or confusing that loss with corruption.
- Window traversal and source replacement remain distinct even though neither returns copied object
  content.
- The repository protocol and exact facade change together; the old query names are deliberately
  rejected and guarded.
- The prototype still retains no target and invokes no resolver. Invalidation cadence and product
  policy remain separate experiments.
- No persistent gameplay identity, registry, selection, interaction, resource, format, asset,
  networking, or Wulin contract is implied.

## Evidence

Experiment 0099 passes all 95 engine-runtime tests, all prototype/workbench tests, strict Clippy,
and Deno type checks. `canonical-frame-v6` passes in 18.164 seconds with the exact three outcomes,
strict error/old-verb gates, independent source equality, zero resolver-side work, and unchanged GPU
hashes.

The final-worktree `canonical-runtime-v7` passes in 249.862 seconds. Both A/B stale identities report
`source-replaced`, A revisit resolves, adjacent same-source departure reports
`outside-published-window`, failed publications and restart retain resolved content, 32+32 traversal
samples pass, and two lifecycle cycles stop cleanly. Resources remain 492 handles and 21 threads;
private bytes rise 172,032 under the unchanged allowance. The report contains 24 files totaling
25,346,200 bytes. Repository guard passes with zero Flavor deny issues.
