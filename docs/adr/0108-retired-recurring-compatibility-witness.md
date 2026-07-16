# ADR 0108: Retired Recurring Compatibility Witness

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0105 Retired Recurring Compatibility Witness

## Context

Removed calibration/world, standalone contact, and caller-owned terrain-body inspect verbs were
still requested once each by every full acceptance run. All ten requests reached the same generic
unknown-event fallback and were retained under `compatibilityRemoval.removedVerbs`. Their absence
was already protected by owner-specific static guards.

The same support module also owned the current clear-only idle-shell image and semantic evidence,
which is live behavior rather than compatibility history.

## Decision

- Delete the ten recurring retired-verb IPC requests and their report payload.
- Delete `.runseal/support/compatibility-removal.ts` and the historical
  `compatibilityRemoval` full-report field.
- Retain the current clear-only workbench proof as `.runseal/support/idle-shell.ts` and
  `correctness.idleShell`, containing status, renderer health, exact clear image, and uniformly
  background semantic evidence.
- Keep route/API absence behind the existing calibration, contact, and terrain-transaction static
  guards. Add one guard that rejects the deleted support path, old report/function/field names, and
  loss of the current idle-shell authority.
- Do not add a replacement rejection list, alias, compatibility decoder, or historical process
  gate.

## Consequences

- Full acceptance no longer spends live process work or report space proving ten settled historical
  facts through the generic dispatcher fallback.
- The current idle shell remains directly and exactly tested.
- Historical experiments/ADRs retain the removed route rationale; live acceptance and status use
  only current names.
- Runtime, renderer, Prototype, source formats, assets, GPU work/resources, networking, and Wulin
  behavior are unchanged.

## Evidence

`runseal :guard` passes with the new removal guard and all existing static owner guards. The
final-worktree `canonical-runtime-v13` passes in 263.724 seconds. All ten historical event-count
keys and `compatibilityRemoval` are absent; `idleShell` retains the exact accepted hashes, 921,600
background pixels, zero differing pixels, and zero visible/unknown semantics.

The workflow records 979 Sidecar invocations versus 988 in v12. It structurally removes ten retired
requests; one additional state-driven `canonical.status` poll in this run makes the net difference
nine. Five warm/eight measured resource publications retain 492 handles and 21 threads; private
bytes move from 412,295,168 to 412,385,280 (+90,112). The artifact inventory remains 24 files /
25,346,280 bytes.
