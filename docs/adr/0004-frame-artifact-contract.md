# ADR 0004: Frame Artifact Contract

- Status: Superseded
- Date: 2026-07-12
- Supersedes: None
- Superseded by: [ADR 0005](0005-capture-collection-contract.md)

## Context

ADR 0003 established a visible native workbench but its first screenshot evidence came
from desktop capture. Region and scene work require an observation artifact owned by the
renderer so pixels can be correlated with exact runtime state without window placement,
desktop composition, or operator tooling affecting the image.

Experiment 0002 proved deterministic D3D12 back-buffer capture across repeated frames
and Sidecar process restart.

## Decision

- `workbench.capture` is the typed project event for requesting one renderer-owned frame
  artifact. Its payload accepts a constrained capture ID, not an arbitrary filesystem
  path.
- A capture transitions the rendered back buffer through `COPY_SOURCE`, copies into a
  persistent readback resource, waits for only that submitted capture frame, removes row
  pitch, and restores `PRESENT` before presentation.
- Frames without a capture request do not perform capture transitions, readback waits,
  hashing, encoding, or filesystem work.
- Every capture produces a lossless RGBA PNG and JSON frame manifest under the ignored
  `out/experiments/0002-deterministic-visual-loop/` tree.
- The manifest correlates raw pixel and PNG SHA-256 values with process identity, git
  revision, frame index, controlled state, renderer facts, artifact paths, and capture
  timing.
- Generated captures remain disposable evidence. Accepted conclusions and representative
  hashes belong in versioned experiment documentation.

## Consequences

Humans and agents can now discuss the exact renderer output rather than a desktop
screenshot. Later camera, object ID, depth, normal, and semantic-buffer artifacts can
extend the manifest without changing Sidecar's transport envelope.

Capture is intentionally synchronous and CPU-encoded because it is an explicit operator
operation, not a frame hot path. Continuous capture, asynchronous encoding, video, and
shipping-runtime screenshot policy remain out of scope.

## Evidence

- [Experiment 0002](../../experiments/0002-deterministic-visual-loop/README.md) records
  matching raw pixel and PNG hashes across repeated frames and Sidecar restart, distinct
  hashes after a controlled color mutation, and zero residual runtime processes.
