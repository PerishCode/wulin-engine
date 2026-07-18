# Experiment 0154: Object Focus Evidence Single Ownership

Status: Accepted

## Hypothesis

The Activated object-focus acceptance can retain every behavioral gate while emitting recovery
input evidence exactly once. The top-level `nativeInput` value and nested
`focusRecovery.freshAction` are byte-identical, while three constant interpretation flags restate
facts already proven by exact message order, focus clock transitions, and final action counts.
Deleting those duplicates should reduce the canonical report without reducing product authority.

## Scope

- Retain the existing Activated/Rejected processes, native input, focus cancellation/resume,
  missing-target hold, recovery prefix, clock/frame, object identity, feedback, consumption,
  suppression, and graceful completion gates.
- Keep the recovery action oracle once as the admitted session's top-level `nativeInput`.
- Keep only focus-specific cancellation, resume, missing-target, hold, and clock evidence under
  `focusRecovery`.
- Delete `freshAction`, `missingTargetCommittedBeforeRecovery`,
  `staleObjectIntentsDidNotReachResumedSimulation`, and
  `freshObjectIntentsAfterFocusReadmitted` from report production.
- Make the central guard reject all four retired tokens.
- Advance canonical Prototype acceptance from v68 to v69.

Product code/output, native stimulus, process count, Runtime, renderer/GPU resources, source
formats, synchronization, schema, and workspace resource cleanup are out of scope. The mandatory
compatibility cleanup remains Experiment 0155; resource cleanup remains Experiment 0160.

## Workload

1. Measure the v68 admitted object's top-level and nested recovery input evidence.
2. Refactor the focus oracle to return a transient pair: one top-level native-input result and one
   focus-only result.
3. Delete the three constant interpretation fields without replacement.
4. Replace positive guard expectations with forbidden-token checks.
5. Run static gates and the complete canonical Prototype workflow.
6. Compare report bytes and require unchanged exact final behavior evidence.

## Controlled Variables

- The same helper calls and object recovery input validator execute once.
- Missing-target and recovery timing, source-qualified identity, interaction counts, frame counts,
  focus clock, output count, and process cleanup remain strict.
- Raw child launch/completion evidence remains unchanged.
- No compatibility alias, fallback field, decoder, report migration layer, relaxed gate, or product
  behavior is introduced.

## Metrics

- Report bytes and retired-token counts; admitted report keys; top-level input and focus-only key
  sets; native PID/window/thread/timing; final interaction/observation/clock/frame state; workflow
  duration; test counts; Flavor findings; and process cleanup.

## Acceptance Criteria

- `nativeInput` must remain present exactly once at the admitted session top level and retain the
  exact-window atomic Enter-up/F-down/Enter-down oracle.
- `focusRecovery` must retain exact cancellation/resume messages, cancelled batch thread/span,
  missing-target oracle/hold, and exact suspend/resume/reset/elapsed-backlog clock evidence.
- The four retired tokens must have zero occurrences in source and generated report.
- Final behavior must remain exactly one ineligible and one committed action, 12 Activated frames,
  zero Rejected frames, at least one suppression frame, exact source-qualified local ID 496,
  cleared target/acknowledgement, zero stalls/render blocks, exit zero, and two values.
- The report must be smaller than the v68 460,265-byte baseline.
- Product, Runtime, renderer/GPU, source, synchronization, schema, and process-count diffs must
  remain empty.

## Results

Before the change, minified `nativeInput` and `focusRecovery.freshAction` were byte-identical
349-byte values. `canonical-prototype-v69` passed on its first full run in 172.610 seconds with a
459,491-byte report, 774 bytes smaller than v68. The four retired tokens had zero source/report
occurrences.

The admitted report retained one top-level `nativeInput` with exact process/window, atomic thread
30036, 0.0025 ms span, `0.0015/0.0010` ms key intervals, ordered
Enter-up/F-down/Enter-down/Escape messages, and a 268.3063 ms delayed exit. `focusRecovery`
retained the 0.0012 ms cancelled batch, focus resume, missing-target oracle, 258.3602 ms hold, and
the exact one-suspend/one-resume/one-post-resume-reset clock delta.

Final behavior remained one ineligible and one committed action, 12 Activated frames, zero
Rejected frames, and two suppression frames. Source-qualified local ID 496 remained consumed and
excluded; target and acknowledgement were null. The process completed 336 live frames with zero
stalls/render blocks, exit zero, exactly two values, and empty stderr/trailing output.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. `runseal :guard` passed
with zero Flavor denies and five existing warnings. No product Rust, Runtime, renderer/GPU, source,
synchronization, schema, process-count, or resource-cleanup change was made.

## Conclusion

Accepted. Native recovery input now has one report owner. Focus recovery contains only focus-owned
evidence, while exact behavior and process gates remain unchanged and the canonical report is 774
bytes smaller without aliases or compatibility baggage.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
