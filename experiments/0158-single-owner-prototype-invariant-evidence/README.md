# Experiment 0158: Single-Owner Prototype Invariant Evidence

Status: Accepted

## Hypothesis

Each graceful Prototype launch can remain the sole owner of its raw execution
evidence while its paired invariant publishes only derived facts. Exact copied
object/array subtrees in both report branches add report weight and create a
second apparent authority without strengthening acceptance.

## Scope

- Audit the 16 session, Activated, Rejected, and finite-boundary raw/invariant
  pairs in the v72 report.
- Keep raw readiness, completion, native-input, object, camera, clock, and
  simulation values only in their launch report.
- Replace copied invariant values with validated booleans, counts, scalars, and
  purpose-specific projections.
- Add a pairwise runtime gate that rejects any exact copied object/array subtree
  whose minified UTF-8 representation is at least 16 bytes.
- Publish one aggregate gate fact for all 19 launch/invariant pairs.
- Extend the existing Prototype session guard without adding a guard module or
  exceeding the 500-line source limit.
- Advance complete Prototype acceptance from v72 to v73.

Product behavior, native actions, process count, Runtime, renderer/GPU, source,
synchronization, and workspace resources are out of scope. The scheduled
compatibility/resource cleanup remains Experiment 0160.

## Workload

1. Recursively compare every object/array subtree in each v72 raw launch with
   its paired invariant.
2. Measure copied subtree count and minified byte weight by session family.
3. Keep all validation reads before invariant shaping, then return only derived
   evidence.
4. Apply the same recursive comparison at runtime to all 19 pairs.
5. Make the current static guard require the gate, fixed threshold, and
   aggregate report fact.
6. Run formatting, type checking, `runseal :guard`,
   `runseal :canonical-prototype`, and `runseal :init`.
7. Independently audit v73 for zero qualifying copies, exact process behavior,
   report reduction, and process cleanup.

## Controlled Variables

- All 19 graceful launches retain their existing process and native-input
  sequences.
- The raw launch remains the only source for complete readiness, completion,
  camera rig, object identity/state, message, interval, and simulation-command
  values.
- Every existing validator still checks the raw value before emitting a
  narrower invariant fact.
- The comparison is pair-local; equal constants across unrelated launches are
  not treated as duplicate ownership.
- The fixed 16-byte threshold applies only to objects and arrays, not scalar
  values or property names.
- All 103 engine-runtime, 48 Prototype, and 20 reference-host tests remain
  required.
- No compatibility alias, decoder, fallback, dual write, optional branch,
  relaxed behavior gate, process, product change, or resource cleanup is
  admitted.

## Metrics

- Copied subtree count and minified bytes; final report bytes and reduction;
  workflow duration; graceful launch/process/output totals; behavior gates;
  test totals; Flavor findings; and process cleanup.

## Acceptance Criteria

- All 19 launch/invariant pairs must pass the fixed recursive runtime gate.
- No invariant may contain an exact raw-launch object/array subtree of at least
  16 minified UTF-8 bytes.
- The report must publish threshold 16, launch count 19, and copied count zero.
- All 19 launches must use unique PIDs, exit with code zero, own one
  `nativeInput`, emit exactly two values, and have empty stderr/trailing output.
- Existing Activated, Rejected, sustained-capacity, and finite-boundary behavior
  gates must remain exact.
- The v73 report must shrink by at least the 20,423 copied value bytes measured
  in v72.
- Product, Runtime, renderer/GPU, source, synchronization, resource, and
  process-count diffs must remain empty.

## Results

The v72 audit found 178 exact copied object/array subtrees totaling 20,423
minified value bytes across all 19 raw/invariant pairs:

- 141 copies / 16,259 bytes in the 16 session pairs;
- 14 copies / 1,783 bytes in Activated feedback;
- 14 copies / 1,565 bytes in Rejected feedback;
- 9 copies / 816 bytes in finite-boundary evidence.

Copies included complete clocks, object states, camera rigs, message and timing
arrays, identities, simulation commands, and retained-capacity action payloads.
All were already checked against the raw launch before being copied.

The first static guard run rejected an intermediate 541-line session module.
The recursive gate moved to the existing session-gates owner, leaving the
transport module at 493 lines and the central guard at exactly 500 lines.

`canonical-prototype-v73` passed in 167.176 seconds. Its 402,600-byte report is
39,285 bytes smaller than v72, an 8.89% reduction and 18,862 bytes beyond the
measured copied-value floor.

- An independent recursive audit found zero qualifying copied subtrees in all
  19 pairs.
- The report publishes revision
  `prototype-single-owner-invariant-evidence-v1`, launch count 19, threshold 16,
  and copied count zero.
- All 19 launches used unique PIDs, exited with code zero, owned `nativeInput`,
  emitted exactly two values, and had empty stderr/trailing output.
- Activated, Rejected, sustained-capacity, and finite-boundary gates retained
  their exact action, feedback, suppression, displacement, presentation, and
  completion facts.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor
retained zero denies and five existing warnings. No Prototype, Sidecar, Wulin,
Runseal, or Deno process remained.

## Conclusion

Accepted. Raw graceful launches are now the sole owners of complete execution
values. Paired invariants expose only derived acceptance facts, and a fixed
runtime gate prevents nontrivial exact subtree duplication from returning.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
