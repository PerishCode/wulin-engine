# ADR 0075: Transactional Actor Render Admission

- Status: Accepted
- Date: 2026-07-15
- Experiment: 0072 Transactional Actor Render Admission

## Context

The runtime already prepares simulation schedule and actor motion from copied state before their
dual commit. Once canonical composition is enabled, however, that prepared actor could be valid in
the published terrain snapshot yet fall outside a non-prefetch pending render window. Committing it
would make the following frame reject after authoritative simulation state had already advanced.

The renderer already owns the exact integer projection and frame preflight for both the published
and non-prefetch pending windows. Adding another public projection query or approximate movement
bound would create a second authority for the same admission decision.

## Decision

- After schedule/motion preparation and before either commit, `Runtime` constructs the complete
  actor candidate by preserving the current handle and presentation and replacing only motion.
- When canonical composition is enabled, the candidate must pass the renderer's existing private
  actor preflight. Only then may actor motion and the prepared schedule commit together.
- Canonical-disabled advances retain their prior behavior because no actor render projection exists
  before publication.
- Admission covers the published pair and any non-prefetch pending pair, exactly matching frame
  preflight. Speculative prefetch remains excluded.
- Rejection remains an explicit runtime error. Typed nonfatal application backpressure, retry,
  elapsed-time retention, command substitution, and traversal policy are deferred.

## Consequences

- Every successful canonical simulation/actor advance is render-admissible for the next frame
  against all authoritative composition windows known to frame preflight.
- A failed admission mutates neither actor nor schedule and leaves the pending composition
  transaction unchanged; the retained actor can still render.
- The change adds no projection surface, compatibility path, frame algorithm, shader, GPU resource,
  upload, copy, barrier, fence, wait, or synchronization owner.
- Prototype horizontal input and traversal remain unaccepted until an application-level outcome can
  treat expected admission pressure without terminating the process.

## Evidence

Experiment 0072 records unchanged prepublication fractional commit, one admissible shared-window
step, one exact pending-window rejection with actor/schedule/composition rollback, a successful
retained-actor frame, pending release/publication, and the unchanged actor GPU hash baseline.
`runseal :canonical-actor` passed in 31.577 seconds; focused engine-runtime tests and the repository
guard also passed.
