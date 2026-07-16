# ADR 0098: Retired Standalone Presentation Status

- Status: Accepted
- Date: 2026-07-16
- Experiment: 0095 Mandatory Presentation Status Cleanup

## Context

The runtime originally exposed presentation time through both `canonical.time.status` and the
`presentationClock` field of `canonical.status`. Both routes serialize the same private
`PresentationTimeline`; no product or manual operator consumes the standalone route. Its remaining
consumers are eleven maintained temporal and actor acceptance reads.

The mutation controls are different: pause, resume, set, and step perform unique state transitions
and return the resulting timeline status. Removing their response would weaken operator evidence,
but keeping a separate read-only forwarder preserves duplicate authority without adding capability.

## Decision

- Delete `Runtime::presentation_time_status` and the complete `canonical.time.status`
  protocol/dispatch chain.
- Read maintained black-box presentation state only from
  `canonical.status.presentationClock`.
- Retain pause/resume/set/step. After their existing operation, return
  `PresentationTimeline::status_json` directly with no public read-only intermediary.
- Keep one current real-process rejection for the retired verb and add a stable guard that rejects
  its method, enum variant, route, and maintained support reads.
- Add no alias, redirect, replacement verb, response field, cache, or product telemetry.

## Consequences

- Presentation state has one inspect read authority while mutation responses keep their exact
  status shape.
- Temporal and actor acceptance continues to prove rollback, wrap, automatic advancement,
  lifecycle, simulation isolation, render admission, and GPU presentation through current state.
- Historical ADRs describing the former five-verb operator remain settled decision history, not a
  live compatibility obligation.
- Product time/action behavior, renderer/GPU resources, sources, formats, assets, networking, and
  Wulin behavior are unchanged.

## Evidence

Experiment 0095 removes the public Runtime method, protocol variant/parser, and dispatch. All eleven
maintained reads consume the pre-existing aggregate, while pause/resume/set/step retain direct
timeline responses. The retired verb returns generic `unknown_event`, and a deliberate method-name
restoration is rejected by the stable guard in under one second.

Focused checks pass 90 engine-runtime tests plus workbench and Deno checks.
`canonical-actor-v8` passes in 80.180 seconds with exact lifecycle/restart, fractional and
coarse/nominal schedule isolation, rollback, pending admission, GPU, and presentation-epoch evidence.

`canonical-runtime-v4` passes in 268.804 seconds with exactly one retired-event rejection, complete
manual/wrap/rollback/automatic/held-publication temporal evidence, both 32-sample traversal sweeps,
an eight-publication bounded resource checkpoint, and two clean lifecycle cycles. Repository guard
passes with zero Flavor denies.
