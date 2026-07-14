# ADR 0045: Canonical Bootstrap Readiness

- Status: Accepted
- Date: 2026-07-14
- Supersedes: None
- Superseded by: None

## Context

The accepted workbench becomes ready after one idle-shell frame, then relies on three independent
operator mutations to open terrain, open objects, and schedule a canonical pair. A thin prototype
host built on this behavior would have to invent its own configuration, operation order, async
failure policy, and meaning of ready while also establishing application ownership.

Experiment 0042 tests the narrower prerequisite in the existing lifecycle owner: a strict
declarative source/config selection either reaches a rendered canonical frame before readiness or
terminates without fallback.

## Decision

- The workbench accepts at most one `--bootstrap=<repository-relative-json>` argument in addition
  to its Sidecar stamp. Unknown and duplicate arguments are errors.
- Bootstrap schema 1 selects exactly one terrain pack, one schema-3 object pack, and one signed
  global origin/center/active-radius configuration. It contains no presentation, camera, input,
  simulation, actor, UI, or discovery policy.
- Configuration and pack paths use normal repository-relative components. Bootstrap JSON is size
  bounded; pack paths remain under `out/cooked` with their canonical extensions.
- Configured startup opens both sources, schedules one manual canonical pair, and advances the
  existing runtime frame transaction while the window remains hidden. The ordinary readiness line
  is emitted and the window shown only after a canonical frame completes.
- Configuration, source, scheduling, asynchronous pair, timeout, or close failure is terminal and
  emits no readiness. Configured startup never falls back to an idle shell.
- Unconfigured startup retains the accepted idle-shell-first readiness behavior.

## Consequences

- A later thin host can consume one proven bootstrap/readiness contract rather than copying a
  three-command diagnostic workflow.
- Bootstrap uses host wall time only as a bounded readiness timeout; no duration enters runtime
  presentation or future simulation state.
- The first schema is intentionally narrow and repository-local. Distribution manifests, content
  discovery, multiple profiles, save selection, and portability require later evidence.

## Evidence

Experiment 0042 passed the 559.8-second direct workflow. Invalid, missing-source, and asynchronous
corrupt-payload configurations all exited nonzero without readiness. Valid initial and restarted
processes reached canonical-ready on hidden frame 8 in 102.8 and 100.1 ms, reproduced the same
configuration hash and tick-zero frame, and left no Sidecar-owned process. Ordinary startup,
deterministic input, all prior GPU hashes, 32+32 traversal, the accepted resource plateau, 16
lifecycle cycles, and the repository guard passed.

Generated evidence is ignored under
`out/captures/0042-declarative-runtime-bootstrap/`.
