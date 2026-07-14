# Experiment 0037: Source-Duration Playback

Status: Accepted

## Hypothesis

The canonical renderer can honor the pinned Fox source-clip durations through its existing 64
sampled poses using one bounded, frame-driven integer clock, while retaining exact deterministic
control, authored presentation authority, fixed GPU submission, zero content movement, and one
renderer path.

## Scope

This experiment assigns a fixed 4,800-unit presentation second and advances 80 units after each
submitted canonical frame. The cooked Survey, Walk, and Run durations must quantize exactly to
16,400, 3,400, and 5,560 units. Fixture clips retain their existing 5,120-unit/64-frame period.
The clock frame is bounded by the common exact period of every fixture and imported clip:
31,002,560 frames or 2,480,204,800 presentation units.

The sampled phase is:

```text
(authored_phase_offset + floor(((clock_frame * 80) mod clip_duration_units) * 64
                               / clip_duration_units)) mod 64
```

The existing 1,024-key rig/clip/phase domain, fixed palettes, descriptors, resources, and
indirect dispatches remain unchanged. Three source durations occupy one aligned four-DWORD block
in the skeletal root constants, and the accepted `clip % 3` alias pattern selects them; no duration
buffer or runtime asset lookup is added.

The clock remains frame-driven rather than wall-clock driven. Frame interpolation, display-rate
pacing, gameplay/network time, clip blending, state transitions, root motion, animation
compression, dynamic rigs, schema changes, and Wulin content are out of scope.

## Workload

1. Validate the three cooked f32 source durations against the fixed 4,800-unit quantum and derive
   all eight imported clip-alias durations exactly.
2. Apply one shared integer phase formula in the CPU oracle and GPU cull/pose work. Retain the
   exact fixture phase sequence for frames 0 through 63 and every subsequent 64-frame cycle.
3. Expand the deterministic clock from modulo 64 to the exact 31,002,560-frame common period.
   Revalidate pause/resume/status, paused set/step, invalid-request rollback, automatic advance,
   content independence, and exact common-period wrap.
4. Publish the authored imported Walk workload and capture controlled frames 0, 42, 43, and 85.
   Require the end-of-cycle pose to differ, both sampled loop returns to reproduce frame 0, and
   GPU palette phase to agree with the CPU duration oracle.
5. Re-run physical reorder, source switching, all I/O/copy holds, corruption rollback,
   traversal/prefetch/rollover, the 64-publication resource plateau, 16 lifecycle cycles, and the
   full repository guard.

## Controlled Variables

- Signed schema-3 spatial/identity/presentation authority, exact `i64` addressing, caches,
  composition, grounding, LOD, material, occlusion, and publication remain unchanged.
- Authored animation enablement, clip, phase offset, and variation remain content data. Duration
  mapping may read rig and clip but may not choose either.
- Fox geometry, skin bindings, hierarchy, inverse binds, 64 sampled poses, normalized-space
  deformation, and conservative animated bounds remain byte-exact.
- One submitted canonical frame advances exactly one clock frame after probe/capture submission.
  Idle-shell frames, content operations, and wall time cannot advance it.
- Set accepts only frames below the declared common period. Step remains paused-only and bounded
  to `1..=4096`; invalid operations must not mutate state.

## Metrics

- Source f32 durations, quantized duration units, alias-duration table, per-clip frame crossings,
  clock frame/period/wrap counts, and CPU/GPU selected phases.
- Active/reused poses, evaluated bones, palette bytes, dispatch counts, root-constant DWORDs,
  resource/descriptors counts, and catalog/GPU bytes.
- Controlled-frame palette, surface, color, PNG, object-ID, publication, source, residency,
  grounding, and identity hashes.
- Traversal continuity, device state, handle/private-byte plateau, and descendant cleanup.

## Acceptance Criteria

- Source durations quantize exactly to 16,400/3,400/5,560 units; every alias duration is nonzero,
  divides the common presentation-unit period, and produces a phase in `0..63` using integer math.
- Fixture frame-to-phase mapping remains `frame mod 64`. Imported CPU and GPU phases agree at all
  controlled frames with no float time arithmetic in the runtime shader path.
- Authored Walk frame 42 differs from frame 0. Frames 43 and 85 reproduce frame-0 sampled pose,
  surface, color, PNG, and object-ID evidence while retaining the same published content.
- Setting the clock to the last common-period frame and stepping once returns exactly to frame 0.
  Invalid set and running-state step requests roll back; automatic and held-pair time continue to
  advance independently of content work.
- The change adds no GPU resource, descriptor, palette byte, content copy, publication, or
  indirect dispatch. The skeletal root constants remain within the D3D12 64-DWORD limit.
- Every Experiment 0036 regression not intentionally superseded by duration-aware timing passes;
  no validation error, false occlusion, device removal, resource growth, or lifecycle residue
  occurs, and the full repository guard passes.

## Evidence

The direct workflow remains:

```powershell
runseal :canonical-runtime
```

Generated evidence will remain ignored under
`out/captures/0037-source-duration-playback/`.

## Results

The direct workflow passed in 539.4 seconds. The pinned Survey, Walk, and Run durations quantized
to 16,400, 3,400, and 5,560 units at 4,800 units per presentation second. Their eight aliases and
the 5,120-unit fixture duration all divided the 2,480,204,800-unit common period, producing the
bounded 31,002,560-frame clock. CPU tests confirmed fixture phase remains `frame mod 64` and every
rig/clip returns to phase zero at the common period.

The controlled imported-duration source presented 10,538 authored Walk objects as one active GPU
pose with 10,537 pose reuses. Frames 0/42/43/85 selected GPU phases 0/63/0/0. Frame 42 produced
distinct color, PNG, and object-ID hashes, while frames 43 and 85 reproduced frame 0 exactly:

```text
frame 0  color 489ed410... png 47635f3f... object 01458973...
frame 42 color 9db1231a... png 459c2b7e... object 708b1807...
frame 43 color 489ed410... png 47635f3f... object 01458973...
frame 85 color 489ed410... png 47635f3f... object 01458973...
```

The maximum controlled CPU/GPU palette delta was `2.3283064e-10`. Tick 64 no longer wrapped the
global clock, while setting frame 31,002,559 and stepping once produced exact all-content frame 0
evidence and incremented the common-period wrap count. Automatic and held-pair timing remained
independent of source, residency, copy, and publication work.

An initial implementation attempted 64 DWORD root constants plus the descriptor table. D3D12
rejected the 65-DWORD root-signature cost before startup. The accepted implementation uploads only
the three source durations in one aligned four-DWORD block and derives the already-verified alias
with `clip % 3`, leaving the complete root signature at 61 DWORDs (60 constants plus one table).
No GPU resource, descriptor, palette byte, or dispatch was added.

All physical reorder, source switching, four hold gates, corruption rollback, restart, rollover,
32 reactive crossings, and 32 prepared crossings passed. The 64-publication resource baseline was
531 handles and 405,532,672 private bytes; peak handles stayed 531, and the final sample was 516
handles and 405,995,520 bytes. All 16 lifecycle cycles stopped without descendants or device
removal. The full repository guard passed with zero deny issues.

## Conclusion

Accepted. The canonical frame clock now honors the pinned source durations with bounded integer
math while keeping authored phase offsets and clip selection as schema-3 content authority. The
runtime remains deterministic and frame-driven and still has no wall-clock, interpolation,
transition, root-motion, runtime asset, or second-renderer path.

## Promotion

Promoted the 4,800-unit presentation quantum, exact 31,002,560-frame common period, per-rig/clip
integer phase mapping, source-duration root constants, and controlled duration evidence. ADR 0040
supersedes the modulo-64 timing boundary in ADR 0036. Display-rate pacing, wall-clock interpolation,
gameplay/network time, clip blending/transitions, and root motion remain later gates.
