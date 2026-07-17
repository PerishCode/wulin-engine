# Experiment 0135: Retired Startup Report Branches

Status: Accepted

## Hypothesis

After Experiment 0130 deleted the `startupNativeInput` report field and every startup-action route,
eight current native-session oracles retained duplicate shape checks for that impossible field.
Deleting those historical branches while retaining one removal guard over all eight owners should
reduce compatibility surface without changing any live input, timing, or product invariant.

## Scope

- Remove the retired report-field branch from camera repeat and camera re-press.
- Remove it from Run release, Run re-press, opposite locomotion, diagonal Walk, diagonal Run, and
  forward release.
- Add one static guard that scans all eight current source owners for the retired field.
- Advance the full Prototype workflow revision from v49 to v50.

Native input helpers, process/window selection, action timing, session schema, product behavior,
Runtime, renderer/GPU resources, source formats, and synchronization are out of scope.

## Workload

1. Audit every TypeScript and Rust occurrence of `startupNativeInput`.
2. Delete the eight per-session `Object.hasOwn` branches without changing adjacent live oracles.
3. Require the retired token to remain absent from all current Prototype session sources and
   present only as a forbidden token in the central guard.
4. Run formatting, type checking, the repository guard, and the complete v50 Prototype workflow.
5. Re-run initialization and repository guards after recording the accepted evidence.

## Controlled Variables

- All exact-PID, exact-window, post-readiness actions and native message sequences remain
  unchanged.
- Every live interval, actor, camera, presentation, object, clock, and completion oracle remains
  unchanged.
- Each normal process still emits exactly readiness plus completion; diagnostic failures remain
  readiness-free or completion-free as previously specified.
- No alias, fallback, decoder, compatibility report, retry, event stream, or intermediate product
  output is admitted.

## Metrics

- Retired-field occurrence count and owner coverage, native-session completion count, exact output
  count, representative movement/focus/object invariants, workflow duration, report bytes, test
  totals, Flavor findings, and process cleanup.

## Acceptance Criteria

- None of the eight current native-session oracle files may reference `startupNativeInput`.
- One central guard must scan all eight source owners and reject any return of the retired token.
- The only repository occurrences must remain the guard's forbidden-token checks.
- All normal native sessions must retain exactly two output values and their existing semantic
  oracles.
- The complete Prototype workflow, repository guard, and initialization gate must pass with no
  product or engine change.

## Results

The audit found exactly eight residual branches: camera repeat, camera re-press, Run release, Run
re-press, opposite locomotion, diagonal Walk, diagonal Run, and forward release. All eight were
deleted directly. The central Prototype-session guard now reads every owner and rejects the
retired token; the only two remaining repository occurrences are forbidden-token checks in that
guard.

`canonical-prototype-v50` passed in 174.239 seconds. All 16 normal native sessions completed with
exactly two output values. Representative unchanged evidence included forward release committing
16 Walk steps and `deltaZQ9=-512`, the focus-discontinuity actor remaining exactly unchanged, and
the Activated object action completing exactly 12 acknowledgement frames followed by two
suppression frames. The ignored report was 446,503 bytes.

All 103 engine-runtime, 45 Prototype, and 20 reference-host tests passed. `runseal :guard` retained
zero Flavor denies and five existing warnings. No Rust product, Runtime, renderer/GPU, source,
resource, or synchronization code changed.

## Conclusion

Accepted. Current native-session acceptance now validates only live schema, transport, timing, and
semantic contracts. Historical startup-report shape handling has one explicit static removal
authority and no executable compatibility branch.

## Reproduction

```powershell
runseal :canonical-prototype
runseal :guard
runseal :init
```

Generated reports remain ignored under `out/captures/`.
