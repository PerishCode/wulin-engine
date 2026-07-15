# ADR 0050: Runtime Fixed Simulation Schedule

- Status: Accepted
- Date: 2026-07-15
- Supersedes: None
- Superseded by: None

## Context

ADR 0043 moved presentation time and successful-frame commit into `Runtime` but explicitly left
elapsed wall time, simulation time, and fixed-step policy undefined. ADR 0049 then accepted exact
caller-owned terrain contact without velocity, gravity, or time. Defining vertical dynamics next
would give its velocity and acceleration no accepted quantum; driving it from presentation frames
would also make simulation rate and success depend on rendering.

The prototype currently pumps messages and renders continuously. The reference host has no live
frame clock contract, and its existing `Instant` use is diagnostic bootstrap/frame-duration
measurement rather than simulation authority. Experiment 0047 therefore tests the schedule before
choosing an operating-system clock driver or a spatial consumer.

## Decision

- `Runtime` owns one process-local fixed simulation schedule independent from
  `PresentationTimeline`. It stores a `u64` step tick, sub-step rational remainder, successful
  advance count, and emitted-step count.
- Callers explicitly submit `0..=125,000,000` elapsed nanoseconds. The schedule multiplies by an
  independent 60 steps/second, emits the quotient over a 1,000,000,000 denominator, retains the
  exact remainder, and returns a typed zero-through-eight-step batch.
- Mutation is checked and transactional. Oversized elapsed input or unrepresentable next state
  fails without clamp, elapsed drop, backlog, wrapping counter, or partial commit.
- Runtime frames and presentation controls do not advance simulation. Simulation advance does not
  render, present, mutate presentation time, sample input, read sources, touch the GPU, or dispatch
  per-step work.
- Diagnostic status/advance controls and one isolated long-duration replay probe provide
  deterministic experiment evidence. The probe's timing clock is measurement only; none of these
  controls make the workbench the schedule owner or drive the live schedule.
- Live monotonic clock sampling, stall splitting, pause/focus policy, replay timestamping,
  simulation consumers, and interpolation remain later decisions.

## Consequences

- Vertical dynamics and later actor updates can name one accepted fixed quantum without depending
  on render frequency or presentation success.
- The 125 ms input bound guarantees at most eight due steps per call. A future host must explicitly
  split longer elapsed intervals or define a separate stall policy; the runtime will not conceal
  that choice.
- The scheduler proves timing state and batch production only. It does not yet make the prototype
  interactive or move a body, and it introduces no unused callback/general job abstraction.
- Presentation animation remains frame-driven until a separate experiment decides whether and how
  presentation consumes simulation or interpolated time.

## Evidence

Experiment 0047 proved exact partition/replay behavior, zero drift across 28,800 bounded calls and
216,000 ticks, zero-through-eight batches, transactional invalid/overflow failure, presentation
and render independence, process reset, and zero reported runtime work beyond integer mutation.
The one-hour replay hash was
`1ee26e9eba0160996a3cb554c17f7641ce766728f57f97bc2a1167350ca2a374`. The 692.8-second final
canonical workflow retained fixed GPU/capture/query/contact hashes, 32 reactive plus 32 prepared
traversal samples, a 531-handle plateau with zero transient growth, and 16 clean lifecycle cycles.
