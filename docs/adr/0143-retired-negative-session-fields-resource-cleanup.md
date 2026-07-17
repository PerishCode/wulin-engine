# ADR 0143: Retired Negative Session Fields and Resource Cleanup

- Status: Accepted
- Date: 2026-07-17
- Experiment: 0140 Retired Negative Session Fields and Resource Cleanup

## Context

Prototype readiness/completion still emitted six static false-valued fields declaring that no
event stream, event history, or copied object state existed. Rust tests and TypeScript acceptance
then checked or copied those negative declarations. They carried no changing state and duplicated
the stronger current positive ownership/session evidence. The workspace also retained more than
3.2 GB of compiler and generated acceptance output after full validation.

## Decision

- Delete all six static negative product fields and fourteen downstream test/check/summary
  occurrences.
- Advance the exact Prototype session schema directly from v1 to v2; add no decoder, alias,
  optional-field branch, or replacement flag.
- Make the existing Prototype session guard scan all six current owners as the sole return
  authority.
- Advance complete Prototype and Runtime workflow revisions to v55 and v19.
- After validation and the guarded commit, delete only resolved workspace-local `target/` and
  `out/`. Do not add a recurring cleanup wrapper or touch `.task/` or shared global caches.

## Consequences

- Session reports contain only positive current contract and state fields.
- Existing two-value process framing, input/action behavior, object state, Runtime, renderer/GPU,
  source, synchronization, resource, and lifecycle ownership remain unchanged.
- Future builds regenerate only the compiler/output resources they require.
- Cleanup remains an explicit scheduled operation rather than product behavior or repository
  command surface.

## Evidence

Both session-report tests and `runseal :guard` passed. `canonical-prototype-v55` passed in 170.874
seconds with a 439,342-byte report, 58 v2 contract occurrences, zero retired fields, and every
existing native session/process gate. All 103 engine-runtime, 48 Prototype, and 20 reference-host
tests passed.

`canonical-runtime-v19` passed in 317.927 seconds with a 7,528,196-byte report, two v2 Prototype
checkpoint occurrences, and zero retired fields. It recorded 1,037 Sidecar invocations, four
warm/eight measured publications, stable 499 handles/23 threads, private bytes
`409,800,704 -> 410,427,392`, 2/2 lifecycle cycles, and 24 artifacts / 25,346,264 bytes.

The post-commit cleanup removed resolved workspace-local `target/` and `out/`, which contained
7,740 files / 3,253,478,132 bytes combined. Neither path remained afterward; `.task/`, shared
global caches, and tracked state were unchanged.
