# Experiment 0068: Frame-Safe Actor GPU Admission

Status: Accepted

## Hypothesis

The sole live runtime actor can enter the existing skeletal candidate dispatch as one fixed dynamic
candidate derived from the accepted integer render projection, using one frame-slotted upload
resource and one exact 64-bit generation identity, without becoming a streamed object, adding a
second scene or render path, or introducing a GPU copy, pass, barrier, fence, or wait.

## Scope

Copy the current runtime actor into each frame transaction, project it only after the canonical
composition for that frame is enabled, and encode the projection as the same self-contained visible
record consumed by surface, shadow, and occlusion. The streamed candidate range remains
`0..25_600`; the capacity-one actor owns candidate address `25_600`.

The logical actor candidate has two physical upload slots because the renderer has two frames in
flight. The CPU writes only the slot whose previous fence has completed, then binds that slot as a
root SRV. An absent actor writes an explicit empty record. The upload contains no global float
coordinate: Q9 window position and Q16 body height are converted only into the existing bounded
visible-record float fields.

This experiment does not add actor movement policy, camera following, input mapping, gravity,
multi-actor storage, streaming mutation, a dynamic object page, a renderer mode, or Wulin content.

## Workload

1. Extend the self-contained visible record from one 32-bit stable key to an exact two-word stable
   identity. Streamed records zero-extend their accepted key; the fixed dynamic candidate carries
   the full actor generation.
2. Add exactly one upload resource containing one visible candidate per frame slot. Bind the
   current slot directly as a root SRV after its frame fence is complete.
3. Increase the existing candidate/visible/occlusion capacities by exactly one while preserving
   all streamed candidate addresses and the 15-bit visibility attachment encoding.
4. Extend the existing cull dispatch by one X group. Only lane zero of its first Y group reads the
   actor slot; all streamed groups retain their current address and source reads.
5. Include the admitted actor in the CPU skeletal, surface, and occlusion oracles and expose exact
   source/compacted visible-record identity evidence.
6. Prove absent, visible, rejected-by-frustum, motion/presentation replacement, despawn, respawn,
   immediate replay, and alternating-frame-slot behavior with a maintained focused actor workflow.
7. Run `runseal :guard` and the complete canonical workflow at the merge checkpoint because this
   changes shaders, fixed GPU resources, frame ownership, culling, and lifecycle evidence.

## Controlled Variables

- The actor remains authoritative in the capacity-one runtime slot. Rendering receives a copied
  frame snapshot and never mutates actor motion, presentation, generation, or schedule state.
- Streamed object resources remain immutable skeletal-cull inputs. Actor admission reads no region
  instance, local-ID, presentation, terrain page, or ground-numerator descriptor.
- Candidate addresses `0..25_599`, streamed stable identities, active-window ordering, terrain
  grounding, presentation time, LOD thresholds, pose reuse, mesh execution, surface material,
  shadow, and occlusion policy remain unchanged.
- Candidate address identifies the source class; the two-word stable identity preserves the exact
  source key within that class. No hash or truncation substitutes for actor generation.
- The actor upload is read directly from upload memory. There is no default resource, staging copy,
  queue signal, new command-list phase, or explicit synchronization beyond the existing frame-slot
  fence wait.
- A live actor with an enabled composition must project successfully against the current and any
  non-prefetch pending publication. Outside-window admission fails before GPU command recording
  rather than clipping, hiding, or falling back to another coordinate.

## Metrics

- Logical streamed/dynamic/total candidate capacities and exact fixed buffer-byte deltas.
- Actor upload resource count, frame-slot count, record bytes, write count, copy count, and fence
  behavior.
- Exact candidate address, semantic region, bounded position, height, presentation, generation
  words, source-visible record, occlusion classification, and compacted record equality.
- GPU/CPU visible, rejected, animated/static, pose, meshlet, vertex, triangle, skin-influence,
  surface, shadow, and occlusion counts with and without an actor.
- Deterministic color, semantic-ID, diagnostic, shadow-depth, visible-order, and actor-record hashes.
- Process resources across focused alternating frames, the canonical 64-publication plateau, and
  all 16 complete lifecycle cycles.

## Acceptance Criteria

- There is exactly one logical dynamic candidate, at address `25_600`, and one frame-slotted upload
  resource. No streamed address changes and no actor data is written into a source-resident page.
- The full `u64` actor generation and complete presentation survive culling and any occlusion
  compaction exactly. Despawn clears admission; respawn changes generation identity without stale
  data from either physical frame slot.
- An in-frustum actor changes GPU and CPU oracle counts by exactly its authored workload and is
  present in surface, shadow, and occlusion execution. An out-of-frustum actor increments rejection
  only; an outside-window actor rejects the frame transaction.
- No-actor canonical output hashes and streamed workload counts remain exact. Only the declared
  identity-stride, capacity-one, upload-resource, and derived readback byte deltas are accepted.
- Alternating actor states across at least four submitted frames prove that CPU writes never race a
  prior frame slot. Immediate replay of the same actor state is byte-identical.
- The implementation adds no alternate renderer, compatibility field, fallback coordinate, source
  alias, GPU copy, command-list pass, barrier, fence, wait, or independent lifecycle path.
- Focused actor acceptance, `runseal :guard`, the full canonical workflow, resource plateau, and all
  lifecycle cycles pass with zero Flavor denies.

## Reference Environment

The experiment uses the repository-pinned Rust/DXC toolchain, the reference Windows D3D12 runtime,
freshly cooked signed canonical sources, the two-buffer swap chain, and the capacity-one runtime
actor.

## Evidence

`runseal :canonical-actor` completed in 21.539 seconds over freshly cooked signed sources. The
baseline had 25,600 streamed candidates, zero dynamic candidates, 10,538 visible objects, and
15,062 frustum rejections. Admitting the actor at candidate 25,600 added exactly one visible,
animated, reused-pose object; 16 meshlets; 1,014 vertices; 576 triangles; and 4,056 skin
influences to both GPU and CPU oracles. It changed no active/evaluated pose count and produced one
`runtime.actor` semantic object with ID 98,305 and 3,866 pixels.

Generation 1 and 2 survived source visibility and stable occlusion compaction with exact 56-byte
record hashes `2f14c5bbe4268821f0cdd3cbc7a8fbce6d92c08c944476a91f82f353f505ac3a` and
`ab33d261aebcc263cf32a7849fb3637a2a82e37c130a021a1962327a8198dc78`. Immediate replay was
byte-identical, generation/presentation replacement changed the record, and both despawn clears
removed it. A third in-window actor changed only the rejection count. A fourth actor outside the
window failed before command recording with frame index 1,343 unchanged; its actor state remained
exact, and despawn was followed by a successful frame. The frame-slot upload write counter advanced
from 12 to 21 over the nine successful submitted frames and did not advance for that failed frame.

The actor capture fixed color, PNG, object-ID, and diagnostic hashes at
`d7fbc2e6c26cfe7f74a8b5751e1e6239f07f37f3e0204322032fe7ca4e50329e`,
`7e9d40d716870dd7a00dd54cebfd333ee9344de086093566b15b74b8e635962c`,
`a9fa7a8f1bec4fc5fe62dc7a969bcd87332b308352e59bf0616c2248841bcaf6`, and
`eba4b91018819328e51b9a503ba00503887b3a4aa88bf3dd4426abc6d888b3bb`. The 16.284-second
no-actor frame/replay workflow retained its four prior external hashes and exact streamed workload.

The declared fixed resource widths increase by 416,148 bytes: 409,824 for four record-capacity
copies affected by the 52-to-56-byte identity extension, 6,164 for the capacity-one palette and
index/mask/group additions, 48 for two enlarged surface-sample resources, and 112 for the two-slot
actor upload. Exactly one resource object is added. The upload is a root SRV and reports zero GPU
copies; the existing cull dispatch grows by one X group, occlusion grows from 100 to 101 groups,
and no pass, barrier, signal, fence, wait, or independent lifecycle path is added.

The focused 64-publication resource workflow completed in 260.779 seconds with a 504-handle peak
and a 496-handle final quiescent plateau. `runseal :guard` passed with zero Flavor denies. The final
739.3-second `runseal :canonical-runtime` workflow preserved canonical correctness, all fault and
traversal gates, prototype startup/restart with its live actor, a 496-to-488 bounded resource
plateau, and all 16 complete lifecycle cycles.
