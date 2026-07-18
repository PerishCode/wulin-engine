# Experiment 0159: Retired Object-Action Readiness Baseline

Status: Accepted

## Hypothesis

The Rejected object-action session can validate its own pre-action startup and
actor state without a separate readiness-only baseline process. The host already
authors the exact schema-2 bootstrap document, and the session's raw readiness
precedes every native action.

## Scope

- Delete the `objectActionBaseline` readiness-only process and report field.
- Replace both object-feedback baseline parameters with exact host-document
  startup expectations.
- Derive expected config byte count and SHA-256 from the same serialization
  written to disk.
- Validate exact revision, mode, config path, source paths, signed origin and
  center, active radius, and singleton playable bounds.
- Retain direct actor validation against each expected center.
- Make the existing Prototype session guard reject the retired process/report
  spelling and require deterministic startup expectation ownership.
- Advance complete Prototype acceptance from v73 to v74.

The two plain first/restart readiness captures, all 19 graceful launches, native
input, product behavior, Runtime, renderer/GPU, source, synchronization, and
workspace resources are out of scope. The scheduled compatibility/resource
cleanup remains Experiment 0160.

## Workload

1. Inventory every producer and consumer of `objectActionBaseline`.
2. Compare its startup and actor projections with the Rejected session.
3. Measure its serialized size, process duration, and PID ownership.
4. Centralize exact schema-2 document serialization and derive its expected
   startup fact before launch.
5. Validate Activated and Rejected readiness against their respective authored
   documents and centers.
6. Delete the redundant launch, parameter, comparison, and report field.
7. Extend the current 500-line guard without adding another guard module.
8. Run formatting, type checking, `runseal :guard`,
   `runseal :canonical-prototype`, and `runseal :init`.
9. Inspect v74 for process/report removal, exact behavior preservation, report
   reduction, duration, and process cleanup.

## Controlled Variables

- Activated and Rejected retain their existing configs, exact object sources,
  native sequences, and graceful completion.
- Direct startup validation covers the same fields as the former equality but
  derives bytes and SHA-256 from the actual host serialization.
- `actorInvariant` retains capacity, live-count, generation, presentation,
  bounded epoch, center-position, grounding, and current-authority checks.
- First/restart still use two independent readiness-only processes.
- All 103 engine-runtime, 48 Prototype, and 20 reference-host tests remain
  required.
- No replacement process, compatibility alias, optional field, fallback,
  relaxed threshold, product delay, product change, or resource cleanup is
  admitted.

## Metrics

- Removed launch/report count; report bytes and reduction; workflow duration;
  graceful launch/process/output totals; startup bytes/hash equality; behavior
  gates; test totals; Flavor findings; and process cleanup.

## Acceptance Criteria

- No current source or v74 report may contain `objectActionBaseline`.
- The object-action baseline process count must fall from one to zero.
- Both object-action sessions must match their exact host-authored config byte
  count, SHA-256, paths, signed coordinates, active radius, and playable bounds.
- Both sessions must pass direct actor validation at their expected centers.
- All 19 graceful launches must retain unique PIDs, exit zero, own native input,
  emit exactly two values, and have empty stderr/trailing output.
- Activated, Rejected, sustained-capacity, finite-boundary, and the v73
  19-pair zero-copy gates must remain exact.
- The report must shrink by at least the measured 5,787 baseline bytes.
- Product, Runtime, renderer/GPU, source, synchronization, resource, and
  graceful-process-count diffs must remain empty.

## Results

The v73 audit found one 5,787-byte `objectActionBaseline` report. Its process
reached readiness in 3,539.8212 ms and was then forcibly terminated. Its startup
and actor invariant projections were exactly equal to those of the Rejected
launch, and no other consumer existed.

`canonical-prototype-v74` passed on its first full run in 163.148 seconds. Its
391,418-byte report is 11,182 bytes smaller than v73, a 2.78% reduction and
5,395 bytes beyond the measured minified baseline value.

- The report and current host/object-gate sources contain zero
  `objectActionBaseline` occurrences; the central guard retains the sole
  forbidden spelling.
- Activated and Rejected each matched an independently recomputed 494-byte
  schema-2 document. Their SHA-256 values were respectively
  `3efbe45824d7b1358588796acdee19a8b641850b5ae1ddd98ad6797ff8642f71`
  and
  `90033862def0734932b606ffea9ef1ab58f9ceb91e4ad3704df733739d3fd7b5`.
- Both first/restart readiness captures remain and used distinct PIDs.
- All 19 graceful launches retained unique PIDs, exit code zero, one native
  input owner, exactly two values, and empty stderr/trailing output.
- The independent 19-pair audit still found zero copied subtrees at or above the
  fixed 16-byte threshold.
- Activated retained one commit, one ineligible action, and 12 Activated frames;
  Rejected retained zero commits, two ineligible actions, and 12 Rejected
  frames; finite-boundary tangential Run and stationary completion remained
  exact.

All 103 engine-runtime, 48 Prototype, and 20 reference-host tests passed. Flavor
retained zero denies and five existing warnings. The central guard remained
exactly 500 lines, and no Prototype, Sidecar, Wulin, Runseal, or Deno process
remained.

## Conclusion

Accepted. The object-action sessions now prove their exact authored bootstrap
and initial actor state directly. The redundant readiness-only baseline process,
pairwise-equality parameters, and report branch are gone without weakening
behavior or lifecycle evidence.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
