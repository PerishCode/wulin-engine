# ADR 0005: Capture Collection Contract

- Status: Accepted
- Date: 2026-07-12
- Supersedes: [ADR 0004](0004-frame-artifact-contract.md)
- Superseded by: None

## Context

ADR 0004 correctly established renderer-owned captures but assigned every artifact to
Experiment 0002's output directory. Experiment 0003 needs the same capture mechanism
without claiming that its scene evidence belongs to an earlier experiment.

## Decision

- `workbench.capture` accepts a constrained `id` and constrained `collection`; neither
  field is an arbitrary path.
- Both values contain 1 through 64 ASCII letters, digits, hyphens, or underscores.
- Captures are written under `out/captures/<collection>/<id>.{png,json}`.
- A missing collection maps to `operator` for protocol compatibility. Repository
  wrappers always state their collection explicitly.
- The frame manifest records its collection and the returned artifact paths remain the
  authority for locating generated evidence.
- All renderer transitions, synchronization, hashing, encoding, and disposable-output
  rules accepted by ADR 0004 remain unchanged.

## Consequences

Experiments and operators share one capture implementation while retaining explicit
artifact ownership. Path traversal is excluded by construction. Moving generated output
does not alter the renderer hot path or make captures source-of-truth files.

## Evidence

- Experiments 0002 and 0003 both pass through distinct collections using the same typed
  capture event.
