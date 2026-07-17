# Experiment 0140: Retired Negative Session Fields and Resource Cleanup

Status: Accepted

## Hypothesis

Six static false-valued Prototype session fields and their acceptance copies can be deleted
without weakening current session, input, object, process, or resource authority. Their names only
reported the absence of event streams, event history, and copied object state; current positive
state and ownership already prove the live contract. Workspace-local compiler and generated-output
trees can then be removed after validation without affecting tracked state.

## Scope

- Delete `eventStream` from the readiness session contract.
- Delete `eventHistory` and four `copiedObjectState` fields from readiness/completion object
  reports.
- Delete every Rust test assertion, TypeScript check, and acceptance-summary copy of those fields.
- Advance the sole Prototype session schema directly from v1 to v2 without a decoder or alias.
- Make the existing Prototype session guard reject restoration in all six current owners.
- Advance the complete Prototype and Runtime workflow revisions from v54/v18 to v55/v19.
- Measure and remove only workspace-local `target/` and `out/` after validation and the guarded
  commit.

Product input/action behavior, two-value session framing, positive object state, Runtime,
renderer/GPU resources, source formats, Sidecar ownership, shared global tool caches, `.task/`, and
source assets are out of scope.

## Workload

1. Audit every static negative session-field occurrence in product, tests, and acceptance support.
2. Delete the complete field/check/summary chain and advance the exact current schema revisions.
3. Add one static current-owner scan to the existing Prototype session guard.
4. Run focused session tests, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :canonical-runtime`.
5. Record exact workspace resource counts and bytes; after the guarded commit, resolve `target/`
   and `out/`, prove both remain direct workspace descendants, and delete them with native
   PowerShell filesystem operations.

## Controlled Variables

- Readiness/completion remain exactly two JSON values from one process with sequences 1/2 and
  graceful completion only.
- Actor, clock, frame, observation target, interaction capacity/pending/acknowledgement/counters/
  consumed/exclusion, and current feedback fields remain unchanged.
- Every native input session, bootstrap/source failure, traversal, resource, and lifecycle gate
  remains live.
- No v1 decoder, optional-field path, replacement negative flag, cleanup wrapper, or global-cache
  mutation is added.

## Metrics

- Removed product/report fields and acceptance occurrences; session/prototype/runtime revisions;
  report field absence; focused and total test counts; full workflow durations/report bytes;
  Sidecar invocations; warm/measured publications; handle/thread/private-byte checkpoint;
  lifecycle cycles; artifact count/bytes; pre-cleanup path/file/byte totals; post-cleanup path
  absence; `.task/` preservation; and final tracked worktree state.

## Acceptance Criteria

- No live product, test, or acceptance owner contains any retired negative session field.
- The Prototype session revision is exactly v2; no v1 alias or decoder remains.
- The sole existing session guard scans all six current owners and rejects any field restoration.
- Prototype reports contain the v2 session contract and zero retired fields while retaining every
  current native session/process gate.
- Runtime reports contain the v2 Prototype checkpoint and zero retired fields while resource and
  lifecycle checkpoints pass.
- The guarded commit contains no generated file. After it completes, `target/` and `out/` are
  absent, while `.task/`, shared global caches, and tracked state remain intact.

## Results

The cleanup removed six product JSON fields and fourteen downstream test/check/summary
occurrences: twenty settled negative-field occurrences in total. The session schema advances
directly from `live-prototype-session-completion-v1` to v2. The existing Prototype session guard
now scans product session code, session tests, session acceptance, observation acceptance,
interaction acceptance, and object gates; no compatibility decoder or replacement report field
exists.

Both focused session-report tests passed. `runseal :guard` passed with zero Flavor denies and five
existing warnings. All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed.

`canonical-prototype-v55` passed on its first run in 170.874 seconds. Its 439,342-byte report
contains 58 v2 session-contract occurrences and zero retired fields; every existing native
session/process, failure, Sidecar, finite-boundary, focus, camera, movement, Jump, and object gate
remained unchanged.

`canonical-runtime-v19` passed in 317.927 seconds. Its 7,528,196-byte report contains two v2
Prototype checkpoint occurrences and zero retired fields. It recorded 1,037 Sidecar invocations,
four warm and eight measured publications, stable 499 handles and 23 threads, private bytes
`409,800,704 -> 410,427,392`, 2/2 lifecycle cycles, and 24 artifacts totaling 25,346,264 bytes.

The initial cleanup audit measured 6,773 `target/` files / 2,662,278,418 bytes and 88 `out/` files
/ 183,629,126 bytes. After the guarded commit, the verified resolved paths contained 7,616
`target/` files / 2,868,976,896 bytes and 124 `out/` files / 384,501,236 bytes: 7,740 files /
3,253,478,132 bytes combined. Both direct workspace descendants were then
deleted; neither path remained, `.task/` remained, shared global caches were untouched, and no
tracked file changed.

## Conclusion

Accepted. Static negative telemetry and every downstream compatibility expectation are gone;
current positive session/object/process authority remains exact under session schema v2, and
workspace-local compiler/generated resources were reclaimed without adding a cleanup surface or
touching shared caches.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, maintained Sidecar lifecycle, and the
repository-local Cargo target/output trees.

## Reproduction

```powershell
cargo test --locked -p prototype --test session_report
runseal :guard
runseal :canonical-prototype
runseal :canonical-runtime
```

The final resource deletion is intentionally not a recurring wrapper command.
