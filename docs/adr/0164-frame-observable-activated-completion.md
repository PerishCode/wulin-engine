# ADR 0164: Frame-Observable Activated Completion

- Status: Accepted
- Date: 2026-07-19
- Experiment: 0161 Frame-Observable Activated Completion

## Context

The maintained post-readiness Activated recovery helper posted Escape after a fixed 250 ms delay.
On a regenerated cold workspace, three complete runs reached only three of the required 12
projected acknowledgement frames before exit, despite zero render blocks. The product contract was
correct; acceptance terminated before the frame-semantic boundary it intended to validate.

Adding product telemetry or another GPU readback path would duplicate existing completion
authority. Extending the delay would retain the same machine-dependent proxy.

## Decision

- Replace the recovery helper's fixed success dwell with an acceptance-owned visible-frame
  observer prepared before action input.
- Bind it to the exact visible Prototype PID/window and capture the client pixels while temporarily
  topmost without activation.
- Detect the fixed Activated-green color class rising by at least 64 pixels over the captured
  baseline, followed by two samples no more than 16 pixels above baseline.
- Restore z-order and post Escape only after that semantic completion.
- Treat 10 seconds strictly as a bounded failure deadline.
- Retain the final product-owned exact 12 Activated frames plus following suppression as the
  authoritative behavior gate.
- Preserve two-value stdout and add no Runtime call, product field, renderer/GPU resource,
  synchronization, process, retry, or compatibility path.

## Consequences

- Cold and warm hosts terminate the Activated case on the same rendered meaning rather than the
  same elapsed duration.
- Failure diagnostics now distinguish absent green projection, incomplete clear, native helper
  failure, and product final-state divergence.
- Desktop capture requires the exact window to remain visibly capturable; inability to do so is a
  bounded acceptance failure, not a product fallback.
- The other 17 graceful sessions and all product/runtime ownership remain unchanged.

## Evidence

Focused evidence observed baseline/peak/completion green counts `1/66/1` and completed in
1,115.6051 ms. A direct observer-then-Jump diagnostic retained exact `+1/+1/+1`
suspend/resume/reset deltas, excluding observer process and z-order leakage.

`canonical-prototype-v76` passed in 150.059 seconds. PID 21060 observed counts `1/1237/1`, six
Activated samples, two clear samples, and 37 total samples in 1,346.6549 ms. Product completion
reported exact 12 Activated frames, 14 suppression frames, one commit, cleared acknowledgement and
target, zero render blocks, exit zero, and exactly two output values.

The 375,635-byte report retained 18 graceful launches and zero nontrivial raw/invariant copies. All
103 engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor retained zero denies
and five existing warnings. No product Rust, Runtime, renderer/GPU, source, resource,
synchronization, or process-count file changed.
