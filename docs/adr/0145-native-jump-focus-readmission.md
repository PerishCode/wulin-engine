# ADR 0145: Native Jump Focus Readmission

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0142 Native Jump Focus Readmission

## Context

The maintained focus process proved that a Space edge immediately before focus loss cannot cross
the activation/time discontinuity. The separate Jump-readmission process proved a second grounded
press after a completed flight, but did not prove that focus cleanup ends an already-held Space
lifetime without poisoning the next press.

## Decision

- Keep the first native Space held after its exact first flight lands.
- Atomically post duplicate Space-down and focus loss on the same visible PID/window thread.
- Resume focus and retain the existing normalized Space-up/Space-down/delayed-Escape sequence.
- Require exactly one suspend/resume/reset boundary and one exact second airborne trajectory.
- Keep the Experiment 0141 focus-locomotion process unchanged as the stale pre-loss action/intent
  suppression owner.
- Add no product action/input state, queue, report field, compatibility path, process, Runtime
  behavior, or renderer/GPU/source/resource/synchronization ownership.

## Consequences

- Focus loss is proven to terminate held Space without repeating Jump.
- A later normalized Space press is proven to start a fresh grounded Jump after clock recovery.
- The stronger proof reuses the existing Jump-readmission child and exact two-value session rather
  than creating another process or diagnostic output.

## Evidence

`canonical-prototype-v57` passed on its first run in 170.997 seconds with a 445,067-byte report.
PID 26908 held Space through the completed first flight, then atomically posted duplicate
Space-down/focus-loss on thread 23692 with zero batch span. After recovery, exact
Space-up/Space-down produced a seven-step second flight before Escape at 118.1957 ms, with velocity
3,116 Q16 and rise 25,571 Q16. Actor identity, horizontal position, shape, Survey presentation, and
animation epoch remained exact.

The clock recorded exactly one suspend, one resume, and one post-resume reset; suspended samples
advanced `0 -> 84`, Ready/sample counts advanced `2/3 -> 299/385`, and stall/render-block counts
remained zero. Object policies stayed idle and the process emitted exactly two values. All 103
engine-runtime, 48 Prototype, and 20 reference-host tests passed; Flavor remained at zero denies
and five existing warnings.
