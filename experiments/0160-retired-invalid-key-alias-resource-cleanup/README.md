# Experiment 0160: Retired Invalid-Key Alias Witness and Resource Cleanup

Status: Accepted

## Hypothesis

The recurring `invalid-camera-alias` Prototype process can be deleted without
weakening checked virtual-key authority. The maintained reference-host
normalizer rejects every key outside the `u8` domain before fixed input state,
and a focused exact `0x145` test plus static guard is a more direct owner than a
full native process that only proves no low-byte E alias.

After validation and the guarded commit, accumulated workspace-local compiler
and generated outputs can be removed without affecting tracked state.

## Scope

- Delete the `invalid-camera-alias` graceful process and session dispatch.
- Delete `postInvalidAliasSequence`.
- Delete the raw `invalidKey` report, its paired invariant, and the complete
  dedicated validator chain.
- Change the focused reference-host normalization test to input exact `0x145`
  and prove `0x45` is absent from held, pressed, and released state.
- Make the existing Prototype session guard reject every retired
  process/helper/validator/report spelling while retaining the checked
  `u8::try_from` requirement.
- Reduce the single-owner aggregate from 19 to 18 graceful pairs.
- Advance complete Prototype acceptance from v74 to v75.
- After validation and the guarded commit, measure, resolve, verify, and delete
  only workspace-local `target/` and `out/`.

Product input semantics, valid-key behavior, the other 18 graceful sessions,
Runtime, renderer/GPU, source, synchronization, `.task/`, tracked assets, Git
state, and shared/global caches are out of scope.

## Workload

1. Inventory all invalid-alias process, helper, dispatch, report, validator, and
   guard references.
2. Measure v74 process duration and raw/invariant report weight.
3. Confirm the checked product conversion and focused test boundary.
4. Move the exact `0x145 -> no 0x45` proof into the focused unit test.
5. Delete the complete recurring witness chain and invert the static guard.
6. Advance the pair count and complete Prototype revision.
7. Run focused tests, formatting, type checking, `runseal :guard`,
   `runseal :canonical-prototype`, and `runseal :init`.
8. Inspect v75 for zero retired fields, 18 exact graceful launches, zero pair
   copies, behavior preservation, and process cleanup.
9. Commit through the guard hook, then verify absolute cleanup paths remain
   direct workspace descendants and delete them with native PowerShell.

## Controlled Variables

- `HostInput` retains the same checked `usize -> u8` conversion and fixed state.
- The test changes only the out-of-range input value and adds explicit absence
  checks for its low byte.
- No valid native message, product session schema, or product behavior changes.
- All 103 engine-runtime, 48 Prototype, and 20 reference-host tests remain
  required.
- No replacement process, alias, decoder, compatibility branch, rejection
  telemetry, cleanup wrapper, shared-cache mutation, or product delay is
  admitted.

## Metrics

- Removed process/helper/validator/report occurrences and source lines; focused
  and total test counts; report bytes/reduction; workflow duration; graceful
  launch/process/output totals; pair-copy count; behavior gates; Flavor
  findings; pre/post-commit resource file/byte totals; post-cleanup path
  absence; `.task/` preservation; and tracked worktree state.

## Acceptance Criteria

- Current execution and acceptance sources must contain no retired invalid-alias
  process, helper, validator, or report spelling outside the central guard.
- Focused input normalization must reject exact `0x145` without setting any
  `0x45` held/pressed/released state.
- v75 must contain exactly 18 graceful raw/invariant pairs and no invalid-key
  report branch.
- All 18 launches must use unique PIDs, exit zero, own native input, emit two
  values, and have empty stderr/trailing output.
- The fixed pairwise copy audit must remain zero; all other current behavior
  gates must pass.
- The guarded commit must contain no generated output.
- After commit-hook validation, resolved `target/` and `out/` must be direct
  workspace children and then be absent; `.task/` and tracked state must remain.

## Results

The v74 audit measured the retired process at 4,520.2893 ms. Its minified raw
launch occupied 7,650 bytes and its paired invariant occupied 995 bytes, for
8,645 bytes of recurring report weight.

The implementation deletes the complete process/helper/dispatch/report/
validator chain. The focused reference-host test now supplies exact `0x145`
and proves the potential low-byte alias `0x45` is absent from held, pressed, and
released state. The central guard owns the sole retired spellings and continues
to require checked `u8::try_from` normalization. Across the nine implementation,
test, guard, revision, and initialization files, the change deletes 168 lines
and adds 13 without a replacement execution path.

`canonical-prototype-v75` passed on its first complete run in 157.466 seconds.
Its 374,253-byte report is 17,165 bytes smaller than v74, a 4.39% reduction and
8,520 bytes beyond the measured retired pair.

- The report contains no invalid-key branch or retired spelling.
- All 18 graceful launches used unique PIDs, exited zero, owned native input,
  emitted exactly two values, and had empty stderr/trailing output.
- First and restarted readiness used distinct PIDs.
- An independent recursive audit and the maintained runtime gate both found
  zero exact raw/invariant object or array subtree copies at or above 16
  minified UTF-8 bytes.
- Activated, Rejected, sustained-capacity, finite-boundary, focus, Jump,
  camera, Run, diagonal movement, and lifecycle behavior gates remained exact.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed.
Flavor retained zero denies and five existing warnings. The central session
guard is 498 lines, and no Prototype, Sidecar, Wulin, Runseal, Cargo, Rust, or
Deno process remained after acceptance.

The initial resource inventory contained 9,953 `target/` files occupying
3,353,659,751 bytes and 127 `out/` files occupying 377,399,593 bytes: 10,080
files and 3,731,059,344 bytes combined. After final guarded validation, the
resolved direct workspace children contained:

- `target/`: 9,953 files and 3,353,610,043 bytes;
- `out/`: 129 files and 377,402,846 bytes;
- combined: 10,082 files and 3,731,012,889 bytes.

The guarded commit completed before cleanup. Native PowerShell then verified
both resolved paths were direct children of the workspace and recursively
removed only those two trees. Both paths were absent afterward, `.task/`
remained present, tracked state remained clean, and no relevant process was
running.

## Conclusion

Accepted. Checked invalid-key rejection remains with its direct product owner;
the recurring native compatibility witness and its report chain are gone. The
scheduled workspace-local resource cleanup completed without introducing a
repository cleanup surface or touching retained task state and shared caches.

## Reproduction

```powershell
cargo test --locked -p reference-host input::tests::normalization_exposes_only_state_changing_edges_and_focus_cleanup
runseal :guard
runseal :canonical-prototype
runseal :init
```

The final resource deletion is intentionally not a recurring wrapper command.
