# ADR 0070: Self-Contained Visible Record

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The accepted actor projection cannot enter the GPU path honestly because the current
`VisibleObject.physical_index` is a direct streamed-page address. Surface, shadow, and occlusion
reload that page even though skeletal culling has already resolved canonical position, grounding,
height, semantic region, and presentation.

Appending a dynamic actor with a fabricated physical index would leak game/runtime state into the
immutable source-resident object cache and couple every later pass to that workaround.

## Decision

- First make the canonical visible record self-contained; do not add the actor in this decision.
- Publish grounded window-relative position, height, and semantic region with the existing visible
  presentation and candidate identity.
- Restrict streamed region-instance and ground resources to skeletal culling.
- Remove downstream descriptor exposure and reloads rather than retaining a fallback path.
- Preserve candidate capacity, pass order, synchronization, and resource lifetime.

## Consequences

- Surface, shadow, and occlusion consume one compacted submission contract independent of source
  storage ownership.
- Visible and filtered record buffers grow by a fixed measured amount, while downstream random
  source-page reads and descriptor ranges disappear.
- A later actor experiment can define one distinct candidate producer without impersonating a
  streamed region record, but still must prove capacity, identity, and lifecycle separately.

## Evidence

Experiment 0067 accepted a 52-byte self-contained record and removed all downstream streamed-page
and ground-buffer reads. A 21.1-second fresh-source frame/replay gate preserved all four external
capture hashes and repeated the new raw shadow hash with exact shadow and occlusion semantics. The
final 762.4-second canonical workflow passed 32+32 traversal, a bounded active/quiescent
64-publication resource plateau, and 16 complete lifecycle cycles. The three affected fixed-capacity
buffers grew by 1,638,400 bytes in total without adding a resource or synchronization path.
