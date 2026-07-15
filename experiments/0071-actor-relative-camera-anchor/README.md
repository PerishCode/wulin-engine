# Experiment 0071: Actor-Relative Camera Anchor

Status: Accepted

## Hypothesis

The runtime can atomically derive and publish one camera from a generation-qualified actor and
caller-owned fixed offsets by reusing the sole renderer-internal actor projection authority. The
prototype can apply that rig after every simulation opportunity and before every live frame,
without reopening a standalone projection surface or introducing movement, traversal policy, a
renderer path, or GPU resources.

## Scope

Add one `Runtime` mutation that reads the requested actor, derives its current origin-relative
scene center from the accepted internal projection, adds explicit position/target offsets, and
commits one validated camera. The mutation returns only success or failure; the existing camera
state remains the evidence surface. Invalid actor identity, unavailable projection, non-finite or
invalid camera input, and checked coordinate failure must preserve the prior camera.

The prototype owns one fixed camera rig and invokes the mutation after its existing Ready-only
gravity advance opportunity and before each live frame. Readiness remains gated by a nonzero
simulation commit followed by a successful anchored frame.

Horizontal input, locomotion displacement, traversal enablement, prefetch, rollover exercise,
combined simulation/camera/frame atomicity, multi-actor camera selection, smoothing, collision,
camera-relative animation, Wulin content, and diagnostic controls are out of scope.

## Workload

1. Prove exact actor-center derivation at the canonical center, a local alias, signed seams, and a
   recentered origin while preserving one internal projection authority.
2. Prove camera publication is transactional for stale actor handles, actors outside the current
   render window, non-finite offsets, degenerate look vectors, and invalid field of view.
3. Apply the fixed prototype rig after every simulation opportunity and before every live frame;
   publish its exact offsets, resulting camera, invocation count, and frame count.
4. Repeat valid direct process launches and Sidecar start/restart/stop while retaining invalid,
   missing, and corrupt bootstrap rejection without readiness.

## Controlled Variables

- Actor spawn, fixed gravity, simulation schedule, terrain contact, handle generation, and
  presentation remain unchanged.
- Planar displacement and step-up limit remain zero. Input remains Escape-only.
- The camera rig is fixed prototype policy, not an engine default or configurable gameplay
  surface.
- Traversal remains disabled. The actor stays in the current published center and no origin
  rollover occurs in the process workload.
- Projection equations, frame preflight, shaders, GPU resources, copies, barriers, fences, waits,
  and allocation ownership remain unchanged.

## Metrics

- Exact actor scene-center values across center, alias, seam, and recentered configurations.
- Exact fixed rig offsets, committed camera values, anchor invocation count, and live frame count.
- Prior-camera preservation for every rejected transaction.
- Direct restart equality, distinct process identities, and complete Sidecar lifecycle cleanup.
- Engine API and renderer/GPU resource, submission, synchronization, and allocation deltas.

## Acceptance Criteria

- The runtime derives the actor center only through the private accepted projection and publishes
  a fully validated camera or leaves the previous camera byte-exact.
- Center derivation is exact across local aliases and origin recentering, including signed global
  coordinates; no global coordinate is converted to floating point before bounded integer
  projection.
- Every prototype live frame is preceded by exactly one successful fixed-rig anchor invocation.
  Readiness reports an anchor count equal to its live-frame count and an exact camera derived from
  the generation-one grounded actor.
- Direct restarts normalize to identical actor, simulation-driver, and camera-driver evidence.
  Invalid, missing, and corrupt bootstrap sources emit no readiness; Sidecar stop leaves no owned
  process.
- Focused runtime/prototype tests, `runseal :canonical-prototype`, and `runseal :guard` pass. No
  public projection read surface, diagnostic event, traversal/input policy, renderer/GPU resource,
  synchronization, compatibility, or fallback path is added.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain, reference Windows host, canonical signed
terrain/object sources, capacity-one runtime actor, fixed gravity driver, and sole canonical
renderer.

## Evidence

`Runtime::set_actor_relative_camera` now reads the exact actor handle, reuses
`Renderer::project_actor`, restores the active-window alias with checked Q9 integer arithmetic,
and asks `SceneState` to validate a complete candidate before committing it. The mutation returns
only `Result<()>`; the private projection type and fields remain renderer-internal. No inspect verb
or secondary projection route was added.

The private projection tests now cover scene-center recovery across the signed center, Q9 seams,
local-center alias 96/32, and origin recentering. Scene tests prove exact successful publication and
byte-exact prior-camera preservation for non-finite input, an overflowed `f32` sum, a degenerate
view vector, and invalid field of view. Actor-slot tests already prove missing/stale handle
rejection before any dependent mutation. In total, 74 engine-runtime tests plus the semantic actor
test, four prototype tests, and 21 reference-host tests passed.

The prototype owns the fixed rig `[9,4,12]` / `[0,-1,-3]` / `60` and applies it once after each
simulation opportunity and before its live frame. Its `live-prototype-actor-camera-v1` readiness
records the actor handle, rig, resulting camera, anchor count, and live-frame count. The final
`runseal :canonical-prototype` run completed in 35,607.910 ms. Direct processes 24,020 and 30,320
both reported exact camera position `[9,6.1640625,12]`, target `[0,1.1640625,-3]`, and three anchors
for three live frames; normalized actor, simulation, and camera policy evidence was identical.

Unknown bootstrap fields, a missing object source, and a controlled corrupt payload all failed
without readiness. Sidecar start/restart used distinct owned process sets and final stop left no
prototype or broker PID. Fresh terrain/object hashes remained
`17b07794c223c107f17dea9046bc390671501b3b79fa5249428e3dc20a68ab0b` and
`c65096adfe3b3c36897ce562ef81678030b6c4e7884a3e36b47a5381373d7dba`.

Early focused checks exposed three evidence defects rather than product fallback: a test expected
the aliased Z axis with the wrong sign, another supplied a valid view while claiming it was
degenerate, and the first process gate compared serialized Rust `f32` near-plane evidence against
a JavaScript double literal. The corrected gates retain the signed expectations and normalize the
near plane with `Math.fround`; every failed run cleaned its owned processes.

The merge checkpoint then rejected new test attachments in production source and also rejected
reuse of the retired calibration-era `tests/private/scene.rs` path. The tests moved under the
existing private actor-projection test attachment, and the private `Camera::new` visibility stayed
closed. The guard was not weakened. Final `runseal :init` and `runseal :guard` passed; Flavor
reported zero denies and the same five pre-existing warnings.

No frame algorithm, shader, GPU allocation, upload, copy, barrier, fence, signal, wait, or shared
lifecycle owner changed. The focused real prototype gate exercises the affected frame order;
canonical frame/resource/end-to-end workflows were not mechanically rerun.

## Conclusion

The hypothesis is accepted. Prototype v0 has one deterministic actor-relative camera write path
without reviving standalone projection or prematurely bundling input and traversal coordination.
