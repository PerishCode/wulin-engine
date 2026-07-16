# ADR 0105: Frame-Bound Object Target Feedback

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0102 Frame-Bound Object Target Feedback

## Context

ADR 0104 retains one source-qualified prototype object target and revalidates it only when the live
published snapshot changes. The next narrow product question is whether that intent can be visible
without first creating interaction effects or a separate presentation authority.

The existing object-ID attachment identifies only a semantic region. The existing streamed stable
key is a hash and cannot alone recover exact source-qualified identity. A marker would require a
second candidate or copied position, while a host status indication would not be world-space
feedback. The accepted object page already proves a unique authored-ID permutation, and the sole
surface resolve already receives every visible record.

## Decision

- Add an optional `CanonicalObjectIdentity` to immutable `FrameRequest`. Runtime and renderer do not
  retain it across frames.
- Validate authored local ID before render work. After any pending composition publication commits,
  compare source namespace and current global window. Mismatch/absence produces no feedback; a
  match projects owner region to current active index and semantic region.
- Store the streamed authored local ID in the existing visible record's high identity word. Actor
  records continue to carry the full generation across both words.
- Reuse three unused surface constants for enabled, semantic region, and authored local ID. Match
  only non-actor visible records on semantic region plus authored ID; source qualification has
  already been checked against the immutable frame snapshot.
- Apply one static amber mix in the existing surface resolve. Add no marker, outline, pulse, pass,
  resource, descriptor, allocation, copy, wait, fence, or synchronization.
- Reuse one unused word in the existing surface statistics buffer for target pixel count. Probe
  validation independently maps visibility candidates through the committed local-ID pages and
  requires exact equality; the CPU shade oracle reproduces the color transform.
- The prototype forwards its retained identity each frame. Its snapshot/availability policy remains
  the sole product target owner. Workbench set/clear controls own diagnostic frame input only.

## Consequences

- Source replacement and window departure disable feedback in the first frame rendered from the new
  snapshot. Same-source return can restore it on that publication frame, before post-frame product
  policy finishes its typed validation.
- The GPU-visible record remains 56 bytes, surface constants remain 60 DWORDs, descriptors remain
  98, statistics remain 32 bytes, and fixed dispatch counts do not change.
- Authored local ID is now explicit downstream identity metadata for streamed records instead of an
  always-zero high word. It is still snapshot-scoped and not a persistent gameplay/network ID.
- This decision proves only presentation feedback. It does not authorize selection eligibility,
  interaction/action effects, visibility policy, navigation, networking, or Wulin semantics.

## Evidence

Experiment 0102 passes 94 engine-runtime tests and all prototype/reference-host tests.
`canonical-frame-v8` passes in 24.280 seconds with 3,472 exact target pixels, deterministic target
replay, unchanged semantic attachment, and exact clear-to-baseline replay. `canonical-prototype-v20`
passes in 73.149 seconds and submits the native F+W identity in one product frame without copied
object state.

The final-worktree `canonical-runtime-v10` passes in 282.045 seconds. Source replacement and
same-source departure disable projection; source revisit and return recover the same 3,472 pixels.
Rollback, restart, 32+32 traversal, and two lifecycle cycles pass. Active baseline/final resources
remain 527 handles and 24 threads; private bytes increase 589,824 within the accepted plateau. The
report remains 24 files / 25,346,225 bytes.
