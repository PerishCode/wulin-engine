# Experiment 0067: Self-Contained Visible Record

Status: Accepted

## Hypothesis

The existing skeletal cull can publish one self-contained visible record containing grounded
window-relative position, authored height, semantic region, and presentation identity so that
surface, shadow, and occlusion execution no longer reinterpret `physical_index` as an address into
immutable streamed region pages, without changing submitted work, rendered output, synchronization,
or resource lifetime.

## Scope

Replace the visible record's streamed physical address with the canonical position and semantic
data already computed during culling. Downstream stages consume that record directly. The source
region instance, identity, presentation, and terrain pages remain inputs to skeletal culling only.

This experiment does not inject the runtime actor, increase candidate capacity, add dynamic GPU
storage, define actor semantic IDs, change camera policy, add a renderer mode, or alter streaming.

## Workload

1. Define one identical visible-record layout in skeletal, surface, and occlusion shaders with
   grounded window position, height, semantic region, archetype/LOD, stable key, pose/candidate,
   material, yaw, and animation.
2. Emit that record once in skeletal culling and remove downstream instance/ground reloads.
3. Remove the surface root-signature ranges for region instances and ground numerators; copy only
   the catalog and palette descriptors still consumed from the skeletal heap.
4. Use the exact record stride for visible and filtered UAV/SRV descriptors and report the fixed
   byte delta.
5. Prove shader compilation and focused Rust checks first, then run one targeted live GPU gate.
   Run `runseal :guard` and the full canonical workflow once at the merge checkpoint because the
   GPU record, descriptor exposure, shaders, and all downstream consumers change.

## Controlled Variables

- Candidate count, candidate indices, active-window ordering, cull/LOD decisions, presentation
  clock, palette construction, surface materials, occlusion policy, and indirect dispatch remain
  unchanged.
- Grounded position uses the same window-relative float operations already executed by skeletal
  culling and downstream canonical-position reconstruction.
- The source-resident 50-slot object heap remains immutable and remains a skeletal-cull input.
- No new resource, descriptor heap, command-list phase, barrier, fence, copy, or readback is added.

## Metrics

- Visible-record stride and exact committed byte delta for visible, filtered-visible, and existing
  readback resources.
- Root-signature descriptor ranges and copied descriptor count removed from downstream execution.
- Canonical source/visible/occlusion/submission counts, pose and surface evidence, external capture
  hashes, raw shadow-depth hash, GPU timings, resource plateau, and lifecycle results.
- Focused gate, `runseal :guard`, and full canonical elapsed time.

## Acceptance Criteria

- `physical_index` is absent from the live visible record and no surface, shadow, or occlusion
  shader binds or reads streamed region instances or ground numerators.
- Grounded position, height, semantic region, presentation, pose, candidate, and stable identity
  survive visibility and occlusion compaction exactly.
- The record has one explicit stride used by every visible/filtered SRV and UAV; no legacy shorter
  write stride remains.
- Candidate/submission counts, color/PNG/semantic-ID/diagnostic captures, surface samples,
  shadow classification/occupancy/range, occlusion evidence, resource counts, synchronization
  counts, plateau, rollback, and lifecycle acceptance remain unchanged except for the predefined
  fixed buffer-byte delta.
- Because the grounded position crosses one explicit GPU record boundary instead of being
  reconstructed separately in each pass, one new raw shadow-depth hash is accepted only if it is
  deterministic on immediate replay and all external captures plus the shadow semantic evidence
  above remain exact.
- No actor GPU path, compatibility alias, fallback reload, or alternate renderer is introduced.

## Reference Environment

The experiment uses the repository-pinned Rust/DXC toolchain, reference Windows D3D12 runtime,
freshly cooked signed canonical sources, and the sole canonical acceptance workflow.

## Evidence

The first focused GPU run reached the occlusion readback oracle in 11.5 seconds and exposed a stale
nine-word CPU probe assumption after the shader record grew. The probe now derives its record word
count from the sole stride constant and names the candidate word explicitly.

A 52-byte grounded-position variant then preserved the accepted color, PNG, semantic-ID, and
diagnostic hashes exactly while changing only the raw shadow-depth hash. A 56-byte alternative
carrying source position and the original Q16 ground numerator restored the old shadow hash but
changed the final color/PNG hashes. The experiment rejects that larger split reconstruction: it
would preserve an internal historical bit pattern at the cost of the actual output and an extra
four bytes per visible record.

The accepted 52-byte record grows each 25,600-record visible or filtered buffer from 921,600 to
1,331,200 bytes and the paired order readback from 1,843,200 to 2,662,400 bytes: a fixed total
increase of 1,638,400 committed bytes. Surface descriptor copying falls from 62 descriptors to the
10 catalog descriptors still consumed; region instances and ground numerators are no longer
exposed downstream. No resource object, dispatch, barrier, fence, copy phase, or readback phase was
added.

The maintained `runseal :canonical-frame` gate passed in 21.1 seconds over freshly cooked minimal
sources. Its immediate replay preserved the accepted color, PNG, semantic-ID, and diagnostic hashes
and repeated raw shadow-depth SHA-256
`34481150db654d8955b7efdae0eaf55eaca039ec50bfc679092068b0c4ae4ebd`. Both frames retained 10,538
visible casters, 88,557 occupied shadow texels, 960,019 clear texels, one shadowed and five lit
controlled samples with zero mismatch, 2,084 occluded candidates, and 8,454 stable survivors.

The first full seal reached the 64-publication resource gate after all earlier correctness, host,
presentation, rollover, and 32+32 traversal gates passed. It then exposed that the old check compared
active inspect handles directly with a quiescent process sample. The replacement
`runseal :canonical-resources` workflow records the states separately. Its 276.9-second accepted run
started at 531 handles/22 threads, sampled 515-531 handles with flat private bytes during 64 active
publications, and recovered to 516 handles/18 threads after work stopped.

`runseal :guard` passed with zero Flavor denies. The final 762.4-second canonical workflow passed all
correctness and failure gates, both host lifecycles, presentation mutations, imported content,
rollover, 32 reactive and 32 prepared crossings, the active/quiescent resource plateau, and all 16
complete lifecycle cycles. Experiment 0067 is accepted without an actor GPU path, alternate
renderer, compatibility reload, or new lifetime owner.
