# ADR 0064: Composed Host Time Admission

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

The reference host independently owns exact bounded monotonic sampling and bounded Win32 activation
reduction. Their live ordering is still undefined. Sampling before consuming a focus interruption
can admit elapsed across suspension, while exposing separate clock pause controls lets future
composition roots choose inconsistent sequences.

Neither application currently has a retained simulation-body consumer, so application wiring would
sample and discard time rather than prove a useful end-to-end boundary.

## Decision

- Make ordered activation-aware sampling the sole public `HostClock` transition.
- Apply the complete activation batch before taking exactly one monotonic sample.
- Treat suspended/resumed order literally; every resume clears the baseline and requires reset.
- Prepare the combined clock transition on a candidate and commit only after success.
- Remove public independent `suspend` and `resume` mutation.
- Keep Win32 transport, application-loop consumption, Runtime invocation, input mapping, frame
  behavior, and gameplay policy deferred.

## Consequences

- A later composition root gets one ordering-safe host-time operation rather than independent
  transition controls.
- Loss/resume bursts cannot leak stale elapsed into simulation, and final suspended state remains
  explicit.
- The policy still has no live consumer; application ownership remains a separate experiment.

## Evidence

Experiment 0061 added activation-before-sample ordering, both interrupted batch orderings, complete
candidate rollback, and deterministic replay to the existing exact clock boundary tests. All 21
reference-host tests and both unchanged application compilation checks passed. The fixed replay
SHA-256 is `15ab39e6b25ea2a63a97378c51f7ec73242d53d87331245174b4efffef01301e`.

`runseal :init` and `runseal :guard` passed with zero Flavor denies. No process or canonical runtime
workflow was run because the composed policy remains disconnected from applications, Runtime,
frames, GPU resources, synchronization, and lifecycle behavior.
