# ADR 0131: Native Diagonal Run

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0128 Native Diagonal Run

## Context

Run diagonal normalization and the Shift/W/A forward-left path had exact unit
coverage but no dedicated real native process proof. Two expanded runs also
showed that a helper-ready handshake and a later synchronous window-response
probe did not prevent the first live readiness frame from overtaking input
posted after that probe.

## Decision

- Add one atomic exact-window Shift/W/A startup batch, delayed Escape, and exact
  two-value diagonal Run session.
- Require equal negative 45-Q9 X/Z components, clip 2, yaw 40,960, stable
  animation epoch, default orbit, clock continuity, zero blocks, and idle object
  state.
- Permit single-key atomic batches and queue every startup request's complete
  zero-delay prefix while the selected window thread is suspended.
- For delayed Run release and re-press, atomically queue only the initial
  zero-delay prefix and preserve the authored later transition delay.
- Replace native window-action evidence schema 3 with schema 4 and require exact
  `atomicPrefixLength`; keep no old decoder, fallback, or alternate path.

## Consequences

- Shift/W/A ingestion, Run diagonal normalization, and forward-left presentation
  have exact real-process evidence.
- Startup press edges and held commands are queued before the selected window
  thread resumes; there is no response-before-post gap.
- Acceptance gains no retry, product delay, threshold relaxation, event stream,
  journal, copied state, or extra product output.
- Product input/locomotion/presentation, Runtime, renderer/GPU resources,
  synchronization, and object policy remain unchanged.

## Evidence

The first v43 run failed at the prior E/W readiness gate with stationary
orbit-zero Survey. A synchronous response probe was removed after the next run
showed that E-down could still lose the race at the prior camera re-press gate.
Focused schema-4 probes then proved single E, E/W, and single Space atomic
startup outcomes plus unchanged delayed Run release/re-press semantics.

The final `canonical-prototype-v43` passed in 161.489 seconds with every
previous gate. PID 8,092 queued Shift/W/A on thread 7,344 in a 0.0026 ms atomic
span. Readiness was `(-45,-45)`, Run clip 2/yaw 40,960/epoch 5; completion was
`(-585,-585)`, 12 further exact diagonal steps with epoch still 5. Escape
measured 207.613 ms, clocks advanced `4/5 -> 61/62`, render blocks stayed zero,
object state stayed idle, and output contained exactly two values. Flavor
reported zero denies and `runseal :init` passed.
