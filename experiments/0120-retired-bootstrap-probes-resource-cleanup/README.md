# Experiment 0120: Retired Bootstrap Probes and Resource Cleanup

Status: Accepted

## Hypothesis

The recurring invalid-bootstrap launches driven only by the historical `fallback` field and the
old schema-1 unit assertions can be deleted without weakening current bootstrap authority. Exact
path/projection validation plus missing-source and corrupt-payload no-readiness gates remain
sufficient current evidence. Repository-local compiler and generated-output trees can then be
removed after validation without affecting tracked state.

## Scope

- Delete all acceptance mutations that add `fallback = true`, the three resulting
  invalid-document process launches, and their three recurring report fields.
- Delete the schema-1 and fallback assertions from the mixed bootstrap unit test.
- Retain exact current schema-2 decoding, path escape rejection, invalid projection rejection,
  playable-bound ordering/containment, config size/path bounds, missing source, corrupt payload,
  readiness, restart, checkpoint, and lifecycle evidence.
- Make the existing compatibility-removal guard reject restoration of the retired bootstrap
  probes.
- Measure and remove only workspace-local `target/` and `out/` after all tests and the guarded
  commit.

Product bootstrap parsing, schema, error behavior, Runtime behavior, renderer/GPU resources,
Sidecar ownership, global tool caches, `.task/`, and source assets are out of scope.

## Workload

1. Audit every `fallback`, schema-1, invalid-document, and report-field occurrence in live code and
   acceptance support.
2. Remove the retired probes and keep current failure branches explicit.
3. Advance focused Prototype and full runtime report revisions because their shapes shrink.
4. Run reference-host tests, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :canonical-runtime`.
5. Record exact pre-cleanup paths, file counts, and byte totals; after the commit hook completes,
   resolve both paths again, prove they remain direct workspace descendants, and delete them with
   native PowerShell filesystem operations.

## Controlled Variables

- `Document` remains `deny_unknown_fields`; schema version 2 remains the sole accepted value.
- Path escape and invalid global projection unit failures remain in the renamed current test.
- Missing-source and corrupt-payload process gates must still fail before readiness.
- Successful configured readiness, restart equality, all Prototype sessions, resource checkpoint,
  and lifecycle checkpoint remain unchanged.
- No global Cargo/Deno cache or directory outside the repository is touched.

## Metrics

- Deleted compatibility literals, process launches, and report fields; source line delta; focused
  and full report revisions/durations/keys; current failure exit/readiness results; Sidecar
  invocations; warm/measured publications; handle/thread/private-byte checkpoint; lifecycle cycles;
  artifact count/bytes; pre-cleanup path/file/byte counts; post-cleanup path absence; and final
  tracked worktree state.

## Acceptance Criteria

- No live source or acceptance support contains the retired fallback mutation, old mixed-test
  name, or schema-1 bootstrap assertion.
- The sole compatibility-removal guard fails if any retired probe returns.
- Reference-host tests retain exact current decoding/path/projection/bounds evidence.
- `canonical-prototype-v36` contains missing-source and corrupt-payload failures but no
  `invalidDocument`; every existing Prototype gate passes.
- `canonical-runtime-v18` contains current bootstrap/prototype failure/checkpoint evidence but no
  `invalidDocument`; resource and lifecycle checkpoints pass.
- The guarded commit contains no generated file. After the commit hook, `target/` and `out/`
  are absent, while `.task/` and tracked state remain intact.

## Results

The compatibility change removed three recurring invalid-document process launches, three report
fields, two historical unit assertions, and their payload construction. That launch/test/report
chain shrank by 29 net lines. Its stable absence checks add 20 lines to the existing
compatibility-removal guard; including revision/init updates, the executable and acceptance diff
remains a net seven-line deletion. The general guard dispatcher remained below its 500-line limit.

All 20 reference-host tests passed. The renamed
`document_rejects_escaping_path_and_invalid_projection` test retains both current failures.
`runseal :guard` passed with zero Flavor denies.

`canonical-prototype-v36` passed in 128.894 seconds. Its report contains current missing-source and
corrupt-payload exit-code-1/no-readiness evidence and no `invalidDocument`; all input, movement,
Jump, object, session, restart, boundary, and Sidecar gates passed.

`canonical-runtime-v18` passed in 286.415 seconds. Its bootstrap keys are exactly `configPath`,
`missingSource`, `corruptPayload`, `first`, `restarted`, and `stopped`; its Prototype checkpoint
contains `corruptPayload`, readiness/restart, and Sidecar lifecycle without `invalidDocument`.
The run recorded 1,108 Sidecar invocations, 4 warm and 8 measured publications, a stable
527-handle/24-thread checkpoint, private bytes `412,213,248 -> 411,742,208`, 2/2 lifecycle cycles,
and 24 artifacts totaling 25,346,219 bytes.

The initial cleanup audit measured:

- `target/`: 11,976 files, 4,469,069,504 bytes.
- `out/`: 259 files, 441,529,744 bytes.
- Combined: 12,235 files, 4,910,599,248 bytes.

After the guarded commit rebuilt its required test artifacts, the final resolved paths contained
11,976 `target/` files / 4,469,116,945 bytes and 263 `out/` files / 441,628,335 bytes: 12,239
files / 4,910,745,280 bytes combined. Both parents exactly matched the workspace and no
Prototype/workbench process remained. The paths were then deleted; neither existed afterward,
`.task/` remained, and no tracked file changed.

## Conclusion

Accepted. Settled bootstrap compatibility probes and recurring report work are gone, current
failure/readiness/resource/lifecycle evidence remains exact, and workspace-local compiler/generated
trees were reclaimed without touching global caches or source state. No compatibility alias,
replacement rejection registry, product behavior, or engine/GPU/resource ownership was added.

## Reference Environment

Pinned Rust/Deno toolchains, Windows D3D12 reference runtime, maintained Sidecar lifecycle, and the
repository-local Cargo target/output trees.

## Reproduction

```powershell
cargo test --locked -p reference-host
runseal :guard
runseal :canonical-prototype
runseal :canonical-runtime
```

The final resource deletion is intentionally not a recurring wrapper command.
