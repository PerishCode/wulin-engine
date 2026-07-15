# ADR 0071: Frame-Safe Actor GPU Admission

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0069 defines an exact immutable actor projection into the published render window, and ADR
0070 removes downstream source-page dependence from the visible record. The live runtime actor can
now enter rendering without impersonating a streamed object, but it still needs a distinct
candidate address, exact generation identity, and frame-safe CPU-to-GPU lifetime.

A dynamic page, actor-only draw path, or copied default resource would introduce a second scene
authority or unnecessary synchronization. Reusing a streamed address would corrupt candidate
identity and source ownership.

## Decision

- Reserve candidate address 25,600 for the sole runtime actor. Streamed candidates remain
  `0..25_600` with unchanged addresses and immutable source pages.
- Extend the shared visible record to 56 bytes and encode identity as two words. Streamed identity
  is zero-extended; actor identity is the complete `u64` generation without hashing or truncation.
- Own one upload resource with one 56-byte slot per swap-chain frame. Write only after the existing
  fence for that physical frame slot has completed and bind the slot directly as a root SRV.
- Admit the actor through one extra X group of the existing skeletal cull dispatch, then reuse the
  same pose, mesh, surface, shadow, and occlusion execution.
- Reject an actor outside the current or non-prefetch pending render window before GPU command
  recording. Do not clip, hide, mutate a source page, or fall back to another coordinate path.
- Add no GPU copy, render/compute pass, barrier, signal, fence, wait, alternate renderer, or
  independent actor GPU lifecycle path.

## Consequences

- The prototype's capacity-one actor now has actual GPU presentation while CPU/runtime authority,
  exact motion, presentation, and generation remain unchanged.
- Fixed declared resource widths grow by 416,148 bytes and one resource object. The root signature
  remains within its 64-DWORD limit at 63 DWORDs.
- Surface semantic attachment can distinguish `runtime.actor` at object ID 98,305 while streamed
  terrain/region semantic ranges remain disjoint.
- Camera following, input mapping, gravity, locomotion, multiple actors, Wulin content, and general
  dynamic-entity storage remain separate unaccepted decisions.

## Evidence

Experiment 0068 accepted exact visible/rejected/despawn/respawn/replay and outside-window frame
transactions in a 21.539-second fresh-source GPU workflow. Generation 1 and 2 produced distinct
exact 56-byte compacted-record hashes, and the visible actor changed workload by precisely 16
meshlets, 1,014 vertices, 576 triangles, and 4,056 skin influences. The no-actor frame retained its
external hashes and streamed counts.

The focused 64-publication resource workflow passed in 260.779 seconds. The final 739.3-second
canonical workflow passed all fault, traversal, prototype, resource, and 16 lifecycle gates; its
active handles converged from 496 to 488. Repository guard passed with zero Flavor denies.
