# ADR 0113: Retired Transient Object-Action Report

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0110 Retired Transient Object-Action Report

## Context

Early object-action experiments serialized the private same-frame `Attempt` and echoed
`FrameCompletion { applied, feedback }` in canonical readiness so acceptance could inspect the
first action. The renderer now returns exact projected feedback, the Prototype commits exact
acknowledgement/counter/consumption/exclusion/suppression state, and the bounded session proves
post-readiness behavior. The transient copies no longer own unique evidence.

Keeping them would preserve an unnecessary compatibility schema and a duplicate policy return that
can diverge from the state it supposedly describes.

## Decision

- Delete `object_interaction_driver.attempt` and `.completion` from readiness with no alias,
  fallback, version branch, or compatibility decoder.
- Delete `FrameCompletion` and make `Policy::complete_frame` return `Result<()>`.
- Delete transient `Attempt` report serialization. `Attempt` remains private same-frame policy
  input only.
- Focused acceptance derives proximity/facing from the independent exact observation result and
  committed actor output, and uses returned projected feedback plus committed policy state as the
  sole action authority.
- Add a stable static guard for the complete removal.

## Consequences

- Successful-session stdout is smaller and contains no copied transient action result.
- Tests assert durable state and renderer-returned facts instead of an echo from the same policy
  call.
- This cleanup changes no input, action eligibility, feedback, acknowledgement, consumption,
  exclusion, suppression, source/window lifetime, engine/GPU work, or resource shape.

## Evidence

The change deletes the two readiness fields, `FrameCompletion`, its two return construction sites,
the transient `Attempt`/facing report mapper, composition-root plumbing, test return-echo
assertions, and every acceptance consumer with no alias or fallback. The existing Prototype session
guard owns stable absence checks, avoiding another guard module.

All 45 Prototype tests and `runseal :guard` pass with zero Flavor denies.
`canonical-prototype-v27` passes in 81.285 seconds: front/side ID 496 retains exact
Activated/Rejected projection and independently derived proximity/facing; the sustained session
retains consumed ID 496, exclusion-oracle target ID 501, 12 capacity-rejected frames, and 783
suppression frames from live frame 4 through 798. The live readiness driver has neither removed
key. No engine, renderer, shader, ABI, resource, descriptor, copy, readback, or synchronization
source changed.
