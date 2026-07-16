# ADR 0104: Version-Gated Prototype Object Target

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0101 Version-Gated Prototype Object Target

## Context

ADR 0100 admits one explicit post-commit nearby-object observation but retains its result only in
same-completion evidence. ADR 0101 provides source-qualified snapshot identity, and ADR 0102
distinguishes resolution, source replacement, and same-source window departure. A product target can
now retain intent safely, but blindly resolving every frame would add structurally avoidable work,
while resolving only on another F press would leave state stale after composition publication.

The composition coordinator already owns one monotonic token and object source namespace for every
successfully published atomic pair. Product policy currently has no typed access to that value and
must not parse diagnostic JSON or move its target into engine ownership.

## Decision

- Expose one read-only `CanonicalObjectSnapshot` containing the current published pair token and
  object source namespace. Tokens are scoped to one live Runtime; namespace qualification prevents
  cross-source comparison from becoming meaningful accidentally.
- Keep target policy in the prototype. Retain at most one target containing only qualified identity,
  last validated snapshot, and resolved/outside-window availability.
- On explicit F completion, read the snapshot and then run the existing nearest query from the exact
  committed actor output. These consecutive `&self` calls cannot race Runtime mutation.
- A candidate atomically replaces the target; an empty successful query clears it. Query or policy
  validation failure consumes neither pending intent nor the prior target.
- After every successful frame, do nothing when no target exists. When a target exists, compare its
  snapshot with the current typed stamp. Equal stamps eliminate resolver work; a changed stamp
  triggers exactly one call to the accepted typed resolver.
- `Resolved` updates the stamp and availability. `OutsidePublishedWindow` retains identity and the
  new stamp as unavailable so a later same-source publication can restore it. `SourceReplaced`
  clears target intent rather than allowing a later old-source revisit to reacquire it silently.
- Validate complete resolution/source/identity invariants before mutating policy. Keep hard Runtime
  errors terminal rather than coercing them into ordinary target loss.

## Consequences

- Prototype target state stays current at successful frame boundaries without per-frame nearest
  scans or per-frame exact page lookup on unchanged snapshots.
- Window residency and source lifetime have deliberately different product meaning: temporary
  same-source unavailability retains intent; world/source replacement removes it.
- The target does not claim gameplay persistence across Runtime restart, source recook, or server
  state and carries no copied position, presentation, or interaction eligibility.
- Workbench object responses expose the same typed stamp so focused/full acceptance can compare it
  directly with the authoritative published pair and failed-publication rollback.
- No highlight, interaction/action effect, line of sight, navigation, networking, asset/format
  change, or Wulin semantic follows from this decision.

## Evidence

Experiment 0101 passes all 90 engine-runtime tests, all prototype tests, strict Clippy, and selected
Deno checks. `canonical-frame-v7` passes in 27.122 seconds with exact stamp/published-pair equality,
zero query-side work, and unchanged GPU replay hashes. `canonical-prototype-v19` passes in 71.515
seconds; native F+W proves token-1 acquisition followed by one token-2 traversal-publication
revalidation while retaining only identity/stamp/availability.

The final-worktree `canonical-runtime-v9` passes in 253.412 seconds. A/B publication changes token
and namespace; adjacent same-source publication changes token while preserving namespace; both
failed pair types retain exact token 21. Existing rollback, restart, traversal, GPU, resource, and
two-cycle lifecycle evidence passes. Resources remain 492 handles/21 threads; private bytes rise
1,187,840 under the unchanged allowance. The report contains 24 files totaling 25,346,346 bytes.
Repository guard passes with zero Flavor deny issues.
The commit hook passes the same final repository gate.
