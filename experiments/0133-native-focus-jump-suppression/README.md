# Experiment 0133: Native Focus Jump Suppression

Status: Accepted

## Hypothesis

The existing focus-discontinuity process can prove current action ordering without another child or
product output. If Space and W are atomically queued immediately before focus loss on the exact
ready window thread, neither the Jump edge nor held locomotion may reach a later nonzero simulation
after suspension, reset, and resumed Ready progress.

## Scope

- Replace the W-only focus-loss helper with one exact-PID atomic Space/W/focus-loss batch.
- Retain the existing resume delay, Escape completion, suspended sampling, clock recovery, and
  exact unchanged-actor oracle.
- Require grounded idle Jump state at readiness and enough resumed Ready progress to exclude a
  process that simply remained suspended.
- Guard against restoration of the W-only helper and the rejected temporary helper name.

Product HostInput/Jump/locomotion policy, activation/clock ordering, session schema, Runtime,
renderer/GPU resources, source formats, synchronization, and process count are out of scope.

## Workload

1. Audit the product loop ordering from message-pump ingestion through activation-aware sampling,
   action admission, and Ready-only simulation.
2. Reuse the existing focus-discontinuity process after grounded idle readiness.
3. Suspend its exact visible window thread, post Space-down and W-down without delay, then post
   `WM_KILLFOCUS` before releasing the thread.
4. Wait at least 250 ms, post `WM_SETFOCUS`, wait at least 250 ms, and exit normally through Escape.
5. Require one suspend/resume pair, one additional reset, suspended sampling followed by later
   Ready work, zero backlog/stalls/blocks, and a final actor exactly equal to readiness.
6. Run every existing Prototype session, Sidecar lifecycle, static guard, and initialization gate.

## Controlled Variables

- Initial actor/camera/object state, focus delay, resume/exit ownership, product loop ordering,
  Jump impulse, gravity, locomotion, and completion framing remain unchanged.
- The process count and two-value product session surface remain fixed.
- The exact focus batch adds only Space before the retained W and focus-loss messages.
- No intermediate product state, event history, retry, relaxed threshold, fallback, compatibility
  alias, or new action state is admitted.

## Metrics

- Exact process/window/thread identity, ordered native messages, key interval and batch span,
  readiness Jump state, actor byte-equivalent fields, clock suspend/resume/reset/Ready/suspended
  counters, frame/render-block counts, object state, output count, workflow duration, report bytes,
  Flavor findings, and process cleanup.

## Acceptance Criteria

- The batch begins only after readiness and targets that exact PID, visible window, and positive
  window-thread ID.
- Native evidence is exactly focus, Space-down, W-down, focus-loss; both the inter-key interval and
  atomic batch span are in `[0,50]` ms.
- Readiness is grounded with no pending Jump, and completion has exactly one additional
  suspend/resume pair and one post-resume reset.
- Suspended samples increase and later Ready/live-frame progress occurs, while final actor state is
  exactly readiness and render blocks/stalls/backlog remain zero.
- The W-only helper, rejected temporary helper name, extra process, and product/Runtime/GPU/source/
  resource/synchronization changes do not return.

## Results

The product-order audit showed that `HostInput` may expose the Space press in the same ingest that
contains focus loss. The activation-aware sample is nevertheless Suspended, so it admits no elapsed
simulation; a subsequent Suspended or Reset observation clears the pending action before any
resumed Ready transaction. The accepted claim is therefore suppression across the discontinuity,
not immediate deletion of the edge during the original ingest.

The first focused guard rejected the temporary five-word helper name through the existing Flavor
policy. It was renamed directly to `suspendWithActionBatch`; the guard forbids both that rejected
name and the deleted W-only `suspendWithForward` name. No exception or compatibility alias was
added.

Final `canonical-prototype-v48` passed in 144.949 seconds. PID 2252 used visible window thread 20452
to post Space-down and W-down 0.0012 ms apart, followed by focus loss in the same 0.0012 ms atomic
batch. After resume, the clock completed exactly one suspend/resume pair and one additional reset.
It accumulated 740 suspended samples, then reached 1,206 Ready samples and 1,948 live frames with no
stall, elapsed backlog, or render block.

The final actor remained exactly equal to grounded idle readiness: identical generation, signed
region/local position, body height/shape, zero vertical velocity, Survey clip, yaw, and animation
epoch. Object policies remained idle, stdout remained exactly two values, and the process exited
normally through Escape. The ignored report was 447,445 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU, source,
resource, or synchronization code changed.

## Conclusion

Accepted. The existing focus-discontinuity process now proves that a same-batch grounded Space edge
and held W cannot survive suspension/reset into resumed nonzero simulation. The stronger gate
replaces the W-only evidence without adding a process or widening the product surface.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
