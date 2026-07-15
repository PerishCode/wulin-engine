# Experiment 0069: Prototype Gravity Admission

Status: Accepted

## Hypothesis

The non-diagnostic prototype can own one fixed negative vertical acceleration and submit it only
through the accepted Ready-only simulation-actor transaction, keeping its initially touching actor
exactly grounded with zero retained vertical velocity, without adding an engine API, horizontal
movement, input mapping, camera policy, renderer path, or GPU resource.

## Scope

Replace only the prototype's zero vertical-acceleration command with one explicit fixed Q16
constant. The existing message-pump, input/exit, activation-aware sampling, Ready admission,
schedule/actor dual commit, frame, and readiness order remain unchanged.

The constant is prototype policy. `engine-runtime` continues to own fixed-step integration,
planar-first ordering, exact terrain contact, batched rollback, actor identity, and presentation.
The actor begins exactly touching committed terrain; every due step predicts downward motion,
resolves contact, and resets grounded velocity to zero.

Horizontal displacement, step-up tuning, action mapping, jumping, camera following, animation
selection, multi-actor storage, object collision, networking, and Wulin content are out of scope.

## Workload

1. Define one exact prototype gravity step-acceleration constant and prove its fixed 60 Hz physical
   interpretation without adding a configurable gameplay surface.
2. Submit zero planar displacement, zero step-up limit, and the gravity constant for every Ready
   sample; keep Reset, Suspended, and Stalled samples non-admitting.
3. Advance the generation-one imported actor through one or more due steps and require exact input
   and output motion equality, touching height, zero output velocity, stable handle/presentation,
   and one terrain query per step.
4. Repeat direct process launches and Sidecar start/restart/stop, retaining strict startup failure
   behavior and structurally identical normalized gravity evidence across fresh processes.
5. Add a maintained focused prototype wrapper that cooks only its required sources and runs this
   real-process/lifecycle gate without invoking the full canonical runtime workflow.

## Controlled Variables

- Simulation remains 60 Hz with the existing bounded elapsed and maximum eight due steps per
  advance.
- Gravity is one signed Q16 increment per fixed step. No wall-time scaling or floating-point
  integration is introduced.
- Planar deltas and step-up limit remain exactly zero. Input is still used only for Escape.
- The touching terrain sample, actor generation, imported presentation, camera, published pair,
  and renderer path are unchanged.
- A failed simulation transaction, failed frame, or non-Ready host sample exposes no alternate
  actor mutation or gravity path.

## Metrics

- Exact gravity command and implied fixed-step acceleration.
- Due-step count, terrain-query count, input/output actor motion, handle, and presentation.
- Direct restart and Sidecar lifecycle evidence with distinct process identities.
- Focused workflow elapsed time and repository guard result.
- Engine API, renderer pass, resource, copy, barrier, fence, wait, and allocation deltas.

## Acceptance Criteria

- Every Ready-admitted due step receives the same negative prototype gravity constant; all
  non-Ready outcomes perform no simulation advance.
- An initially touching actor remains byte-exact at its starting position and center height after
  the batch, with zero output vertical velocity and one terrain query per step.
- Actor handle and presentation remain byte-exact, and readiness is still published only after a
  nonzero successful dual commit and its following successful frame.
- Direct restarts and Sidecar lifecycle retain normalized evidence and leave no owned process.
  Invalid, missing, and corrupt bootstrap sources emit no readiness.
- The focused prototype workflow and `runseal :guard` pass. No engine API, renderer/GPU resource,
  synchronization, compatibility, fallback, or diagnostic prototype surface is added.

## Reference Environment

The experiment uses the repository-pinned Rust toolchain, reference Windows host, canonical signed
terrain/object sources, capacity-one runtime actor, and the sole canonical renderer.

## Evidence

The prototype now owns `GRAVITY_STEP_ACCELERATION_Q16 = -179` and submits it through the existing
Ready-only schedule/actor transaction. At 60 Hz this encodes `-9.832763671875 m/s²`, within half of
one integer encoding step of `-9.81 m/s²`. Planar displacement and step-up limit remain zero. The
readiness revision is `live-prototype-gravity-driver-v1`; no engine API or renderer code changed.

Three actor/bootstrap tests passed, including the exact fixed-step gravity encoding and initial
touching body, together with the prototype time-policy test and all 21 reference-host tests.
`engine-runtime` and prototype checks plus Deno formatting/type checks passed.

`runseal :canonical-prototype` cooked only two required signed centers, including a controlled
corrupt-source center, then completed in 32,032.031 ms. Direct process IDs 13,596 and 29,432 were
distinct. Both emitted the exact command `(0,0,0,-179)`, one due step, one terrain query, unchanged
generation-one motion/presentation, zero output velocity, and readiness only after the following
successful frame. Their wall elapsed and bootstrap frame counts were intentionally not compared;
their normalized policy/actor evidence was identical.

Unknown bootstrap fields, a missing object source, and the controlled corrupt object payload all
failed without readiness. Sidecar start and restart owned distinct process sets; final stop left no
prototype or broker PID. Fresh terrain/object hashes were
`17b07794c223c107f17dea9046bc390671501b3b79fa5249428e3dc20a68ab0b` and
`c65096adfe3b3c36897ce562ef81678030b6c4e7884a3e36b47a5381373d7dba`.

The change adds zero engine API, renderer pass, GPU resource, copy, barrier, fence, signal, wait,
or compatibility surfaces. The long canonical frame/resource/end-to-end workflows were not run:
the actor snapshot, frame code, shaders, GPU resources, synchronization, and shared runtime
lifecycle are unchanged, while the focused gate exercises the real prototype frame and lifecycle.

## Conclusion

The hypothesis is accepted. Prototype v0 now applies one honest fixed gravity policy through the
already accepted transaction, and the grounded actor remains exact. Horizontal input and camera
behavior can be evaluated later without bundling the force required for downhill recovery.
