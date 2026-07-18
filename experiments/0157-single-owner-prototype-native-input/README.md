# Experiment 0157: Single-Owner Prototype Native Input

Status: Accepted

## Hypothesis

Every graceful Prototype launch can expose its native input through one
`nativeInput` owner without weakening process or behavior evidence. The existing
`postReadinessInput` / `exitInput` pair is redundant: 17 of 19 launches
serialize the exact terminal value twice, while the other launches split or pad
one execution's evidence across two generic fields.

## Scope

- Replace both raw graceful-launch input fields with one `nativeInput` value.
- Keep compound action phases explicitly named inside that value.
- Store the sustained capacity session's separately posted Escape under
  `nativeInput.terminal`.
- Let the window-close session own its direct close evidence as the complete
  `nativeInput`.
- Move all current camera, Jump, locomotion, boundary, object, focus, and
  window-close validators to the sole owner.
- Delete the 17 terminal-value equality assertions made necessary only by
  duplicate serialization.
- Extend the existing 500-line Prototype session guard to reject both retired
  field spellings.
- Advance complete Prototype acceptance from v71 to v72.

Native messages, timing bounds, PID/window/thread ownership, process count,
product output, Prototype policy, Runtime, renderer/GPU resources, source
formats, synchronization, and workspace resources are out of scope. The next
scheduled compatibility/resource cleanup remains Experiment 0160.

## Workload

1. Inventory raw v71 launch fields and compare every nested post-readiness value
   with its terminal field.
2. Measure exact duplicate count and minified byte weight.
3. Replace the producer's two report variables with one `nativeInput` plus an
   internal, non-reported terminal reference.
4. Preserve compound shapes; fold the only separate sustained Escape under
   `terminal`.
5. Migrate all current validators and delete deep-equality-only duplicate
   checks.
6. Make the current 500-line guard reject the old field spellings across every
   producer and validator owner.
7. Run formatting, type checking, `runseal :guard`,
   `runseal :canonical-prototype`, and `runseal :init`.
8. Inspect v72 for exact raw-owner counts, zero retired fields, report
   reduction, unchanged behavior gates, and process cleanup.

## Controlled Variables

- All 19 graceful sessions retain their existing process launch and completion.
- Exact native action objects remain byte-for-byte shaped as schema 4; only
  their report location changes.
- Compound sessions retain every initial, suspended, resumed, motion, hold,
  recovery, and terminal phase under one owner.
- The sustained capacity flow still posts its independent final Escape after the
  existing dwell.
- Window close remains a direct `WM_CLOSE`; no Escape alias is introduced.
- All 103 engine-runtime, 48 Prototype, and 20 reference-host tests remain
  required.
- No compatibility decoder, optional fallback, dual write, report-version
  branch, retry, product delay, relaxed threshold, process, product behavior, or
  resource cleanup is admitted.

## Metrics

- Raw field and duplicate-value counts; duplicate minified bytes; final report
  bytes and reduction; workflow duration; graceful launch/process/output totals;
  exact object feedback/action counts; test totals; Flavor findings; and process
  cleanup.

## Acceptance Criteria

- Each of the 19 graceful launch reports must contain exactly one `nativeInput`.
- Current acceptance sources and the v72 report must contain zero
  `postReadinessInput` and `exitInput` fields.
- No terminal action value may be serialized twice under one raw launch.
- The sustained capacity session must retain one exact independent Escape under
  `terminal`; window close must retain direct `WM_CLOSE` as its sole input.
- Every migrated validator must pass its unchanged exact message, timing,
  process, actor, clock, frame, object, and completion checks.
- All 19 launches must have unique PIDs, exit code zero, exactly two output
  values, and empty stderr/trailing output.
- The report must shrink by at least the 11,217 minified bytes measured in
  duplicated values.
- Product, Runtime, renderer/GPU, source, synchronization, resource, and
  process-count diffs must remain empty.

## Results

The v71 audit found 19 `postReadinessInput` and 19 `exitInput` raw fields.
Seventeen nested terminal values were exact duplicates totaling 11,217 minified
bytes. The sustained capacity launch carried a distinct post-readiness action
and Escape, while window close paired a null post-readiness value with its
direct close input. No generic completion validator consumed either field.
Eleven branch validators performed only a deep-equality check against the
duplicate; the other six duplicate fields were ignored.

An attempted extra guard module was rejected before full acceptance: it created
an eleventh guard directory child, a 501-line wrapper, and an overlong symbol.
The scaffold was deleted. The final design extends the existing Prototype
session guard, which remains exactly 500 lines and keeps the guard directory at
ten source children.

`canonical-prototype-v72` passed on its first full run in 174.608 seconds. Its
441,885-byte report is 21,590 bytes smaller than v71, a 4.66% reduction and
10,373 bytes beyond the measured duplicate payload floor.

- All 19 raw launches contain one `nativeInput`; zero raw launch owns either
  retired field.
- The report contains zero retired-field occurrences and exactly one `terminal`,
  for the sustained capacity session's independent Escape. Window close owns
  direct `WM_CLOSE` as its complete input.
- All 19 launches used unique PIDs, returned exit code zero and exactly two
  values, and emitted empty stderr/trailing output.
- Activated retained one commit, one ineligible action, and 12 feedback frames.
  Rejected retained zero commits, two ineligible actions, and 12 feedback
  frames. Sustained retained one commit, one post-readiness ineligible action,
  12 Activated frames, and 12 capacity-Rejected frames.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor
retained zero denies and five existing warnings. No Prototype, Sidecar, Wulin,
or Runseal process remained.

## Conclusion

Accepted. Graceful Prototype native input now has one raw report owner per
launch. Both historical split fields, all duplicate terminal serialization, and
equality-only compatibility checks are gone without changing product execution
or acceptance strength.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
