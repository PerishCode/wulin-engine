# Experiment 0156: Native Retained-Target Outside-Radius Admission

Status: Accepted

## Hypothesis

The existing Rejected Prototype process can prove the current `OutsideRadius` object-action outcome
without another process or nearest-object scan. After the accepted 12-frame `OutsideFacing`
acknowledgement, the actor can move away from the same retained resolved target and submit Enter
alone; exact source-derived proximity, identity, action counts, and unchanged feedback counts
uniquely distinguish `OutsideRadius`.

## Scope

- Extend the existing Rejected object-feedback child after its initial F/Enter action and 12-frame
  red acknowledgement.
- Release F/Enter, hold D for at least 500 ms, release D, and submit Enter alone before the existing
  delayed Escape.
- Keep every input in the same exact PID, window, and native window thread.
- Derive the final target delta and squared Q18 distance independently from the source oracle and
  signed-region/half-open-local-Q9 positions.
- Require the same retained source-qualified target to remain resolved after motion.
- Advance complete Prototype acceptance from v70 to v71.

Product Rust, object policy, target acquisition, Runtime, renderer/GPU resources, source formats,
synchronization, process count, workspace resources, and the inclusive 512-Q9 radius are out of
scope. The next scheduled compatibility/resource cleanup remains Experiment 0160.

## Workload

1. Establish the existing source-derived local ID 495 target and submit the current stationary
   OutsideFacing rejection.
2. Keep that process alive until the exact 12-frame Rejected acknowledgement has expired.
3. Atomically post F-up, Enter-up, and D-down, wait at least 500 ms inside the same native helper,
   then post D-up.
4. Submit Enter alone and delay Escape by the existing bounded interval.
5. Prove an exact positive-X 32-Q9 Walk lattice, final Survey state, stable actor handle, and zero
   final velocity.
6. Compute the retained target's final signed delta and Q18 squared distance independently from its
   cooked-source terrain position.
7. Run formatting, type checking, `runseal :guard`, `runseal :canonical-prototype`, and
   `runseal :init`.

## Controlled Variables

- The initial Rejected source fixture, identity tie rules, 12-frame feedback lifetime, and object
  policy remain unchanged.
- The final action contains Enter but not F, so it cannot perform another nearest scan or replace
  the retained target.
- The retained target must remain resolved under the same publication token and source namespace.
- Exact `committed=0`, `ineligible=2`, `Activated=0`, `Rejected=12`, and `suppression=0` totals
  distinguish the new feedback-free range rejection from another projected action.
- Native motion down/up timing is measured inside one helper; helper preparation cannot extend an
  unreported held interval.
- No new process, retry, fallback, compatibility alias, telemetry, product delay, product state,
  resource owner, or relaxed threshold is admitted.

## Metrics

- Workflow duration and report bytes; PID/window/thread identity; input batch spans and measured
  holds; exact actor translation; target Q9 delta and Q18 squared distance; source-qualified
  identity and snapshot; action/feedback/frame counts; process output and cleanup; tests and Flavor
  findings.

## Acceptance Criteria

- Initial F/Enter, F-up/Enter-up/D-down/D-up, and final Enter/Escape messages must target one exact
  PID, window, and thread in the required order.
- D must remain held for 500..=1,000 ms within one measured native helper interval.
- The actor must finish at least ten positive-X 32-Q9 Walk steps from readiness, with zero Z
  translation, unchanged handle/presentation, and zero vertical velocity.
- The exact retained target must remain resolved and its final distance must exceed `512² = 262,144`
  Q18.
- The final action must contain Enter without F, commit exactly one additional ineligible outcome,
  and add no Activated, Rejected, or suppression frame.
- The child must complete with exit code zero, exactly two output values, empty stderr/trailing
  output, and no remaining Prototype or Sidecar process.
- Product, Runtime, renderer/GPU, source, synchronization, resource, and process-count diffs must
  remain empty.

## Results

A preliminary v71 run proved the product outcome but was rejected as acceptance evidence: D-down and
D-up used separate helpers, so preparation of the second helper extended motion beyond the reported
505.2903 ms lower-bound wait and produced 3,104 Q9 of translation. The accepted design puts D-down,
its monotonic delay, and D-up inside one native helper, making the complete held interval
observable.

The clean `canonical-prototype-v71` run passed in 172.403 seconds with a 463,475-byte report. The
Rejected child used PID 23372, window `25888704`, and native thread 32748 throughout:

- Initial F/Enter posted in a 0.001 ms atomic batch and remained admitted for 265.0754 ms.
- F-up/Enter-up/D-down posted in a 0.0025 ms atomic prefix; D-up followed after a measured 513.2267
  ms interval.
- The final action posted Enter without F in a 0 ms batch, followed by Escape after 263.1885 ms.

The actor moved exactly `(+960,0)` Q9, equal to 30 positive-X 32-Q9 Walk steps, and finished at
local `(960,0)` with the same generation-1 handle, Survey presentation, and zero step velocity. The
same resolved source-qualified local ID 495 remained retained under publication token 2. Its exact
final delta was `(-1184,-32)` Q9 and its squared distance was 1,402,880 Q18, strictly outside the
inclusive 262,144-Q18 gate.

The child ended with `committed=0`, `ineligible=2`, 12 Rejected frames, zero Activated/suppression/
render-block frames, 195 object-target frames, and 260 live frames. It returned exit code zero,
exactly two output values, and empty stderr/trailing output. All 103 engine-runtime, 48 Prototype,
and 20 reference-host tests passed. Flavor retained zero denies and five existing warnings. No
Prototype, Sidecar, Wulin, or Runseal process remained.

## Conclusion

Accepted. The real Prototype now proves current retained-target `OutsideRadius` admission after an
accepted rejection lifetime, using exact same-process native input and an independent source/Q9
oracle without adding a scan, process, product path, or compatibility surface.

## Reproduction

```powershell
runseal :guard
runseal :canonical-prototype
runseal :init
```

Generated reports remain ignored under `out/captures/`.
