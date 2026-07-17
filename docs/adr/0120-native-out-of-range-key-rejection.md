# ADR 0120: Native Out-of-Range Key Rejection

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0117 Native Out-of-Range Key Rejection

## Context

The Win32 adapter captures the complete `WPARAM` key value as `usize`, and `HostInput` uses checked
`u8` conversion before normalization. Unit tests reject 256, but no real Prototype process proved
that a high value whose low byte aliases a live control cannot be truncated before camera policy.

## Decision

- Maintain one real-process session that posts `0x145` plus W after orbit-zero readiness and exits
  after one bounded delay.
- Use exact camera-relative locomotion as the rejection oracle: accepted checked rejection is
  negative Z at orbit 0; truncation to E (`0x45`) would be negative X at orbit 1.
- Keep checked range ownership in the sole existing `HostInput`. Add no product input value,
  history, event stream, invalid-key report, compatibility decoder, or controller state.

## Consequences

- Native out-of-range rejection gains exact adapter-to-product evidence that distinguishes checked
  rejection from low-byte coercion.
- The acceptance harness gains one labeled invalid key and one bounded session. Repeated session
  baseline comparisons share one local helper to remain below the enforced source limit.
- This decision does not authorize arbitrary virtual-key support, remapping, raw-input handling,
  key replay, another input owner, or Runtime/GPU/resource changes.

## Evidence

Experiment 0117 passed `canonical-prototype-v33` in 114.141 seconds. PID 16624 received key 325
and W 2.2231 ms apart, then Escape after 222.059 ms. Completion retained orbit 0 and produced
exactly 13 negative-Z Walk steps: delta `(0, -416)` Q9, clip 1, and yaw 49,152. Actor
identity/shape, zero vertical velocity, clock discontinuity/stall counts, idle object state,
render-block count, and the two-value clean completion remained exact.
